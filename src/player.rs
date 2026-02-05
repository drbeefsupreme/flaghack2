use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
struct PlayerParts {
    body: Rect,
    stripe: Rect,
    head_center: Vec2,
    head_radius: f32,
    hat: [Vec2; 3],
    hat_band: Rect,
    hand_left: Vec2,
    hand_right: Vec2,
    hand_radius: f32,
}

const BODY_W: f32 = 18.0;
const BODY_H: f32 = 24.0;
const HEAD_RADIUS: f32 = 5.0;
const HAT_W: f32 = 14.0;
const HAT_H: f32 = 10.0;
const HAT_BAND_H: f32 = 2.0;
const HAND_RADIUS: f32 = 3.0;

pub fn draw_player(top_left: Vec2, accent: Color) {
    let parts = compute_player_parts(top_left);
    let robe_color = Color::new(0.08, 0.08, 0.09, 1.0);
    let outline = Color::new(0.02, 0.02, 0.02, 1.0);

    draw_rectangle(parts.body.x, parts.body.y, parts.body.w, parts.body.h, robe_color);
    draw_rectangle(parts.stripe.x, parts.stripe.y, parts.stripe.w, parts.stripe.h, accent);

    draw_triangle(parts.hat[0], parts.hat[1], parts.hat[2], robe_color);
    draw_rectangle(
        parts.hat_band.x,
        parts.hat_band.y,
        parts.hat_band.w,
        parts.hat_band.h,
        accent,
    );

    draw_circle(parts.head_center.x, parts.head_center.y, parts.head_radius, accent);
    draw_circle_lines(parts.head_center.x, parts.head_center.y, parts.head_radius, 1.2, outline);

    draw_circle(parts.hand_left.x, parts.hand_left.y, parts.hand_radius, accent);
    draw_circle(parts.hand_right.x, parts.hand_right.y, parts.hand_radius, accent);
    draw_circle_lines(parts.hand_left.x, parts.hand_left.y, parts.hand_radius, 1.0, outline);
    draw_circle_lines(
        parts.hand_right.x,
        parts.hand_right.y,
        parts.hand_radius,
        1.0,
        outline,
    );
}

fn compute_player_parts(top_left: Vec2) -> PlayerParts {
    let body_x = top_left.x + (PLAYER_WIDTH - BODY_W) * 0.5;
    let body_y = top_left.y + PLAYER_HEIGHT - BODY_H;

    let stripe_w = 4.0;
    let stripe = Rect::new(body_x + BODY_W * 0.5 - stripe_w * 0.5, body_y, stripe_w, BODY_H);

    let head_center = vec2(top_left.x + PLAYER_WIDTH * 0.5, top_left.y + HEAD_RADIUS + 3.0);

    let hat_base_y = head_center.y - HEAD_RADIUS * 0.6;
    let hat_top = vec2(head_center.x, hat_base_y - HAT_H);
    let hat_left = vec2(head_center.x - HAT_W * 0.5, hat_base_y);
    let hat_right = vec2(head_center.x + HAT_W * 0.5, hat_base_y);

    let hat_band = Rect::new(
        head_center.x - HAT_W * 0.4,
        hat_base_y - HAT_BAND_H * 0.5,
        HAT_W * 0.8,
        HAT_BAND_H,
    );

    let hand_y = body_y + BODY_H * 0.45;
    let hand_left = vec2(body_x - HAND_RADIUS - 1.0, hand_y);
    let hand_right = vec2(body_x + BODY_W + HAND_RADIUS + 1.0, hand_y);

    PlayerParts {
        body: Rect::new(body_x, body_y, BODY_W, BODY_H),
        stripe,
        head_center,
        head_radius: HEAD_RADIUS,
        hat: [hat_top, hat_left, hat_right],
        hat_band,
        hand_left,
        hand_right,
        hand_radius: HAND_RADIUS,
    }
}

pub const PLAYER_WIDTH: f32 = 30.0;
pub const PLAYER_HEIGHT: f32 = 40.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn head_is_above_body() {
        let parts = compute_player_parts(vec2(100.0, 200.0));
        assert!(parts.head_center.y + parts.head_radius <= parts.body.y);
    }

    #[test]
    fn hat_is_above_head() {
        let parts = compute_player_parts(vec2(0.0, 0.0));
        let hat_top = parts.hat[0];
        assert!(hat_top.y < parts.head_center.y - parts.head_radius);
    }

    #[test]
    fn stripe_centered_on_body() {
        let parts = compute_player_parts(vec2(50.0, 50.0));
        let body_center = parts.body.x + parts.body.w * 0.5;
        let stripe_center = parts.stripe.x + parts.stripe.w * 0.5;
        assert!((body_center - stripe_center).abs() < 0.01);
    }

    #[test]
    fn hands_outside_body_edges() {
        let parts = compute_player_parts(vec2(50.0, 50.0));
        assert!(parts.hand_left.x < parts.body.x);
        assert!(parts.hand_right.x > parts.body.x + parts.body.w);
    }
}
