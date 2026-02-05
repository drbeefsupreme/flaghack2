use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Fire {
    pub pos: Vec2,
    pub size: Vec2,
    pub angle: f32,
    pub intensity: f32,
}

impl Fire {
    pub fn new(pos: Vec2, size: Vec2) -> Self {
        Self {
            pos,
            size,
            angle: -std::f32::consts::FRAC_PI_2,
            intensity: 1.0,
        }
    }
}

pub fn draw_fire(fire: Fire, time: f32) {
    let intensity = fire.intensity.clamp(0.0, 1.6);
    if intensity <= 0.01 {
        return;
    }

    let seed = fire.pos.x * 0.013 + fire.pos.y * 0.021;
    let sway = (time * 3.3 + seed).sin() * 0.08;
    let flicker = (time * 6.1 + seed * 1.7).sin() * 0.18;

    let height = fire.size.y * (0.85 + 0.2 * (time * 2.7 + seed).sin());
    let width = fire.size.x * (0.75 + 0.25 * (time * 3.1 + seed).cos());

    let angle = fire.angle + sway * 0.35;
    let dir = vec2(angle.cos(), angle.sin());
    let perp = vec2(-dir.y, dir.x);

    let base = fire.pos;
    let point = |forward: f32, lateral: f32| base + dir * forward + perp * lateral;

    let glow = Color::new(1.0, 0.42, 0.12, 0.2 * intensity);
    let ember = Color::new(1.0, 0.6, 0.2, 0.35 * intensity);
    draw_circle(base.x, base.y, width * 0.45, glow);
    draw_circle(base.x, base.y, width * 0.28, ember);

    let outer = Color::new(1.0, 0.38, 0.1, 0.9 * intensity);
    let inner = Color::new(1.0, 0.75, 0.25, 0.9 * intensity);
    let core = Color::new(1.0, 0.95, 0.7, 0.85 * intensity);

    let outer_tip = point(height * (1.05 + flicker * 0.25), 0.0);
    let outer_left = point(height * 0.12, width * 0.6);
    let outer_right = point(height * 0.12, -width * 0.6);
    draw_triangle(outer_tip, outer_left, outer_right, outer);

    let mid_tip = point(height * (0.72 + flicker * 0.2), 0.0);
    let mid_left = point(height * 0.15, width * 0.38);
    let mid_right = point(height * 0.15, -width * 0.38);
    draw_triangle(mid_tip, mid_left, mid_right, inner);

    let core_tip = point(height * (0.45 + flicker * 0.1), 0.0);
    let core_left = point(height * 0.18, width * 0.22);
    let core_right = point(height * 0.18, -width * 0.22);
    draw_triangle(core_tip, core_left, core_right, core);

    let spark_count = 3;
    for i in 0..spark_count {
        let t = i as f32 / spark_count as f32;
        let offset = (time * 4.0 + seed + t * 3.7).sin() * width * 0.15;
        let spark_pos = point(height * (0.7 + t * 0.35), offset);
        draw_circle(spark_pos.x, spark_pos.y, width * 0.06, ember);
    }
}
