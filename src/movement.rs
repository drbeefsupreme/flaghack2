use macroquad::prelude::Vec2;

#[derive(Clone, Copy, Debug, Default)]
pub struct InputState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

pub fn movement_delta(input: InputState, speed: f32, dt: f32) -> Vec2 {
    let mut direction = Vec2::ZERO;

    if input.up {
        direction.y -= 1.0;
    }
    if input.down {
        direction.y += 1.0;
    }
    if input.left {
        direction.x -= 1.0;
    }
    if input.right {
        direction.x += 1.0;
    }

    if direction.length() > 0.0 {
        direction = direction.normalize();
    }

    direction * speed * dt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movement_delta_scales_with_speed_and_time() {
        let input = InputState {
            right: true,
            ..Default::default()
        };
        let delta = movement_delta(input, 100.0, 0.5);

        assert!((delta.x - 50.0).abs() < 0.001);
        assert!(delta.y.abs() < 0.001);
    }

    #[test]
    fn movement_delta_normalizes_diagonal() {
        let input = InputState {
            up: true,
            right: true,
            ..Default::default()
        };
        let delta = movement_delta(input, 100.0, 1.0);
        let expected = 100.0 / 2_f32.sqrt();

        assert!((delta.x - expected).abs() < 0.01);
        assert!((delta.y + expected).abs() < 0.01);
    }
}
