use macroquad::prelude::*;
use crate::flags::Flag;
use std::collections::HashSet;

const PENTAGRAM_RADIUS_TOLERANCE: f32 = 0.45;
const PENTAGRAM_ANGLE_TOLERANCE: f32 = 0.7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeyLineKind {
    Normal,
    Pentagram,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LeyState {
    pub lines: Vec<LeyLine>,
    pub pentagram_centers: Vec<Vec2>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeyLine {
    pub a: Vec2,
    pub b: Vec2,
    pub intensity: f32,
    pub kind: LeyLineKind,
}

pub fn compute_ley_state(flags: &[Flag], max_distance: f32) -> LeyState {
    let mut lines = Vec::new();

    if flags.len() < 2 || max_distance <= 0.0 {
        return LeyState {
            lines,
            pentagram_centers: Vec::new(),
        };
    }

    let pentagrams = if flags.len() >= 5 {
        find_pentagrams(flags, max_distance)
    } else {
        Vec::new()
    };
    let pentagram_pairs = pentagram_pairs(&pentagrams);
    let max_d2 = max_distance * max_distance;

    for i in 0..flags.len() {
        for j in (i + 1)..flags.len() {
            let a = flags[i].pos;
            let b = flags[j].pos;
            let d2 = a.distance_squared(b);
            if d2 <= max_d2 {
                let d = d2.sqrt();
                let t = 1.0 - (d / max_distance);
                let intensity = (t * t).clamp(0.0, 1.0);
                let kind = if pentagram_pairs.contains(&(i, j)) {
                    LeyLineKind::Pentagram
                } else {
                    LeyLineKind::Normal
                };
                lines.push(LeyLine {
                    a,
                    b,
                    intensity,
                    kind,
                });
            }
        }
    }

    let pentagram_centers = pentagrams.iter().map(|p| p.center).collect();

    LeyState {
        lines,
        pentagram_centers,
    }
}

#[cfg(test)]
pub fn compute_ley_lines(flags: &[Flag], max_distance: f32) -> Vec<LeyLine> {
    compute_ley_state(flags, max_distance).lines
}

#[cfg(test)]
pub fn pentagram_centers(flags: &[Flag], max_distance: f32) -> Vec<Vec2> {
    compute_ley_state(flags, max_distance).pentagram_centers
}

struct Pentagram {
    indices: [usize; 5],
    center: Vec2,
}

fn pentagram_pairs(pentagrams: &[Pentagram]) -> HashSet<(usize, usize)> {
    let mut pairs = HashSet::new();
    for pentagram in pentagrams {
        for i in 0..5 {
            for j in (i + 1)..5 {
                let mut u = pentagram.indices[i];
                let mut v = pentagram.indices[j];
                if u > v {
                    std::mem::swap(&mut u, &mut v);
                }
                pairs.insert((u, v));
            }
        }
    }

    pairs
}

fn find_pentagrams(flags: &[Flag], max_distance: f32) -> Vec<Pentagram> {
    let mut pentagrams = Vec::new();
    if flags.len() < 5 {
        return pentagrams;
    }

    for a in 0..flags.len() - 4 {
        for b in (a + 1)..flags.len() - 3 {
            for c in (b + 1)..flags.len() - 2 {
                for d in (c + 1)..flags.len() - 1 {
                    for e in (d + 1)..flags.len() {
                        let indices = [a, b, c, d, e];
                        if let Some(center) = pentagram_center(indices, flags, max_distance) {
                            pentagrams.push(Pentagram { indices, center });
                        }
                    }
                }
            }
        }
    }

    pentagrams
}

fn pentagram_center(
    indices: [usize; 5],
    flags: &[Flag],
    max_distance: f32,
) -> Option<Vec2> {
    let mut centroid = Vec2::ZERO;
    for idx in indices {
        centroid += flags[idx].pos;
    }
    centroid /= 5.0;

    let mut points: Vec<(f32, usize, Vec2, f32)> = Vec::with_capacity(5);
    let mut min_r = f32::MAX;
    let mut max_r: f32 = 0.0;
    let mut sum_r = 0.0;

    for idx in indices {
        let pos = flags[idx].pos;
        let offset = pos - centroid;
        let angle = offset.y.atan2(offset.x);
        let r = offset.length();
        min_r = min_r.min(r);
        max_r = max_r.max(r);
        sum_r += r;
        points.push((angle, idx, pos, r));
    }

    let mean_r = sum_r / 5.0;
    if mean_r <= f32::EPSILON {
        return None;
    }

    if (max_r - min_r) / mean_r > PENTAGRAM_RADIUS_TOLERANCE {
        return None;
    }

    points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let expected = std::f32::consts::TAU / 5.0;
    for i in 0..5 {
        let angle = points[i].0;
        let next_angle = if i == 4 {
            points[0].0 + std::f32::consts::TAU
        } else {
            points[i + 1].0
        };
        let diff = next_angle - angle;
        if (diff - expected).abs() > PENTAGRAM_ANGLE_TOLERANCE {
            return None;
        }
    }

    for i in 0..5 {
        let a = points[i].2;
        let b = points[(i + 2) % 5].2;
        if a.distance(b) > max_distance {
            return None;
        }
    }

    Some(centroid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flags::Flag;

    #[test]
    fn no_lines_with_few_flags() {
        let flags: Vec<Flag> = Vec::new();
        let lines = compute_ley_lines(&flags, 100.0);
        assert!(lines.is_empty());
    }

    #[test]
    fn builds_pairs_within_radius() {
        let flags = vec![
            Flag { pos: vec2(0.0, 0.0), phase: 0.0 },
            Flag { pos: vec2(10.0, 0.0), phase: 0.0 },
            Flag { pos: vec2(300.0, 0.0), phase: 0.0 },
        ];
        let lines = compute_ley_lines(&flags, 50.0);
        assert_eq!(lines.len(), 1);
        let expected = (1.0_f32 - 10.0 / 50.0).powi(2);
        assert!((lines[0].intensity - expected).abs() < 1e-6);
        assert_eq!(lines[0].kind, LeyLineKind::Normal);
    }

    #[test]
    fn intensity_closer_is_brighter() {
        let flags = vec![
            Flag { pos: vec2(0.0, 0.0), phase: 0.0 },
            Flag { pos: vec2(10.0, 0.0), phase: 0.0 },
            Flag { pos: vec2(30.0, 0.0), phase: 0.0 },
        ];
        let lines = compute_ley_lines(&flags, 40.0);
        let mut intensities: Vec<f32> = lines.iter().map(|l| l.intensity).collect();
        intensities.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!(intensities[0] < intensities[1]);
        assert!(intensities[1] < intensities[2]);
    }

    #[test]
    fn pentagram_marks_lines() {
        let radius = 40.0;
        let mut flags = Vec::new();
        for i in 0..5 {
            let angle = i as f32 * std::f32::consts::TAU / 5.0;
            flags.push(Flag {
                pos: vec2(angle.cos() * radius, angle.sin() * radius),
                phase: 0.0,
            });
        }

        let lines = compute_ley_lines(&flags, 80.0);
        assert_eq!(lines.len(), 10);
        assert!(lines.iter().all(|line| line.kind == LeyLineKind::Pentagram));
    }

    #[test]
    fn compute_ley_state_returns_lines_and_centers() {
        let radius = 45.0;
        let mut flags = Vec::new();
        for i in 0..5 {
            let angle = i as f32 * std::f32::consts::TAU / 5.0;
            flags.push(Flag {
                pos: vec2(angle.cos() * radius, angle.sin() * radius),
                phase: 0.0,
            });
        }

        let state = compute_ley_state(&flags, 90.0);
        assert_eq!(state.pentagram_centers.len(), 1);
        assert_eq!(state.lines.len(), 10);
        assert!(state.lines.iter().all(|line| line.kind == LeyLineKind::Pentagram));
    }

    #[test]
    fn pentagram_with_jittered_radius_is_detected() {
        let base_radius = 60.0;
        let mut flags = Vec::new();
        for i in 0..5 {
            let angle = i as f32 * std::f32::consts::TAU / 5.0;
            let jitter = if i % 2 == 0 { 1.15 } else { 0.85 };
            flags.push(Flag {
                pos: vec2(angle.cos() * base_radius * jitter, angle.sin() * base_radius * jitter),
                phase: 0.0,
            });
        }

        let lines = compute_ley_lines(&flags, 150.0);
        assert_eq!(lines.len(), 10);
        assert!(lines.iter().all(|line| line.kind == LeyLineKind::Pentagram));
    }

    #[test]
    fn pentagram_centers_returns_centroid() {
        let radius = 50.0;
        let mut flags = Vec::new();
        for i in 0..5 {
            let angle = i as f32 * std::f32::consts::TAU / 5.0;
            flags.push(Flag {
                pos: vec2(angle.cos() * radius, angle.sin() * radius),
                phase: 0.0,
            });
        }

        let centers = pentagram_centers(&flags, 100.0);
        assert_eq!(centers.len(), 1);
        assert!(centers[0].length() < 1e-4);
    }

    #[test]
    fn non_pentagram_does_not_mark_lines() {
        let flags = vec![
            Flag { pos: vec2(0.0, 0.0), phase: 0.0 },
            Flag { pos: vec2(10.0, 0.0), phase: 0.0 },
            Flag { pos: vec2(20.0, 0.0), phase: 0.0 },
            Flag { pos: vec2(30.0, 0.0), phase: 0.0 },
            Flag { pos: vec2(40.0, 0.0), phase: 0.0 },
        ];
        let lines = compute_ley_lines(&flags, 100.0);
        assert!(lines.iter().all(|line| line.kind == LeyLineKind::Normal));
    }

    #[test]
    fn pentagram_centers_empty_for_non_pentagram() {
        let flags = vec![
            Flag { pos: vec2(0.0, 0.0), phase: 0.0 },
            Flag { pos: vec2(10.0, 0.0), phase: 0.0 },
            Flag { pos: vec2(20.0, 0.0), phase: 0.0 },
            Flag { pos: vec2(30.0, 0.0), phase: 0.0 },
            Flag { pos: vec2(40.0, 0.0), phase: 0.0 },
        ];
        let centers = pentagram_centers(&flags, 100.0);
        assert!(centers.is_empty());
    }
}
