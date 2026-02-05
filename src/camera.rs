use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct CameraState {
    pub zoom: f32,
    pub pan: Vec2,
    drag_last: Option<Vec2>,
}

pub const DEFAULT_ZOOM: f32 = 4.0;

impl CameraState {
    pub fn new() -> Self {
        Self {
            zoom: DEFAULT_ZOOM,
            pan: Vec2::ZERO,
            drag_last: None,
        }
    }

    pub fn begin_drag(&mut self, screen_pos: Vec2) {
        self.drag_last = Some(screen_pos);
    }

    pub fn drag(&mut self, screen_pos: Vec2) -> Option<Vec2> {
        let last = self.drag_last?;
        self.drag_last = Some(screen_pos);
        Some(screen_pos - last)
    }

    pub fn end_drag(&mut self) {
        self.drag_last = None;
    }
}

pub fn view_size(screen: Vec2, zoom: f32) -> Vec2 {
    vec2(screen.x / zoom, screen.y / zoom)
}

pub fn clamp_target(target: Vec2, map_size: Vec2, view: Vec2) -> Vec2 {
    let mut clamped = target;

    let half_w = view.x * 0.5;
    let half_h = view.y * 0.5;

    if map_size.x <= view.x {
        clamped.x = map_size.x * 0.5;
    } else {
        clamped.x = clamped.x.clamp(half_w, map_size.x - half_w);
    }

    if map_size.y <= view.y {
        clamped.y = map_size.y * 0.5;
    } else {
        clamped.y = clamped.y.clamp(half_h, map_size.y - half_h);
    }

    clamped
}

pub fn flip_zoom_y(zoom: Vec2) -> Vec2 {
    vec2(zoom.x, -zoom.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_size_scales_with_zoom() {
        let size = view_size(vec2(800.0, 600.0), 2.0);
        assert_eq!(size, vec2(400.0, 300.0));
    }

    #[test]
    fn clamp_target_keeps_inside_bounds() {
        let target = vec2(10.0, 10.0);
        let map = vec2(1000.0, 800.0);
        let view = vec2(400.0, 300.0);
        let clamped = clamp_target(target, map, view);
        assert!(clamped.x >= 200.0);
        assert!(clamped.y >= 150.0);
    }

    #[test]
    fn clamp_target_centers_when_view_larger_than_map() {
        let target = vec2(0.0, 0.0);
        let map = vec2(200.0, 100.0);
        let view = vec2(400.0, 300.0);
        let clamped = clamp_target(target, map, view);
        assert_eq!(clamped, vec2(100.0, 50.0));
    }

    #[test]
    fn flip_zoom_y_inverts_sign() {
        let zoom = vec2(2.0, -3.0);
        let flipped = flip_zoom_y(zoom);
        assert_eq!(flipped, vec2(2.0, 3.0));
    }

    #[test]
    fn camera_state_defaults_to_zoom() {
        let state = CameraState::new();
        assert!((state.zoom - DEFAULT_ZOOM).abs() < f32::EPSILON);
    }
}
