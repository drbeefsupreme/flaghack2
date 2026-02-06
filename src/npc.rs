use macroquad::prelude::*;

use crate::constants;
use crate::flags;
use crate::geom;
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
const HIPPIE_ANGER_COLOR_SPEED: f32 = 2.0;

#[derive(Clone, Debug)]
pub struct Hippie {
    pub pos: Vec2,
    pub facing: player::Facing,
    pub carried_flags: u8,
    pub angry: bool,
    pub anger_timer: f32,
    pub anger_delay: f32,
    pub steal_cooldown: f32,
    pub flee_timer: f32,
    pub drop_check_timer: f32,
    pub ignore_flags_timer: f32,
    target: Vec2,
    speed: f32,
    rng_state: u32,
}

pub fn try_steal_flag(hippies: &mut [Hippie], origin: Vec2, radius: f32) -> bool {
    if let Some(index) = nearest_hippie_with_flag(hippies, origin, radius) {
        hippies[index].carried_flags = hippies[index].carried_flags.saturating_sub(1);
        hippies[index].angry = true;
        hippies[index].anger_timer = constants::HIPPIE_ANGER_DURATION;
        hippies[index].anger_delay = constants::HIPPIE_ANGER_DELAY;
        hippies[index].steal_cooldown = 0.0;
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
                angry: false,
                anger_timer: 0.0,
                anger_delay: 0.0,
                steal_cooldown: 0.0,
                flee_timer: 0.0,
                drop_check_timer: next_f32(&mut rng_state) * constants::HIPPIE_FLAG_DROP_INTERVAL,
                ignore_flags_timer: 0.0,
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
                angry: false,
                anger_timer: 0.0,
                anger_delay: 0.0,
                steal_cooldown: 0.0,
                flee_timer: 0.0,
                drop_check_timer: next_f32(&mut rng_state) * constants::HIPPIE_FLAG_DROP_INTERVAL,
                ignore_flags_timer: 0.0,
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
    player_pos: Vec2,
    player_inventory: &mut u32,
    player_speed: f32,
) -> bool {
    let mut picked_any = false;
    for hippie in hippies {
        update_hippie_drop(hippie, dt, flags);

        if hippie.ignore_flags_timer <= 0.0 && hippie.carried_flags < HIPPIE_FLAG_CAPACITY {
            picked_any |= hippie_pickup_flags(hippie, flags);
        }

        update_hippie_anger(hippie, player_pos, dt);
        update_hippie_flee(hippie, dt);
        let angry = hippie.angry;

        if angry && hippie.anger_delay <= 0.0 {
            steal_from_player(hippie, player_pos, player_inventory, flags, dt);
        }

        if !angry
            && hippie.flee_timer <= 0.0
            && hippie.pos.distance(hippie.target) <= HIPPIE_TARGET_EPSILON
        {
            hippie.target = random_point_in_polygon(region_vertices, &mut hippie.rng_state);
        }

        let target = if angry {
            player_pos
        } else if hippie.flee_timer > 0.0 {
            hippie.pos + flee_direction(hippie.pos, player_pos) * 80.0 * scale::MODEL_SCALE
        } else {
            hippie.target
        };
        let to_target = target - hippie.pos;
        if to_target.length_squared() > 0.0 {
            hippie.facing = player::facing_from_direction(to_target);
        }

        let speed = if angry {
            chase_speed(player_speed)
        } else {
            hippie.speed
        };
        let step = speed * dt;
        let next_pos = if to_target.length() <= step || step <= 0.0 {
            hippie.target
        } else {
            hippie.pos + to_target.normalize() * step
        };

        if geom::point_in_polygon(next_pos, region_vertices) {
            hippie.pos = next_pos;
        } else if !angry && hippie.flee_timer <= 0.0 {
            hippie.target = random_point_in_polygon(region_vertices, &mut hippie.rng_state);
        }
    }
    picked_any
}

pub fn draw_hippies(hippies: &[Hippie]) {
    for hippie in hippies {
        draw_hippie(hippie.pos, hippie.facing, hippie.carried_flags, hippie.angry);
    }
}

fn draw_hippie(pos: Vec2, facing: player::Facing, carried_flags: u8, angry: bool) {
    let head_center = vec2(pos.x, pos.y - HIPPIE_BODY_LENGTH * 0.5 - HIPPIE_HEAD_RADIUS);
    let body_top = vec2(pos.x, pos.y - HIPPIE_BODY_LENGTH * 0.5);
    let body_bottom = vec2(pos.x, pos.y + HIPPIE_BODY_LENGTH * 0.5);

    let skin = if angry {
        angry_head_color(get_time() as f32)
    } else {
        Color::new(0.95, 0.86, 0.74, 1.0)
    };
    let body = Color::new(0.35, 0.7, 0.45, 1.0);
    let limbs = Color::new(0.2, 0.2, 0.2, 1.0);
    let outline = Color::new(0.05, 0.05, 0.05, 1.0);

    if angry {
        let glow = angry_head_color(get_time() as f32);
        draw_circle(
            head_center.x,
            head_center.y,
            HIPPIE_HEAD_RADIUS * 1.8,
            Color::new(glow.r, glow.g, glow.b, 0.6),
        );
    }
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

fn update_hippie_drop(hippie: &mut Hippie, dt: f32, flags: &mut Vec<flags::Flag>) {
    if hippie.ignore_flags_timer > 0.0 {
        hippie.ignore_flags_timer = (hippie.ignore_flags_timer - dt).max(0.0);
    }

    hippie.drop_check_timer -= dt;
    while hippie.drop_check_timer <= 0.0 {
        hippie.drop_check_timer += constants::HIPPIE_FLAG_DROP_INTERVAL;
        if hippie.carried_flags == 0 {
            continue;
        }
        let roll = next_f32(&mut hippie.rng_state);
        if roll <= constants::HIPPIE_FLAG_DROP_CHANCE {
            hippie.carried_flags = hippie.carried_flags.saturating_sub(1);
            flags.push(flags::make_flag(hippie.pos));
            hippie.ignore_flags_timer = constants::HIPPIE_FLAG_IGNORE_DURATION;
        }
    }
}

fn steal_from_player(
    hippie: &mut Hippie,
    player_pos: Vec2,
    player_inventory: &mut u32,
    flags: &mut Vec<flags::Flag>,
    dt: f32,
) {
    if hippie.steal_cooldown > 0.0 {
        hippie.steal_cooldown = (hippie.steal_cooldown - dt).max(0.0);
    }

    if hippie.steal_cooldown > 0.0 {
        return;
    }

    if hippie.pos.distance(player_pos) > constants::HIPPIE_STEAL_BACK_RADIUS {
        return;
    }

    let stolen = if *player_inventory >= 2 {
        2
    } else if *player_inventory >= 1 {
        1
    } else {
        0
    };

    if stolen > 0 {
        *player_inventory -= stolen;
        let mut remaining = stolen as u8;
        while remaining > 0 {
            if hippie.carried_flags < HIPPIE_FLAG_CAPACITY {
                hippie.carried_flags += 1;
            } else {
                flags.push(flags::make_flag(hippie.pos));
            }
            remaining -= 1;
        }

        hippie.steal_cooldown = constants::HIPPIE_STEAL_COOLDOWN;
        hippie.angry = false;
        hippie.anger_timer = 0.0;
        hippie.anger_delay = 0.0;
        hippie.flee_timer = constants::HIPPIE_FLEE_DURATION;
    }
}

fn update_hippie_anger(hippie: &mut Hippie, player_pos: Vec2, dt: f32) {
    if !hippie.angry {
        return;
    }

    if hippie.anger_delay > 0.0 {
        hippie.anger_delay = (hippie.anger_delay - dt).max(0.0);
    }

    if hippie.anger_timer > 0.0 {
        hippie.anger_timer = (hippie.anger_timer - dt).max(0.0);
    }

    if hippie.anger_timer <= 0.0 {
        let close = hippie.pos.distance(player_pos) <= constants::HIPPIE_ANGER_RADIUS;
        if !close {
            hippie.angry = false;
        }
    }
}

fn update_hippie_flee(hippie: &mut Hippie, dt: f32) {
    if hippie.flee_timer > 0.0 {
        hippie.flee_timer = (hippie.flee_timer - dt).max(0.0);
    }
}

fn flee_direction(hippie_pos: Vec2, player_pos: Vec2) -> Vec2 {
    let dir = hippie_pos - player_pos;
    if dir.length_squared() <= f32::EPSILON {
        vec2(1.0, 0.0)
    } else {
        dir.normalize()
    }
}

fn angry_head_color(time: f32) -> Color {
    let t = 0.5 + 0.5 * (time * HIPPIE_ANGER_COLOR_SPEED).sin();
    let red = Color::new(1.0, 0.1, 0.05, 1.0);
    let orange = Color::new(1.0, 0.55, 0.0, 1.0);
    Color::new(
        red.r + (orange.r - red.r) * t,
        red.g + (orange.g - red.g) * t,
        red.b + (orange.b - red.b) * t,
        1.0,
    )
}

fn chase_speed(player_speed: f32) -> f32 {
    player_speed * constants::HIPPIE_CHASE_SPEED_FACTOR
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

fn random_point_in_polygon(vertices: &[Vec2], rng_state: &mut u32) -> Vec2 {
    let Some((min, max)) = geom::polygon_bounds(vertices) else {
        return Vec2::ZERO;
    };
    for _ in 0..HIPPIE_BOUNDS_ATTEMPTS {
        let x = lerp(min.x, max.x, next_f32(rng_state));
        let y = lerp(min.y, max.y, next_f32(rng_state));
        let candidate = vec2(x, y);
        if geom::point_in_polygon(candidate, vertices) {
            return candidate;
        }
    }
    vertices[0]
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
        assert!(geom::point_in_polygon(vec2(5.0, 5.0), &square));
        assert!(!geom::point_in_polygon(vec2(12.0, 5.0), &square));
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
            assert!(geom::point_in_polygon(p, &square));
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
            update_hippies(
                &mut hippies,
                0.1,
                &square,
                &mut flags,
                vec2(50.0, 50.0),
                &mut 0,
                100.0,
            );
            assert!(geom::point_in_polygon(hippies[0].pos, &square));
        }
    }

    #[test]
    fn hippie_drop_sets_ignore_and_spawns_flag() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_hippies_with_flags(&[(vec2(5.0, 5.0), 1)], &square);
        hippies[0].drop_check_timer = 0.0;
        hippies[0].rng_state = 0;
        let mut flags = Vec::new();
        update_hippie_drop(&mut hippies[0], 0.1, &mut flags);
        assert_eq!(hippies[0].carried_flags, 0);
        assert_eq!(flags.len(), 1);
        assert!((hippies[0].ignore_flags_timer - constants::HIPPIE_FLAG_IGNORE_DURATION).abs() < 1e-6);
    }

    #[test]
    fn hippie_ignores_pickup_during_cooldown() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_hippies_with_flags(&[(vec2(5.0, 5.0), 0)], &square);
        hippies[0].ignore_flags_timer = constants::HIPPIE_FLAG_IGNORE_DURATION;
        let mut flags = vec![flags::Flag { pos: vec2(5.0, 5.0), phase: 0.0 }];
        update_hippies(
            &mut hippies,
            0.0,
            &square,
            &mut flags,
            vec2(0.0, 0.0),
            &mut 0,
            100.0,
        );
        assert_eq!(flags.len(), 1);
        assert_eq!(hippies[0].carried_flags, 0);
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
        let mut inventory = 0;
        let picked = update_hippies(
            &mut hippies,
            0.0,
            &square,
            &mut flags,
            vec2(0.0, 0.0),
            &mut inventory,
            100.0,
        );
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
        let mut inventory = 0;
        let picked = update_hippies(
            &mut hippies,
            0.0,
            &square,
            &mut flags,
            vec2(0.0, 0.0),
            &mut inventory,
            100.0,
        );
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
                angry: false,
                anger_timer: 0.0,
                anger_delay: 0.0,
                steal_cooldown: 0.0,
                flee_timer: 0.0,
                drop_check_timer: 0.0,
                ignore_flags_timer: 0.0,
                target: vec2(0.0, 0.0),
                speed: HIPPIE_SPEED,
                rng_state: 1,
            },
            Hippie {
                pos: vec2(3.0, 0.0),
                facing: player::Facing::Down,
                carried_flags: 2,
                angry: false,
                anger_timer: 0.0,
                anger_delay: 0.0,
                steal_cooldown: 0.0,
                flee_timer: 0.0,
                drop_check_timer: 0.0,
                ignore_flags_timer: 0.0,
                target: vec2(0.0, 0.0),
                speed: HIPPIE_SPEED,
                rng_state: 2,
            },
        ];

        let stolen = try_steal_flag(&mut hippies, vec2(2.5, 0.0), 4.0);
        assert!(stolen);
        assert_eq!(hippies[1].carried_flags, 1);
        assert_eq!(hippies[0].carried_flags, 1);
        assert!(hippies[1].angry);
        assert!((hippies[1].anger_timer - constants::HIPPIE_ANGER_DURATION).abs() < 1e-6);
    }

    #[test]
    fn steal_flag_fails_without_flags() {
        let mut hippies = vec![Hippie {
            pos: vec2(0.0, 0.0),
            facing: player::Facing::Down,
            carried_flags: 0,
            angry: false,
            anger_timer: 0.0,
            anger_delay: 0.0,
            steal_cooldown: 0.0,
            flee_timer: 0.0,
            drop_check_timer: 0.0,
            ignore_flags_timer: 0.0,
            target: vec2(0.0, 0.0),
            speed: HIPPIE_SPEED,
            rng_state: 1,
        }];

        let stolen = try_steal_flag(&mut hippies, vec2(0.0, 0.0), 4.0);
        assert!(!stolen);
    }

    #[test]
    fn anger_clears_when_timer_elapsed_and_far() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_hippies(&[vec2(5.0, 5.0)], &square);
        hippies[0].angry = true;
        hippies[0].anger_timer = 0.0;
        hippies[0].flee_timer = 0.0;
        hippies[0].anger_delay = 0.0;
        hippies[0].drop_check_timer = 0.0;
        hippies[0].ignore_flags_timer = 0.0;
        let mut flags = Vec::new();
        update_hippies(
            &mut hippies,
            0.1,
            &square,
            &mut flags,
            vec2(100.0, 100.0),
            &mut 0,
            100.0,
        );
        assert!(!hippies[0].angry);
    }

    #[test]
    fn anger_persists_while_close() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_hippies(&[vec2(5.0, 5.0)], &square);
        hippies[0].angry = true;
        hippies[0].anger_timer = 0.0;
        hippies[0].flee_timer = 0.0;
        hippies[0].anger_delay = 0.0;
        hippies[0].drop_check_timer = 0.0;
        hippies[0].ignore_flags_timer = 0.0;
        let mut flags = Vec::new();
        update_hippies(
            &mut hippies,
            0.1,
            &square,
            &mut flags,
            vec2(6.0, 6.0),
            &mut 0,
            100.0,
        );
        assert!(hippies[0].angry);
    }

    #[test]
    fn angry_color_cycles_over_time() {
        let a = angry_head_color(0.0);
        let b = angry_head_color(0.5);
        let delta = (a.g - b.g).abs() + (a.b - b.b).abs();
        assert!(delta > 1e-3);
    }

    #[test]
    fn hippie_steals_from_player_when_close_and_angry() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_hippies(&[vec2(5.0, 5.0)], &square);
        hippies[0].angry = true;
        hippies[0].anger_timer = constants::HIPPIE_ANGER_DURATION;
        hippies[0].flee_timer = 0.0;
        hippies[0].anger_delay = 0.0;
        hippies[0].drop_check_timer = 0.0;
        hippies[0].ignore_flags_timer = 0.0;
        let mut flags = Vec::new();
        let mut inventory = 3;
        let total_before =
            inventory + flags.len() as u32 + hippies.iter().map(|h| h.carried_flags as u32).sum::<u32>();
        update_hippies(
            &mut hippies,
            0.0,
            &square,
            &mut flags,
            vec2(5.0, 5.0),
            &mut inventory,
            100.0,
        );
        assert_eq!(inventory, 1);
        assert_eq!(hippies[0].carried_flags, 2);
        let total_after =
            inventory + flags.len() as u32 + hippies.iter().map(|h| h.carried_flags as u32).sum::<u32>();
        assert_eq!(total_before, total_after);
        assert!(hippies[0].steal_cooldown > 0.0);
        assert!(!hippies[0].angry);
        assert!((hippies[0].flee_timer - constants::HIPPIE_FLEE_DURATION).abs() < 1e-6);
    }

    #[test]
    fn hippie_steal_drops_excess_when_full() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_hippies_with_flags(&[(vec2(5.0, 5.0), 2)], &square);
        hippies[0].angry = true;
        hippies[0].anger_timer = constants::HIPPIE_ANGER_DURATION;
        let mut flags = Vec::new();
        let mut inventory = 2;
        let total_before =
            inventory + flags.len() as u32 + hippies.iter().map(|h| h.carried_flags as u32).sum::<u32>();
        update_hippies(
            &mut hippies,
            0.0,
            &square,
            &mut flags,
            vec2(5.0, 5.0),
            &mut inventory,
            100.0,
        );
        assert_eq!(inventory, 0);
        assert_eq!(hippies[0].carried_flags, 2);
        assert_eq!(flags.len(), 2);
        let total_after =
            inventory + flags.len() as u32 + hippies.iter().map(|h| h.carried_flags as u32).sum::<u32>();
        assert_eq!(total_before, total_after);
    }

    #[test]
    fn hippie_steal_respects_cooldown() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_hippies(&[vec2(5.0, 5.0)], &square);
        hippies[0].angry = true;
        hippies[0].anger_timer = constants::HIPPIE_ANGER_DURATION;
        hippies[0].flee_timer = 0.0;
        hippies[0].anger_delay = 0.0;
        hippies[0].drop_check_timer = 0.0;
        hippies[0].ignore_flags_timer = 0.0;
        let mut flags = Vec::new();
        let mut inventory = 2;
        update_hippies(
            &mut hippies,
            0.0,
            &square,
            &mut flags,
            vec2(5.0, 5.0),
            &mut inventory,
            100.0,
        );
        let after_first = inventory;
        update_hippies(
            &mut hippies,
            0.0,
            &square,
            &mut flags,
            vec2(5.0, 5.0),
            &mut inventory,
            100.0,
        );
        assert_eq!(inventory, after_first);
    }

    #[test]
    fn chase_speed_uses_player_speed_factor() {
        let speed = chase_speed(100.0);
        assert!((speed - 100.0 * constants::HIPPIE_CHASE_SPEED_FACTOR).abs() < 1e-6);
    }
}
