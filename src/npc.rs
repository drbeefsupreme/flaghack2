use macroquad::prelude::*;

use crate::constants;
use crate::flags;
use crate::player;
use crate::scale;

const HIPPIE_SPEED: f32 = 18.0 * scale::MODEL_SCALE;
const HIPPIE_TARGET_EPSILON: f32 = 4.0 * scale::MODEL_SCALE;
const HIPPIE_BOUNDS_ATTEMPTS: usize = 16;
const HIPPIE_HEAD_RADIUS: f32 = 5.0 * scale::MODEL_SCALE;
const HIPPIE_BODY_LENGTH: f32 = 18.0 * scale::MODEL_SCALE;
const HIPPIE_ARM_LENGTH: f32 = 10.0 * scale::MODEL_SCALE;
const HIPPIE_LEG_LENGTH: f32 = 12.0 * scale::MODEL_SCALE;
const HIPPIE_HAND_RADIUS: f32 = 2.0 * scale::MODEL_SCALE;
const HIPPIE_FLAG_CAPACITY: u8 = 2;
const HIPPIE_FLAG_PICKUP_RADIUS: f32 = 20.0 * scale::MODEL_SCALE;
const HIPPIE_FLAG_ANGLE: f32 = std::f32::consts::FRAC_PI_4;

#[derive(Clone, Debug)]
pub struct Hippie {
    pub pos: Vec2,
    pub facing: player::Facing,
    pub carried_flags: u8,
    target: Vec2,
    speed: f32,
    rng_state: u32,
}

pub fn try_steal_flag(hippies: &mut [Hippie], origin: Vec2, radius: f32) -> bool {
    if let Some(index) = nearest_hippie_with_flag(hippies, origin, radius) {
        hippies[index].carried_flags = hippies[index].carried_flags.saturating_sub(1);
        return true;
    }
    false
}

pub fn spawn_hippies(positions: &[Vec2], region_vertices: &[Vec2]) -> Vec<Hippie> {
    positions
        .iter()
        .enumerate()
        .map(|(i, &pos)| {
            let mut rng_state = hash_seed(pos, i as u32);
            let target = random_point_in_polygon(region_vertices, &mut rng_state);
            Hippie {
                pos,
                facing: player::Facing::Down,
                carried_flags: 0,
                target,
                speed: HIPPIE_SPEED,
                rng_state,
            }
        })
        .collect()
}

pub fn spawn_hippies_with_flags(
    spawns: &[(Vec2, u8)],
    region_vertices: &[Vec2],
) -> Vec<Hippie> {
    spawns
        .iter()
        .enumerate()
        .map(|(i, &(pos, carried))| {
            let mut rng_state = hash_seed(pos, i as u32);
            let target = random_point_in_polygon(region_vertices, &mut rng_state);
            Hippie {
                pos,
                facing: player::Facing::Down,
                carried_flags: carried.min(HIPPIE_FLAG_CAPACITY),
                target,
                speed: HIPPIE_SPEED,
                rng_state,
            }
        })
        .collect()
}

pub fn update_hippies(
    hippies: &mut [Hippie],
    dt: f32,
    region_vertices: &[Vec2],
    flags: &mut Vec<flags::Flag>,
) -> bool {
    let mut picked_any = false;
    for hippie in hippies {
        if hippie.carried_flags < HIPPIE_FLAG_CAPACITY {
            picked_any |= hippie_pickup_flags(hippie, flags);
        }

        if hippie.pos.distance(hippie.target) <= HIPPIE_TARGET_EPSILON {
            hippie.target = random_point_in_polygon(region_vertices, &mut hippie.rng_state);
        }

        let to_target = hippie.target - hippie.pos;
        if to_target.length_squared() > 0.0 {
            hippie.facing = player::facing_from_direction(to_target);
        }

        let step = hippie.speed * dt;
        let next_pos = if to_target.length() <= step || step <= 0.0 {
            hippie.target
        } else {
            hippie.pos + to_target.normalize() * step
        };

        if point_in_polygon(next_pos, region_vertices) {
            hippie.pos = next_pos;
        } else {
            hippie.target = random_point_in_polygon(region_vertices, &mut hippie.rng_state);
        }
    }
    picked_any
}

pub fn draw_hippies(hippies: &[Hippie]) {
    for hippie in hippies {
        draw_hippie(hippie.pos, hippie.facing, hippie.carried_flags);
    }
}

