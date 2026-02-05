use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facing {
    Down,
    Up,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
struct PlayerParts {
    body: Rect,
    head_center: Vec2,
    head_radius: f32,
    hat: [Vec2; 3],
    hat_band: Rect,
    hand_left: Hand,
    hand_right: Hand,
}

const BODY_W: f32 = 18.0;
const BODY_H: f32 = 24.0;
const HEAD_RADIUS: f32 = 5.0;
const HAT_W: f32 = 14.0;
const HAT_H: f32 = 10.0;
const HAT_BAND_H: f32 = 2.0;
const HAND_RADIUS: f32 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HandSide {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
struct Hand {
    pos: Vec2,
    side: HandSide,
    visible: bool,
}

pub fn draw_player(top_left: Vec2, accent: Color, facing: Facing) {
    let parts = compute_player_parts(top_left, facing);
    let robe_color = Color::new(0.08, 0.08, 0.09, 1.0);
    let outline = Color::new(0.02, 0.02, 0.02, 1.0);
    let (robe_left, robe_right, split) = robe_colors(facing, accent, robe_color);

    if split {
        let half_w = parts.body.w * 0.5;
        draw_rectangle(parts.body.x, parts.body.y, half_w, parts.body.h, robe_left);
        draw_rectangle(
            parts.body.x + half_w,
            parts.body.y,
            half_w,
            parts.body.h,
            robe_right,
        );
    } else {
        draw_rectangle(parts.body.x, parts.body.y, parts.body.w, parts.body.h, robe_left);
    }

    draw_triangle(parts.hat[0], parts.hat[1], parts.hat[2], robe_color);
    draw_rectangle(
        parts.hat_band.x,
        parts.hat_band.y,
        parts.hat_band.w,
        parts.hat_band.h,
        accent,
    );

    let head_color = head_color(facing);
    draw_circle(parts.head_center.x, parts.head_center.y, parts.head_radius, head_color);
    draw_circle_lines(parts.head_center.x, parts.head_center.y, parts.head_radius, 1.2, outline);

    draw_hand(parts.hand_left, accent, outline);
    draw_hand(parts.hand_right, accent, outline);
}

fn compute_player_parts(top_left: Vec2, facing: Facing) -> PlayerParts {
    let body_x = top_left.x + (PLAYER_WIDTH - BODY_W) * 0.5;
    let body_y = top_left.y + PLAYER_HEIGHT - BODY_H;

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
    let left_pos = vec2(body_x - HAND_RADIUS - 1.0, hand_y);
    let right_pos = vec2(body_x + BODY_W + HAND_RADIUS + 1.0, hand_y);
    let center_pos = vec2(body_x + BODY_W * 0.5, hand_y);

    let (left_visible, right_visible) = match facing {
        Facing::Up => (false, false),
        Facing::Left => (true, false),
        Facing::Right => (false, true),
        Facing::Down => (true, true),
    };

    PlayerParts {
        body: Rect::new(body_x, body_y, BODY_W, BODY_H),
        head_center,
        head_radius: HEAD_RADIUS,
        hat: [hat_top, hat_left, hat_right],
        hat_band,
        hand_left: Hand {
            pos: if facing == Facing::Left { center_pos } else { left_pos },
            side: HandSide::Left,
            visible: left_visible,
        },
        hand_right: Hand {
            pos: if facing == Facing::Right { center_pos } else { right_pos },
            side: HandSide::Right,
            visible: right_visible,
        },
    }
}

fn draw_hand(hand: Hand, color: Color, outline: Color) {
    if !hand.visible {
        return;
    }

    let _ = hand.side;
    draw_circle(hand.pos.x, hand.pos.y, HAND_RADIUS, color);
    draw_circle_lines(hand.pos.x, hand.pos.y, HAND_RADIUS, 1.0, outline);
}

pub fn facing_from_direction(direction: Vec2) -> Facing {
    if direction.abs().x >= direction.abs().y {
        if direction.x >= 0.0 {
            Facing::Right
        } else {
            Facing::Left
        }
    } else if direction.y >= 0.0 {
        Facing::Down
    } else {
        Facing::Up
    }
}

fn robe_colors(facing: Facing, accent: Color, robe_color: Color) -> (Color, Color, bool) {
    match facing {
        Facing::Left => (robe_color, robe_color, false),
        Facing::Right => (accent, accent, false),
        Facing::Up | Facing::Down => (accent, robe_color, true),
    }
}

fn head_color(facing: Facing) -> Color {
    match facing {
        Facing::Up => Color::new(0.86, 0.72, 0.36, 1.0),
        _ => Color::new(0.96, 0.86, 0.74, 1.0),
    }
}

pub const PLAYER_WIDTH: f32 = 30.0;
pub const PLAYER_HEIGHT: f32 = 40.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn head_is_above_body() {
        let parts = compute_player_parts(vec2(100.0, 200.0), Facing::Down);
        assert!(parts.head_center.y + parts.head_radius <= parts.body.y);
    }

    #[test]
    fn hat_is_above_head() {
        let parts = compute_player_parts(vec2(0.0, 0.0), Facing::Down);
        let hat_top = parts.hat[0];
        assert!(hat_top.y < parts.head_center.y - parts.head_radius);
    }

    #[test]
    fn hands_outside_body_edges() {
        let parts = compute_player_parts(vec2(50.0, 50.0), Facing::Down);
        assert!(parts.hand_left.pos.x < parts.body.x);
        assert!(parts.hand_right.pos.x > parts.body.x + parts.body.w);
    }

    #[test]
    fn facing_from_direction_picks_horizontal() {
        assert_eq!(facing_from_direction(vec2(1.0, 0.2)), Facing::Right);
        assert_eq!(facing_from_direction(vec2(-1.0, 0.1)), Facing::Left);
    }

    #[test]
    fn facing_from_direction_picks_vertical() {
        assert_eq!(facing_from_direction(vec2(0.1, 1.0)), Facing::Down);
        assert_eq!(facing_from_direction(vec2(0.2, -1.0)), Facing::Up);
    }

    #[test]
    fn robe_colors_split_for_front_back() {
        let accent = Color::new(1.0, 1.0, 0.0, 1.0);
        let robe = Color::new(0.1, 0.1, 0.1, 1.0);
        let (left, right, split) = robe_colors(Facing::Down, accent, robe);
        assert!(split);
        assert_eq!(left, accent);
        assert_eq!(right, robe);
    }

    #[test]
    fn robe_colors_solid_for_sides() {
        let accent = Color::new(1.0, 1.0, 0.0, 1.0);
        let robe = Color::new(0.1, 0.1, 0.1, 1.0);
        let (_, _, split_left) = robe_colors(Facing::Left, accent, robe);
        let (_, _, split_right) = robe_colors(Facing::Right, accent, robe);
        assert!(!split_left);
        assert!(!split_right);
    }

    #[test]
    fn head_color_switches_for_back() {
        let front = head_color(Facing::Down);
        let back = head_color(Facing::Up);
        assert_ne!(front, back);
    }

    #[test]
    fn hand_visibility_for_up() {
        let parts = compute_player_parts(vec2(0.0, 0.0), Facing::Up);
        assert!(!parts.hand_left.visible);
        assert!(!parts.hand_right.visible);
    }

    #[test]
    fn hand_visibility_for_sides() {
        let left = compute_player_parts(vec2(0.0, 0.0), Facing::Left);
        let right = compute_player_parts(vec2(0.0, 0.0), Facing::Right);

        assert!(left.hand_left.visible);
        assert!(!left.hand_right.visible);
        assert!(!right.hand_left.visible);
        assert!(right.hand_right.visible);
    }

    #[test]
    fn side_facing_hand_is_centered() {
        let left = compute_player_parts(vec2(20.0, 10.0), Facing::Left);
        let right = compute_player_parts(vec2(20.0, 10.0), Facing::Right);
        let body_center = left.body.x + left.body.w * 0.5;

        assert!((left.hand_left.pos.x - body_center).abs() < 0.01);
        assert!((right.hand_right.pos.x - body_center).abs() < 0.01);
    }
}
