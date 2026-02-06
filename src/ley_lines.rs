use macroquad::prelude::*;
use crate::flags::Flag;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeyLine {
    pub a: Vec2,
    pub b: Vec2,
    pub intensity: f32,
}

pub fn compute_ley_lines(flags: &[Flag], max_distance: f32) -> Vec<LeyLine> {
    let mut lines = Vec::new();

    if flags.len() < 2 || max_distance <= 0.0 {
        return lines;
    }

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
                lines.push(LeyLine { a, b, intensity });
            }
        }
    }

    lines
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
        let expected = (1.0 - 10.0 / 50.0).powi(2);
        assert!((lines[0].intensity - expected).abs() < 1e-6);
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
}