fn draw_hippie(pos: Vec2, facing: player::Facing, carried_flags: u8) {
    let head_center = vec2(pos.x, pos.y - HIPPIE_BODY_LENGTH * 0.5 - HIPPIE_HEAD_RADIUS);
    let body_top = vec2(pos.x, pos.y - HIPPIE_BODY_LENGTH * 0.5);
    let body_bottom = vec2(pos.x, pos.y + HIPPIE_BODY_LENGTH * 0.5);

    let skin = Color::new(0.95, 0.86, 0.74, 1.0);
    let body = Color::new(0.35, 0.7, 0.45, 1.0);
    let limbs = Color::new(0.2, 0.2, 0.2, 1.0);
    let outline = Color::new(0.05, 0.05, 0.05, 1.0);

    draw_circle(head_center.x, head_center.y, HIPPIE_HEAD_RADIUS + 1.0, outline);
    draw_circle(head_center.x, head_center.y, HIPPIE_HEAD_RADIUS, skin);

    draw_line(
        body_top.x,
        body_top.y,
        body_bottom.x,
        body_bottom.y,
        2.0 * scale::MODEL_SCALE,
        body,
    );

    let arm_offset = match facing {
        player::Facing::Left => vec2(-HIPPIE_ARM_LENGTH, 0.0),
        player::Facing::Right => vec2(HIPPIE_ARM_LENGTH, 0.0),
        _ => vec2(0.0, 0.0),
    };

    let arm_left = vec2(pos.x - HIPPIE_ARM_LENGTH * 0.6, pos.y - HIPPIE_BODY_LENGTH * 0.2);
    let arm_right = vec2(pos.x + HIPPIE_ARM_LENGTH * 0.6, pos.y - HIPPIE_BODY_LENGTH * 0.2);
    let left_hand = arm_left + vec2(-HIPPIE_ARM_LENGTH * 0.5, 0.0) + arm_offset;
    let right_hand = arm_right + vec2(HIPPIE_ARM_LENGTH * 0.5, 0.0) + arm_offset;

    draw_line(
        arm_left.x,
        arm_left.y,
        left_hand.x,
        left_hand.y,
        1.5 * scale::MODEL_SCALE,
        limbs,
    );
    draw_line(
        arm_right.x,
        arm_right.y,
        right_hand.x,
        right_hand.y,
        1.5 * scale::MODEL_SCALE,
        limbs,
    );
    draw_circle(left_hand.x, left_hand.y, HIPPIE_HAND_RADIUS, skin);
    draw_circle(right_hand.x, right_hand.y, HIPPIE_HAND_RADIUS, skin);

    let leg_offset = HIPPIE_LEG_LENGTH * 0.5;
    let left_foot = vec2(pos.x - leg_offset * 0.4, pos.y + HIPPIE_BODY_LENGTH * 0.5 + HIPPIE_LEG_LENGTH);
    let right_foot = vec2(pos.x + leg_offset * 0.4, pos.y + HIPPIE_BODY_LENGTH * 0.5 + HIPPIE_LEG_LENGTH);

    draw_line(
        body_bottom.x,
        body_bottom.y,
        left_foot.x,
        left_foot.y,
        1.5 * scale::MODEL_SCALE,
        limbs,
    );
    draw_line(
        body_bottom.x,
        body_bottom.y,
        right_foot.x,
        right_foot.y,
        1.5 * scale::MODEL_SCALE,
        limbs,
    );

    if carried_flags > 0 {
        draw_hand_flag(left_hand, facing);
    }
    if carried_flags > 1 {
        draw_hand_flag(right_hand, facing);
    }
}

fn draw_hand_flag(hand: Vec2, facing: player::Facing) {
    let (rotation, cloth_sign) = hippie_flag_orientation(facing);

    draw_rotated_rect(
        hand,
        vec2(constants::FLAG_POLE_WIDTH, constants::FLAG_POLE_HEIGHT),
        vec2(0.5, 1.0),
        rotation,
        Color::new(0.55, 0.44, 0.28, 1.0),
    );

    let pole_top = hand + rotate_vec(vec2(0.0, -constants::FLAG_POLE_HEIGHT), rotation);
    let cloth_anchor = pole_top
        + rotate_vec(vec2(cloth_sign * constants::FLAG_POLE_WIDTH * 0.5, 0.0), rotation);
    let cloth_offset = if cloth_sign < 0.0 { vec2(1.0, 0.0) } else { vec2(0.0, 0.0) };
    draw_rotated_rect(
        cloth_anchor,
        constants::FLAG_CLOTH_SIZE,
        cloth_offset,
        rotation,
        constants::ACCENT,
    );
}

