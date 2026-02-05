pub const MODEL_SCALE: f32 = 0.25;

pub fn scaled(value: f32) -> f32 {
    value * MODEL_SCALE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaled_applies_model_scale() {
        assert!((scaled(4.0) - 1.0).abs() < f32::EPSILON);
    }
}
