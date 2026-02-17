use macroquad::prelude::*;

use crate::fire;
use crate::player;
use crate::scale;

pub const EFFIGY_HEIGHT: f32 = 200.0 * scale::MODEL_SCALE;
pub const EFFIGY_WIDTH: f32 = 100.0 * scale::MODEL_SCALE;
pub const PLATFORM_HEIGHT: f32 = 40.0 * scale::MODEL_SCALE;
pub const PLATFORM_WIDTH: f32 = 160.0 * scale::MODEL_SCALE;

pub struct Effigy {
    pub pos: Vec2,
    pub burning: bool,
    pub burn_progress: f32,
}

pub fn draw_effigy(effigy: &Effigy, time: f32) {
    let s = scale::MODEL_SCALE;
    let base = effigy.pos;

    // Platform
    let plat_color = Color::new(0.45, 0.32, 0.22, 1.0);
    draw_rectangle(
        base.x - PLATFORM_WIDTH * 0.5,
        base.y - PLATFORM_HEIGHT,
        PLATFORM_WIDTH,
        PLATFORM_HEIGHT,
        plat_color,
    );

    // Body (torso)
    let wood = Color::new(0.55, 0.40, 0.25, 1.0);
    let wood_dark = Color::new(0.40, 0.28, 0.18, 1.0);
    let body_w = 20.0 * s;
    let body_h = EFFIGY_HEIGHT * 0.5;
    let body_top = base.y - PLATFORM_HEIGHT - body_h;
    draw_rectangle(base.x - body_w * 0.5, body_top, body_w, body_h, wood);

    // Head
    let head_r = 16.0 * s;
    let head_y = body_top - head_r;
    draw_circle(base.x, head_y, head_r, wood);
    draw_circle_lines(base.x, head_y, head_r, 1.5 * s, wood_dark);

    // Arms (horizontal beam)
    let arm_w = EFFIGY_WIDTH;
    let arm_h = 12.0 * s;
    let arm_y = body_top + body_h * 0.2;
    draw_rectangle(base.x - arm_w * 0.5, arm_y, arm_w, arm_h, wood);

    // Legs
    let body_bottom = base.y - PLATFORM_HEIGHT;
    let leg_spread = 30.0 * s;
    let leg_w = 3.0 * s;
    draw_line(
        base.x - 4.0 * s,
        body_bottom,
        base.x - leg_spread,
        base.y,
        leg_w,
        wood,
    );
    draw_line(
        base.x + 4.0 * s,
        body_bottom,
        base.x + leg_spread,
        base.y,
        leg_w,
        wood,
    );

    // X-bracing lattice on torso
    let lattice_color = Color::new(0.48, 0.35, 0.22, 0.7);
    let lattice_w = 1.5 * s;
    draw_line(
        base.x - body_w * 0.5,
        body_top,
        base.x + body_w * 0.5,
        body_top + body_h,
        lattice_w,
        lattice_color,
    );
    draw_line(
        base.x + body_w * 0.5,
        body_top,
        base.x - body_w * 0.5,
        body_top + body_h,
        lattice_w,
        lattice_color,
    );

    // Fire at base when burning
    if effigy.burning {
        let fire_pos = vec2(base.x, base.y - PLATFORM_HEIGHT * 0.5);
        let fire_size = vec2(PLATFORM_WIDTH * 0.8, EFFIGY_HEIGHT * 0.6);
        fire::draw_fire(fire::Fire::new(fire_pos, fire_size), time);
    }
}

pub fn head_y(base: Vec2) -> f32 {
    let body_h = EFFIGY_HEIGHT * 0.5;
    let body_top = base.y - PLATFORM_HEIGHT - body_h;
    let head_r = 16.0 * scale::MODEL_SCALE;
    body_top - head_r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_wider_than_body() {
        assert!(PLATFORM_WIDTH > EFFIGY_WIDTH);
    }

    #[test]
    fn head_above_body() {
        let base = vec2(500.0, 500.0);
        let hy = head_y(base);
        let body_top = base.y - PLATFORM_HEIGHT - EFFIGY_HEIGHT * 0.5;
        assert!(hy < body_top);
    }

    #[test]
    fn effigy_taller_than_player() {
        assert!(EFFIGY_HEIGHT > player::PLAYER_HEIGHT * 4.0);
    }
}
