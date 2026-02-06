use crate::flags::Flag;
use macroquad::prelude::*;
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

#[derive(Clone, Copy, Debug)]
struct LineCandidate {
    indices: (usize, usize),
    a: Vec2,
    b: Vec2,
    intensity: f32,
}

pub fn compute_ley_state(flags: &[Flag], max_distance: f32) -> LeyState {
    if flags.len() < 2 || max_distance <= 0.0 {
        return LeyState {
            lines: Vec::new(),
            pentagram_centers: Vec::new(),
        };
    }

    let (line_candidates, neighbors) = build_proximity_graph(flags, max_distance);
    let pentagrams = if flags.len() >= 5 {
        find_pentagrams(flags, max_distance, &neighbors)
    } else {
        Vec::new()
    };
    let pentagram_pairs = pentagram_pairs(&pentagrams);
    let lines = line_candidates
        .into_iter()
        .map(|candidate| LeyLine {
            a: candidate.a,
            b: candidate.b,
            intensity: candidate.intensity,
            kind: if pentagram_pairs.contains(&candidate.indices) {
                LeyLineKind::Pentagram
            } else {
                LeyLineKind::Normal
            },
        })
        .collect();

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

fn build_proximity_graph(
    flags: &[Flag],
    max_distance: f32,
) -> (Vec<LineCandidate>, Vec<Vec<usize>>) {
    let mut lines = Vec::new();
    let mut neighbors = vec![Vec::new(); flags.len()];
    let max_d2 = max_distance * max_distance;

    for i in 0..flags.len() {
        for j in (i + 1)..flags.len() {
            let a = flags[i].pos;
            let b = flags[j].pos;
            let d2 = a.distance_squared(b);
            if d2 <= max_d2 {
                neighbors[i].push(j);
                neighbors[j].push(i);
                let d = d2.sqrt();
                let t = 1.0 - (d / max_distance);
                lines.push(LineCandidate {
                    indices: (i, j),
                    a,
                    b,
                    intensity: (t * t).clamp(0.0, 1.0),
                });
            }
        }
    }

    (lines, neighbors)
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

fn neighbors_after(sorted_neighbors: &[usize], min_index: usize) -> &[usize] {
    let mut start = 0;
    while start < sorted_neighbors.len() && sorted_neighbors[start] <= min_index {
        start += 1;
    }
    &sorted_neighbors[start..]
}

fn intersect_sorted(left: &[usize], right: &[usize]) -> Vec<usize> {
    let mut i = 0;
    let mut j = 0;
    let mut out = Vec::with_capacity(left.len().min(right.len()));

    while i < left.len() && j < right.len() {
        match left[i].cmp(&right[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                out.push(left[i]);
                i += 1;
                j += 1;
            }
        }
    }

    out
}

fn find_pentagrams(flags: &[Flag], max_distance: f32, neighbors: &[Vec<usize>]) -> Vec<Pentagram> {
    let mut pentagrams = Vec::new();
    if flags.len() < 5 {
        return pentagrams;
    }

    for a in 0..flags.len() - 4 {
        let b_candidates = neighbors_after(&neighbors[a], a);
        if b_candidates.len() < 4 {
            continue;
        }

        for (b_pos, &b) in b_candidates.iter().enumerate() {
            let c_candidates = intersect_sorted(&b_candidates[(b_pos + 1)..], &neighbors[b]);
            if c_candidates.len() < 3 {
                continue;
            }

            for (c_pos, &c) in c_candidates.iter().enumerate() {
                let d_candidates = intersect_sorted(&c_candidates[(c_pos + 1)..], &neighbors[c]);
                if d_candidates.len() < 2 {
                    continue;
                }

                for (d_pos, &d) in d_candidates.iter().enumerate() {
                    let e_candidates =
                        intersect_sorted(&d_candidates[(d_pos + 1)..], &neighbors[d]);
                    for &e in &e_candidates {
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

fn pentagram_center(indices: [usize; 5], flags: &[Flag], max_distance: f32) -> Option<Vec2> {
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
            Flag {
                pos: vec2(0.0, 0.0),
                phase: 0.0,
            },
            Flag {
                pos: vec2(10.0, 0.0),
                phase: 0.0,
            },
            Flag {
                pos: vec2(300.0, 0.0),
                phase: 0.0,
            },
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
            Flag {
                pos: vec2(0.0, 0.0),
                phase: 0.0,
            },
            Flag {
                pos: vec2(10.0, 0.0),
                phase: 0.0,
            },
            Flag {
                pos: vec2(30.0, 0.0),
                phase: 0.0,
            },
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
        assert!(state
            .lines
            .iter()
            .all(|line| line.kind == LeyLineKind::Pentagram));
    }

    #[test]
    fn pentagram_with_jittered_radius_is_detected() {
        let base_radius = 60.0;
        let mut flags = Vec::new();
        for i in 0..5 {
            let angle = i as f32 * std::f32::consts::TAU / 5.0;
            let jitter = if i % 2 == 0 { 1.15 } else { 0.85 };
            flags.push(Flag {
                pos: vec2(
                    angle.cos() * base_radius * jitter,
                    angle.sin() * base_radius * jitter,
                ),
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
            Flag {
                pos: vec2(0.0, 0.0),
                phase: 0.0,
            },
            Flag {
                pos: vec2(10.0, 0.0),
                phase: 0.0,
            },
            Flag {
                pos: vec2(20.0, 0.0),
                phase: 0.0,
            },
            Flag {
                pos: vec2(30.0, 0.0),
                phase: 0.0,
            },
            Flag {
                pos: vec2(40.0, 0.0),
                phase: 0.0,
            },
        ];
        let lines = compute_ley_lines(&flags, 100.0);
        assert!(lines.iter().all(|line| line.kind == LeyLineKind::Normal));
    }

    #[test]
    fn pentagram_centers_empty_for_non_pentagram() {
        let flags = vec![
            Flag {
                pos: vec2(0.0, 0.0),
                phase: 0.0,
            },
            Flag {
                pos: vec2(10.0, 0.0),
                phase: 0.0,
            },
            Flag {
                pos: vec2(20.0, 0.0),
                phase: 0.0,
            },
            Flag {
                pos: vec2(30.0, 0.0),
                phase: 0.0,
            },
            Flag {
                pos: vec2(40.0, 0.0),
                phase: 0.0,
            },
        ];
        let centers = pentagram_centers(&flags, 100.0);
        assert!(centers.is_empty());
    }

    #[test]
    fn pentagram_detected_with_many_distant_noise_flags() {
        let mut flags = Vec::new();

        for i in 0..40 {
            flags.push(Flag {
                pos: vec2(1000.0 + i as f32 * 300.0, -2000.0 + (i % 3) as f32 * 500.0),
                phase: 0.0,
            });
        }

        let radius = 42.0;
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
        assert_eq!(
            state
                .lines
                .iter()
                .filter(|line| line.kind == LeyLineKind::Pentagram)
                .count(),
            10
        );
    }
}