fn hippie_flag_orientation(facing: player::Facing) -> (f32, f32) {
    match facing {
        player::Facing::Left => (-HIPPIE_FLAG_ANGLE, -1.0),
        player::Facing::Right => (
            HIPPIE_FLAG_ANGLE - std::f32::consts::FRAC_PI_2,
            1.0,
        ),
        _ => (-HIPPIE_FLAG_ANGLE, 1.0),
    }
}

fn draw_rotated_rect(center: Vec2, size: Vec2, offset: Vec2, rotation: f32, color: Color) {
    draw_rectangle_ex(
        center.x,
        center.y,
        size.x,
        size.y,
        DrawRectangleParams {
            offset,
            rotation,
            color,
        },
    );
}

fn rotate_vec(point: Vec2, angle: f32) -> Vec2 {
    vec2(
        point.x * angle.cos() - point.y * angle.sin(),
        point.x * angle.sin() + point.y * angle.cos(),
    )
}

fn hippie_pickup_flags(hippie: &mut Hippie, flags: &mut Vec<flags::Flag>) -> bool {
    let mut picked = false;
    let mut index = 0;
    while index < flags.len() && hippie.carried_flags < HIPPIE_FLAG_CAPACITY {
        let flag_pos = flags[index].pos;
        if flag_pos.distance(hippie.pos) <= HIPPIE_FLAG_PICKUP_RADIUS {
            flags.swap_remove(index);
            hippie.carried_flags += 1;
            picked = true;
            continue;
        }
        index += 1;
    }
    picked
}

fn nearest_hippie_with_flag(
    hippies: &[Hippie],
    origin: Vec2,
    radius: f32,
) -> Option<usize> {
    let mut best = None;
    let mut best_d2 = radius * radius;
    for (i, hippie) in hippies.iter().enumerate() {
        if hippie.carried_flags == 0 {
            continue;
        }
        let d2 = hippie.pos.distance_squared(origin);
        if d2 <= best_d2 {
            best = Some(i);
            best_d2 = d2;
        }
    }
    best
}

fn point_in_polygon(point: Vec2, vertices: &[Vec2]) -> bool {
    if vertices.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = vertices.len() - 1;
    for i in 0..vertices.len() {
        let vi = vertices[i];
        let vj = vertices[j];
        let intersects = (vi.y > point.y) != (vj.y > point.y)
            && point.x
                < (vj.x - vi.x) * (point.y - vi.y) / (vj.y - vi.y + f32::EPSILON) + vi.x;
        if intersects {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn random_point_in_polygon(vertices: &[Vec2], rng_state: &mut u32) -> Vec2 {
    if vertices.is_empty() {
        return Vec2::ZERO;
    }
    let (min, max) = polygon_bounds(vertices);
    for _ in 0..HIPPIE_BOUNDS_ATTEMPTS {
        let x = lerp(min.x, max.x, next_f32(rng_state));
        let y = lerp(min.y, max.y, next_f32(rng_state));
        let candidate = vec2(x, y);
        if point_in_polygon(candidate, vertices) {
            return candidate;
        }
    }
    vertices[0]
}

fn polygon_bounds(vertices: &[Vec2]) -> (Vec2, Vec2) {
    let mut min = vertices[0];
    let mut max = vertices[0];
    for v in &vertices[1..] {
        min.x = min.x.min(v.x);
        min.y = min.y.min(v.y);
        max.x = max.x.max(v.x);
        max.y = max.y.max(v.y);
    }
    (min, max)
}

fn next_f32(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    let v = (*state >> 8) as f32;
    v / ((u32::MAX >> 8) as f32 + 1.0)
}

fn hash_seed(pos: Vec2, index: u32) -> u32 {
    let x = (pos.x * 10.0).to_bits();
    let y = (pos.y * 10.0).to_bits();
    x ^ y ^ index.rotate_left(13)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_in_polygon_detects_inside() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(10.0, 0.0),
            vec2(10.0, 10.0),
            vec2(0.0, 10.0),
        ];
        assert!(point_in_polygon(vec2(5.0, 5.0), &square));
        assert!(!point_in_polygon(vec2(12.0, 5.0), &square));
    }

    #[test]
    fn random_point_in_polygon_stays_inside() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(10.0, 0.0),
            vec2(10.0, 10.0),
            vec2(0.0, 10.0),
        ];
        let mut rng = 1u32;
        for _ in 0..32 {
            let p = random_point_in_polygon(&square, &mut rng);
            assert!(point_in_polygon(p, &square));
        }
    }

    #[test]
    fn hippie_update_keeps_inside_region() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_hippies(&[vec2(10.0, 10.0)], &square);
        let mut flags = Vec::new();
        for _ in 0..60 {
            update_hippies(&mut hippies, 0.1, &square, &mut flags);
            assert!(point_in_polygon(hippies[0].pos, &square));
        }
    }

    #[test]
    fn hippie_picks_up_flags_until_full() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_hippies(&[vec2(10.0, 10.0)], &square);
        let mut flags = vec![
            flags::Flag { pos: vec2(10.0, 11.0), phase: 0.0 },
            flags::Flag { pos: vec2(9.0, 10.0), phase: 0.0 },
            flags::Flag { pos: vec2(12.0, 10.0), phase: 0.0 },
        ];
        let picked = update_hippies(&mut hippies, 0.0, &square, &mut flags);
        assert!(picked);
        assert_eq!(hippies[0].carried_flags, 2);
        assert_eq!(flags.len(), 1);
    }

    #[test]
    fn hippie_does_not_pick_up_when_full() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_hippies(&[vec2(10.0, 10.0)], &square);
        hippies[0].carried_flags = HIPPIE_FLAG_CAPACITY;
        let mut flags = vec![flags::Flag { pos: vec2(10.0, 10.0), phase: 0.0 }];
        let picked = update_hippies(&mut hippies, 0.0, &square, &mut flags);
        assert!(!picked);
        assert_eq!(flags.len(), 1);
    }

    #[test]
    fn spawn_hippies_with_flags_clamps_capacity() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let hippies = spawn_hippies_with_flags(&[(vec2(5.0, 5.0), 5)], &square);
        assert_eq!(hippies[0].carried_flags, HIPPIE_FLAG_CAPACITY);
    }

    #[test]
    fn hippie_flag_cloth_anchor_is_at_pole_top() {
        let hand = vec2(0.0, 0.0);
        let (rotation, cloth_sign) = hippie_flag_orientation(player::Facing::Right);
        let pole_top = hand + rotate_vec(vec2(0.0, -constants::FLAG_POLE_HEIGHT), rotation);
        let cloth_anchor = pole_top
            + rotate_vec(vec2(cloth_sign * constants::FLAG_POLE_WIDTH * 0.5, 0.0), rotation);
        let distance = cloth_anchor.distance(pole_top);
        assert!((distance - constants::FLAG_POLE_WIDTH * 0.5).abs() < 1e-4);
    }

    #[test]
    fn hippie_flag_right_facing_rotates_clockwise() {
        let (rotation, _) = hippie_flag_orientation(player::Facing::Right);
        let expected = HIPPIE_FLAG_ANGLE - std::f32::consts::FRAC_PI_2;
        assert!((rotation - expected).abs() < 1e-6);
    }

    #[test]
    fn steal_flag_takes_from_nearest_hippie() {
        let mut hippies = vec![
            Hippie {
                pos: vec2(0.0, 0.0),
                facing: player::Facing::Down,
                carried_flags: 1,
                target: vec2(0.0, 0.0),
                speed: HIPPIE_SPEED,
                rng_state: 1,
            },
            Hippie {
                pos: vec2(3.0, 0.0),
                facing: player::Facing::Down,
                carried_flags: 2,
                target: vec2(0.0, 0.0),
                speed: HIPPIE_SPEED,
                rng_state: 2,
            },
        ];

        let stolen = try_steal_flag(&mut hippies, vec2(2.5, 0.0), 4.0);
        assert!(stolen);
        assert_eq!(hippies[1].carried_flags, 1);
        assert_eq!(hippies[0].carried_flags, 1);
    }

    #[test]
    fn steal_flag_fails_without_flags() {
        let mut hippies = vec![Hippie {
            pos: vec2(0.0, 0.0),
            facing: player::Facing::Down,
            carried_flags: 0,
            target: vec2(0.0, 0.0),
            speed: HIPPIE_SPEED,
            rng_state: 1,
        }];

        let stolen = try_steal_flag(&mut hippies, vec2(0.0, 0.0), 4.0);
        assert!(!stolen);
    }
}
