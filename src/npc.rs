use macroquad::prelude::*;

use crate::constants;
use crate::flag_state;
use crate::flags;
use crate::geom;
use crate::player;
use crate::scale;
use crate::GameMode;

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
    pub camp_index: usize,
    target: Vec2,
    speed: f32,
    rng_state: u32,
    pub flag_psychosis: f32,
    pub drunkenness: f32,
    pub dirtiness: f32,
}

pub fn try_steal_flag(
    hippies: &mut [Hippie],
    origin: Vec2,
    radius: f32,
    flag_state: &mut flag_state::FlagState,
) -> bool {
    if let Some(index) = nearest_hippie_with_flag(hippies, origin, radius) {
        if !flag_state.steal_from_hippie(&mut hippies[index].carried_flags) {
            return false;
        }
        hippies[index].angry = true;
        hippies[index].anger_timer = constants::HIPPIE_ANGER_DURATION;
        hippies[index].anger_delay = constants::HIPPIE_ANGER_DELAY;
        hippies[index].steal_cooldown = 0.0;
        return true;
    }
    false
}

pub fn spawn_hippies(positions: &[Vec2], camp_index: usize, camp_vertices: &[Vec2]) -> Vec<Hippie> {
    positions
        .iter()
        .enumerate()
        .map(|(i, &pos)| {
            let mut rng_state = hash_seed(pos, i as u32);
            let carried_flags = initial_carried_flags(&mut rng_state);
            let target = random_point_in_polygon(camp_vertices, &mut rng_state);
            Hippie {
                pos,
                facing: player::Facing::Down,
                carried_flags,
                angry: false,
                anger_timer: 0.0,
                anger_delay: 0.0,
                steal_cooldown: 0.0,
                flee_timer: 0.0,
                drop_check_timer: next_f32(&mut rng_state) * constants::HIPPIE_FLAG_DROP_INTERVAL,
                ignore_flags_timer: 0.0,
                camp_index,
                target,
                speed: HIPPIE_SPEED,
                rng_state,
                flag_psychosis: 0.0,
                drunkenness: 0.0,
                dirtiness: 0.0,
            }
        })
        .collect()
}

pub fn spawn_hippies_with_flags(
    spawns: &[(Vec2, u8)],
    camp_index: usize,
    camp_vertices: &[Vec2],
) -> Vec<Hippie> {
    spawns
        .iter()
        .enumerate()
        .map(|(i, &(pos, carried))| {
            let mut rng_state = hash_seed(pos, i as u32);
            let target = random_point_in_polygon(camp_vertices, &mut rng_state);
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
                camp_index,
                target,
                speed: HIPPIE_SPEED,
                rng_state,
                flag_psychosis: 0.0,
                drunkenness: 0.0,
                dirtiness: 0.0,
            }
        })
        .collect()
}

pub fn update_hippies(
    hippies: &mut [Hippie],
    dt: f32,
    camp_vertices: &[Vec<Vec2>],
    flag_state: &mut flag_state::FlagState,
    player_pos: Vec2,
    player_speed: f32,
    mode: GameMode,
    pentagram_centers: &[Vec2],
    pentagram_blueprints: &[Vec<Vec2>],
    time: f32,
) -> bool {
    let mut picked_any = false;
    let player_has_flags = flag_state.player_inventory() > 0;
    let mut desired_positions = Vec::with_capacity(hippies.len());
    let mut inside_camps = Vec::with_capacity(hippies.len());
    let mut angry_flags = Vec::with_capacity(hippies.len());
    let mut pentagram_slots: Vec<Option<Vec2>> = Vec::with_capacity(hippies.len());
    for hippie in hippies.iter_mut() {
        let camp = camp_for_index(camp_vertices, hippie.camp_index);
        let inside_camp = geom::point_in_polygon(hippie.pos, camp);
        update_hippie_drop(hippie, dt, flag_state);

        if hippie.ignore_flags_timer <= 0.0 && hippie.carried_flags < HIPPIE_FLAG_CAPACITY {
            let pickup_radius = if mode == GameMode::Burn {
                drunk_vision_range(HIPPIE_FLAG_PICKUP_RADIUS, hippie.drunkenness)
            } else {
                HIPPIE_FLAG_PICKUP_RADIUS
            };
            picked_any |= flag_state.transfer_ground_to_hippie(
                &mut hippie.carried_flags,
                HIPPIE_FLAG_CAPACITY,
                hippie.pos,
                pickup_radius,
            );
        }

        // In Burn mode: psychosis-driven behavior instead of anger
        let chasing_flag = if mode == GameMode::Burn {
            update_hippie_psychosis(hippie, pentagram_centers, dt);

            // Psychosis decay when entering camp with flags
            if inside_camp && hippie.carried_flags > 0 {
                hippie.flag_psychosis =
                    (hippie.flag_psychosis - constants::PSYCHOSIS_CAMP_RETURN_DECAY * dt).max(0.0);
            }

            // Priority 1: Build pentagram when in camp with flags
            let blueprint = camp_blueprint(pentagram_blueprints, hippie.camp_index);
            if inside_camp && hippie.carried_flags > 0 && !blueprint.is_empty() {
                if let Some(slot) =
                    find_empty_pentagram_slot(blueprint, flag_state.ground_flags(), hippie.pos)
                {
                    hippie.target = slot;
                    pentagram_slots.push(Some(slot));
                    true
                } else if let Some(flag_target) =
                    psychosis_target(hippie, flag_state.ground_flags(), camp)
                {
                    hippie.target = flag_target;
                    pentagram_slots.push(None);
                    true
                } else {
                    pentagram_slots.push(None);
                    false
                }
            } else {
                // Not in camp or no flags - chase outside flags
                if let Some(flag_target) =
                    psychosis_target(hippie, flag_state.ground_flags(), camp)
                {
                    hippie.target = flag_target;
                    pentagram_slots.push(None);
                    true
                } else {
                    pentagram_slots.push(None);
                    false
                }
            }
        } else {
            update_hippie_anger(hippie, player_pos, player_has_flags, dt);
            pentagram_slots.push(None);
            false
        };

        update_hippie_flee(hippie, dt);
        let angry = if mode == GameMode::Burn {
            false
        } else {
            hippie.angry
        };

        if angry && hippie.anger_delay <= 0.0 {
            steal_from_player(hippie, player_pos, flag_state, dt);
        }

        let pursuing = angry || chasing_flag;

        if !pursuing && hippie.flee_timer <= 0.0 {
            if inside_camp {
                if hippie.pos.distance(hippie.target) <= HIPPIE_TARGET_EPSILON {
                    hippie.target = random_point_in_polygon(camp, &mut hippie.rng_state);
                }
            } else if !geom::point_in_polygon(hippie.target, camp) {
                hippie.target = random_point_in_polygon(camp, &mut hippie.rng_state);
            }
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

        let base_speed = if angry {
            chase_speed(player_speed)
        } else if chasing_flag {
            hippie.speed * constants::PSYCHOSIS_SPEED_BOOST
        } else {
            hippie.speed
        };
        let speed = if mode == GameMode::Burn {
            drunk_speed(base_speed, hippie.drunkenness)
        } else {
            base_speed
        };
        let step = speed * dt;
        let mut next_pos = if to_target.length() <= step || step <= 0.0 {
            target
        } else {
            hippie.pos + to_target.normalize() * step
        };

        // Apply wobble in Burn mode
        if mode == GameMode::Burn && hippie.drunkenness > 0.01 && to_target.length_squared() > 0.0 {
            let wobble = drunk_wobble_offset(
                to_target.normalize_or_zero(),
                time,
                hippie.drunkenness,
                hippie.rng_state as f32 * 0.01,
            );
            next_pos += wobble * dt;
        }

        let desired = if pursuing || !inside_camp {
            next_pos
        } else if geom::point_in_polygon(next_pos, camp) {
            next_pos
        } else {
            if hippie.flee_timer <= 0.0 {
                hippie.target = random_point_in_polygon(camp, &mut hippie.rng_state);
            }
            hippie.pos
        };

        desired_positions.push(desired);
        inside_camps.push(inside_camp);
        angry_flags.push(pursuing);
    }

    resolve_hippie_collisions(
        &mut desired_positions,
        constants::HIPPIE_COLLISION_RADIUS * 2.0,
    );

    for (idx, hippie) in hippies.iter_mut().enumerate() {
        let camp = camp_for_index(camp_vertices, hippie.camp_index);
        let desired = desired_positions[idx];
        let inside_camp = inside_camps[idx];
        let angry = angry_flags[idx];
        if !angry && inside_camp && !geom::point_in_polygon(desired, camp) {
            continue;
        }
        hippie.pos = desired;
    }

    // Drop flags at pentagram blueprint slots
    for (idx, hippie) in hippies.iter_mut().enumerate() {
        if let Some(slot) = pentagram_slots[idx] {
            if hippie.pos.distance(slot) <= constants::PENTAGRAM_DROP_RADIUS
                && hippie.carried_flags > 0
            {
                flag_state.drop_from_hippie(&mut hippie.carried_flags, 1, slot);
                picked_any = true;
            }
        }
    }

    picked_any
}

pub fn draw_hippies(hippies: &[Hippie], time: f32) {
    for hippie in hippies {
        draw_hippie(
            hippie.pos,
            hippie.facing,
            hippie.carried_flags,
            hippie.angry,
            hippie.dirtiness,
        );
        draw_dust_trail(hippie.pos, hippie.dirtiness, time);
        draw_stink_lines(hippie.pos, hippie.dirtiness, time);
        draw_psychosis_sparkles(hippie.pos, hippie.flag_psychosis, time, hippie.rng_state);
    }
}

fn draw_hippie(pos: Vec2, facing: player::Facing, carried_flags: u8, angry: bool, dirtiness: f32) {
    let head_center = vec2(pos.x, pos.y - HIPPIE_BODY_LENGTH * 0.5 - HIPPIE_HEAD_RADIUS);
    let body_top = vec2(pos.x, pos.y - HIPPIE_BODY_LENGTH * 0.5);
    let body_bottom = vec2(pos.x, pos.y + HIPPIE_BODY_LENGTH * 0.5);

    let skin = if angry {
        angry_head_color(get_time() as f32)
    } else {
        dirty_color_shift(Color::new(0.95, 0.86, 0.74, 1.0), dirtiness)
    };
    let body = dirty_color_shift(Color::new(0.35, 0.7, 0.45, 1.0), dirtiness);
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
    draw_circle(
        head_center.x,
        head_center.y,
        HIPPIE_HEAD_RADIUS + 1.0,
        outline,
    );
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

    let arm_left = vec2(
        pos.x - HIPPIE_ARM_LENGTH * 0.6,
        pos.y - HIPPIE_BODY_LENGTH * 0.2,
    );
    let arm_right = vec2(
        pos.x + HIPPIE_ARM_LENGTH * 0.6,
        pos.y - HIPPIE_BODY_LENGTH * 0.2,
    );
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
    let left_foot = vec2(
        pos.x - leg_offset * 0.4,
        pos.y + HIPPIE_BODY_LENGTH * 0.5 + HIPPIE_LEG_LENGTH,
    );
    let right_foot = vec2(
        pos.x + leg_offset * 0.4,
        pos.y + HIPPIE_BODY_LENGTH * 0.5 + HIPPIE_LEG_LENGTH,
    );

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
        + rotate_vec(
            vec2(cloth_sign * constants::FLAG_POLE_WIDTH * 0.5, 0.0),
            rotation,
        );
    let cloth_offset = if cloth_sign < 0.0 {
        vec2(1.0, 0.0)
    } else {
        vec2(0.0, 0.0)
    };
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
        player::Facing::Right => (HIPPIE_FLAG_ANGLE - std::f32::consts::FRAC_PI_2, 1.0),
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

fn update_hippie_drop(hippie: &mut Hippie, dt: f32, flag_state: &mut flag_state::FlagState) {
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
            flag_state.drop_from_hippie(&mut hippie.carried_flags, 1, hippie.pos);
            hippie.ignore_flags_timer = constants::HIPPIE_FLAG_IGNORE_DURATION;
        }
    }
}

fn steal_from_player(
    hippie: &mut Hippie,
    player_pos: Vec2,
    flag_state: &mut flag_state::FlagState,
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

    let stolen = flag_state.steal_from_player_to_hippie(
        &mut hippie.carried_flags,
        HIPPIE_FLAG_CAPACITY,
        hippie.pos,
        2,
    );

    if stolen > 0 {
        hippie.steal_cooldown = constants::HIPPIE_STEAL_COOLDOWN;
        hippie.angry = false;
        hippie.anger_timer = 0.0;
        hippie.anger_delay = 0.0;
        hippie.flee_timer = constants::HIPPIE_FLEE_DURATION;
    }
}

fn update_hippie_anger(hippie: &mut Hippie, player_pos: Vec2, player_has_flags: bool, dt: f32) {
    if !hippie.angry {
        return;
    }

    if !player_has_flags {
        hippie.angry = false;
        hippie.anger_timer = 0.0;
        hippie.anger_delay = 0.0;
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

fn resolve_hippie_collisions(positions: &mut [Vec2], min_distance: f32) {
    if positions.len() < 2 {
        return;
    }

    let min_sq = min_distance * min_distance;
    for _ in 0..2 {
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                let delta = positions[i] - positions[j];
                let dist_sq = delta.length_squared();
                if dist_sq >= min_sq {
                    continue;
                }

                let dist = dist_sq.sqrt();
                let dir = if dist > 0.0 {
                    delta / dist
                } else {
                    vec2(1.0, 0.0)
                };
                let push = if dist > 0.0 {
                    (min_distance - dist) * 0.5
                } else {
                    min_distance * 0.5
                };

                positions[i] += dir * push;
                positions[j] -= dir * push;
            }
        }
    }
}

fn nearest_hippie_with_flag(hippies: &[Hippie], origin: Vec2, radius: f32) -> Option<usize> {
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

pub(crate) fn random_point_in_polygon(vertices: &[Vec2], rng_state: &mut u32) -> Vec2 {
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

fn initial_carried_flags(rng_state: &mut u32) -> u8 {
    carried_flags_from_roll(next_f32(rng_state))
}

fn carried_flags_from_roll(roll: f32) -> u8 {
    let two = constants::HIPPIE_START_TWO_FLAG_CHANCE;
    let one = constants::HIPPIE_START_ONE_FLAG_CHANCE;
    if roll < two {
        2
    } else if roll < two + one {
        1
    } else {
        0
    }
}

fn camp_for_index<'a>(camps: &'a [Vec<Vec2>], index: usize) -> &'a [Vec2] {
    if camps.is_empty() {
        return &[];
    }
    camps
        .get(index)
        .map(|camp| camp.as_slice())
        .unwrap_or_else(|| camps[0].as_slice())
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

// === Burn mode: spawn hippies with properties ===

pub fn spawn_burn_hippies(
    positions: &[Vec2],
    camp_index: usize,
    camp_vertices: &[Vec2],
    seed: u32,
) -> Vec<Hippie> {
    let mut rng = seed;
    positions
        .iter()
        .enumerate()
        .map(|(i, &pos)| {
            let mut hippie_rng = hash_seed(pos, i as u32);
            let carried_flags = initial_carried_flags(&mut hippie_rng);
            let target = random_point_in_polygon(camp_vertices, &mut hippie_rng);

            // Half of hippies start at max psychosis
            let psych_roll = next_f32(&mut rng);
            let flag_psychosis = if psych_roll < 0.5 {
                constants::PSYCHOSIS_MAX
            } else {
                next_f32(&mut rng) * 0.3
            };
            let drunkenness = next_f32(&mut rng);
            let dirtiness = next_f32(&mut rng);

            Hippie {
                pos,
                facing: player::Facing::Down,
                carried_flags,
                angry: false,
                anger_timer: 0.0,
                anger_delay: 0.0,
                steal_cooldown: 0.0,
                flee_timer: 0.0,
                drop_check_timer: next_f32(&mut hippie_rng)
                    * constants::HIPPIE_FLAG_DROP_INTERVAL,
                ignore_flags_timer: 0.0,
                camp_index,
                target,
                speed: HIPPIE_SPEED,
                rng_state: hippie_rng,
                flag_psychosis,
                drunkenness,
                dirtiness,
            }
        })
        .collect()
}

// === Dirtiness rendering ===

pub fn dirty_color_shift(base_color: Color, dirtiness: f32) -> Color {
    let brown = Color::new(0.45, 0.35, 0.25, 1.0);
    let t = dirtiness * 0.6;
    Color::new(
        base_color.r + (brown.r - base_color.r) * t,
        base_color.g + (brown.g - base_color.g) * t,
        base_color.b + (brown.b - base_color.b) * t,
        base_color.a,
    )
}

fn draw_dust_trail(pos: Vec2, dirtiness: f32, time: f32) {
    if dirtiness < constants::DIRTY_DUST_THRESHOLD {
        return;
    }
    let s = scale::MODEL_SCALE;
    let intensity = (dirtiness - constants::DIRTY_DUST_THRESHOLD)
        / (1.0 - constants::DIRTY_DUST_THRESHOLD);
    let dust_color = Color::new(0.65, 0.55, 0.40, 0.15 + intensity * 0.15);

    for i in 0..constants::DIRTY_DUST_PARTICLE_COUNT {
        let phase = i as f32 * 2.1 + pos.x * 0.01;
        let ox = (time * 1.5 + phase).sin() * 6.0 * s;
        let oy = (time * 1.1 + phase * 0.7).cos() * 4.0 * s + 8.0 * s;
        let r = (3.0 + 2.0 * (time * 0.8 + phase).sin().abs()) * s;
        draw_circle(pos.x + ox, pos.y + oy, r, dust_color);
    }
}

fn draw_stink_lines(pos: Vec2, dirtiness: f32, time: f32) {
    if dirtiness < constants::DIRTY_STINK_THRESHOLD {
        return;
    }
    let s = scale::MODEL_SCALE;
    let intensity =
        (dirtiness - constants::DIRTY_STINK_THRESHOLD) / (1.0 - constants::DIRTY_STINK_THRESHOLD);
    let stink_color = Color::new(0.45, 0.55, 0.20, 0.3 + intensity * 0.3);
    let head_y = pos.y - HIPPIE_BODY_LENGTH * 0.5 - HIPPIE_HEAD_RADIUS;

    for i in 0..constants::DIRTY_STINK_LINE_COUNT {
        let phase = i as f32 * 1.3;
        let base_x = pos.x + (i as f32 - 1.0) * 4.0 * s;
        let wave_y = head_y - 8.0 * s - (time * 2.0 + phase).sin().abs() * 6.0 * s;
        let segments = 4;
        for seg in 0..segments {
            let t0 = seg as f32 / segments as f32;
            let t1 = (seg + 1) as f32 / segments as f32;
            let y0 = wave_y - t0 * 10.0 * s;
            let y1 = wave_y - t1 * 10.0 * s;
            let x0 = base_x + (time * 3.0 + phase + t0 * 4.0).sin() * 3.0 * s;
            let x1 = base_x + (time * 3.0 + phase + t1 * 4.0).sin() * 3.0 * s;
            draw_line(x0, y0, x1, y1, 1.0 * s, stink_color);
        }
    }
}

// === Psychosis sparkles ===

const PSYCHOSIS_SPARKLE_COUNT: usize = 8;
const PSYCHOSIS_SPARKLE_MIN_PSYCHOSIS: f32 = 0.1;

fn draw_psychosis_sparkles(pos: Vec2, psychosis: f32, time: f32, seed: u32) {
    if psychosis < PSYCHOSIS_SPARKLE_MIN_PSYCHOSIS {
        return;
    }
    let s = scale::MODEL_SCALE;
    let intensity = (psychosis / constants::PSYCHOSIS_MAX).min(1.0);
    let count = (PSYCHOSIS_SPARKLE_COUNT as f32 * intensity).ceil() as usize;
    let body_height = HIPPIE_BODY_LENGTH + HIPPIE_HEAD_RADIUS * 2.0 + HIPPIE_LEG_LENGTH;

    for i in 0..count {
        let phase = seed as f32 * 0.001 + i as f32 * 1.7;
        let orbit_speed = 2.0 + (i as f32 * 0.3);
        let angle = time * orbit_speed + phase;
        let radius = (8.0 + (i as f32 * 3.0).sin().abs() * 10.0) * s;
        let vert_offset = ((time * 1.5 + phase * 0.5).sin() * 0.5 + 0.5) * body_height * 0.8;

        let ox = angle.cos() * radius;
        let oy = -body_height * 0.3 + vert_offset + angle.sin() * radius * 0.3;

        let sparkle_size = (1.0 + (time * 4.0 + phase).sin().abs() * 1.5) * s * intensity;
        let alpha = (0.4 + 0.4 * (time * 3.0 + phase).sin().abs()) * intensity;

        // Yellow-gold sparkle color
        let r = 1.0;
        let g = 0.85 + 0.15 * (time * 2.0 + phase).sin();
        let b = 0.1 + 0.2 * (time * 3.5 + phase * 1.3).sin().abs();

        draw_circle(
            pos.x + ox,
            pos.y + oy,
            sparkle_size,
            Color::new(r, g, b, alpha),
        );
    }
}

// === Drunkenness mechanics ===

pub fn drunk_speed(base_speed: f32, drunkenness: f32) -> f32 {
    let factor = 1.0 - drunkenness * (1.0 - constants::DRUNK_SPEED_FACTOR_MIN);
    base_speed * factor
}

pub fn drunk_vision_range(base_range: f32, drunkenness: f32) -> f32 {
    let factor = 1.0 - drunkenness * (1.0 - constants::DRUNK_VISION_FACTOR_MIN);
    base_range * factor
}

pub fn drunk_wobble_offset(direction: Vec2, time: f32, drunkenness: f32, seed: f32) -> Vec2 {
    if drunkenness < 0.01 || direction.length_squared() < f32::EPSILON {
        return Vec2::ZERO;
    }
    let perp = vec2(-direction.y, direction.x);
    let wobble = (time * constants::DRUNK_WOBBLE_FREQUENCY + seed).sin()
        * constants::DRUNK_WOBBLE_AMPLITUDE
        * drunkenness;
    perp * wobble
}

// === Pentagram building ===

fn camp_blueprint<'a>(blueprints: &'a [Vec<Vec2>], camp_index: usize) -> &'a [Vec2] {
    blueprints
        .get(camp_index)
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}

fn find_empty_pentagram_slot(
    blueprint: &[Vec2],
    ground_flags: &[flags::Flag],
    hippie_pos: Vec2,
) -> Option<Vec2> {
    let mut best_dist = f32::MAX;
    let mut best_slot = None;

    for &slot in blueprint {
        let occupied = ground_flags
            .iter()
            .any(|f| f.pos.distance(slot) <= constants::PENTAGRAM_SLOT_DETECT_RADIUS);
        if !occupied {
            let dist = hippie_pos.distance(slot);
            if dist < best_dist {
                best_dist = dist;
                best_slot = Some(slot);
            }
        }
    }

    best_slot
}

// === Flag Psychosis AI ===

pub fn update_hippie_psychosis(hippie: &mut Hippie, pentagram_centers: &[Vec2], dt: f32) {
    for center in pentagram_centers {
        let dist = hippie.pos.distance(*center);
        if dist < constants::PSYCHOSIS_PENTAGRAM_RADIUS {
            let proximity = 1.0 - (dist / constants::PSYCHOSIS_PENTAGRAM_RADIUS);
            hippie.flag_psychosis += constants::PSYCHOSIS_PENTAGRAM_GAIN_RATE * proximity * dt;
            hippie.flag_psychosis = hippie.flag_psychosis.min(constants::PSYCHOSIS_MAX);
        }
    }
}

pub fn psychosis_target(
    hippie: &Hippie,
    ground_flags: &[flags::Flag],
    camp_vertices: &[Vec2],
) -> Option<Vec2> {
    if hippie.flag_psychosis < constants::PSYCHOSIS_CHASE_THRESHOLD {
        return None;
    }
    if hippie.carried_flags >= HIPPIE_FLAG_CAPACITY {
        return None;
    }

    let pursuit_range = constants::PSYCHOSIS_BASE_PURSUIT_RANGE
        + (constants::PSYCHOSIS_MAX_PURSUIT_RANGE - constants::PSYCHOSIS_BASE_PURSUIT_RANGE)
            * hippie.flag_psychosis.min(1.0);

    let mut best_dist = pursuit_range;
    let mut best_pos = None;
    for flag in ground_flags {
        if geom::point_in_polygon(flag.pos, camp_vertices) {
            continue;
        }
        let dist = hippie.pos.distance(flag.pos);
        if dist < best_dist {
            best_dist = dist;
            best_pos = Some(flag.pos);
        }
    }
    best_pos
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flag_state::FlagState;
    use crate::flags;

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
    fn carried_flags_roll_respects_chances() {
        assert_eq!(carried_flags_from_roll(0.05), 2);
        assert_eq!(carried_flags_from_roll(0.2), 1);
        assert_eq!(carried_flags_from_roll(0.9), 0);
    }

    #[test]
    fn hippie_update_keeps_inside_camp() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_hippies_with_flags(&[(vec2(10.0, 10.0), 0)], 0, &square);
        let mut flag_state = FlagState::new(Vec::new(), 0, 0);
        let camps = vec![square.clone()];
        for _ in 0..60 {
            update_hippies(
                &mut hippies,
                0.1,
                &camps,
                &mut flag_state,
                vec2(50.0, 50.0),
                100.0,
                GameMode::Classic,
                &[],
                &[],
                0.0,
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
        let mut hippies = spawn_hippies_with_flags(&[(vec2(5.0, 5.0), 1)], 0, &square);
        hippies[0].drop_check_timer = 0.0;
        hippies[0].rng_state = 0;
        let mut flag_state = FlagState::new(Vec::new(), 0, 1);
        update_hippie_drop(&mut hippies[0], 0.1, &mut flag_state);
        assert_eq!(hippies[0].carried_flags, 0);
        assert_eq!(flag_state.ground_flags().len(), 1);
        assert!(
            (hippies[0].ignore_flags_timer - constants::HIPPIE_FLAG_IGNORE_DURATION).abs() < 1e-6
        );
    }

    #[test]
    fn hippie_ignores_pickup_during_cooldown() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_hippies_with_flags(&[(vec2(5.0, 5.0), 0)], 0, &square);
        hippies[0].ignore_flags_timer = constants::HIPPIE_FLAG_IGNORE_DURATION;
        let mut flag_state = FlagState::new(
            vec![flags::Flag {
                pos: vec2(5.0, 5.0),
                phase: 0.0,
            }],
            0,
            1,
        );
        let camps = vec![square.clone()];
        update_hippies(
            &mut hippies,
            0.0,
            &camps,
            &mut flag_state,
            vec2(0.0, 0.0),
            100.0,
            GameMode::Classic,
            &[],
            &[],
            0.0,
        );
        assert_eq!(flag_state.ground_flags().len(), 1);
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
        let mut hippies = spawn_hippies_with_flags(&[(vec2(10.0, 10.0), 0)], 0, &square);
        let flags = vec![
            flags::Flag {
                pos: vec2(10.0, 11.0),
                phase: 0.0,
            },
            flags::Flag {
                pos: vec2(9.0, 10.0),
                phase: 0.0,
            },
            flags::Flag {
                pos: vec2(12.0, 10.0),
                phase: 0.0,
            },
        ];
        let mut flag_state = FlagState::new(flags, 0, 3);
        let camps = vec![square.clone()];
        let picked = update_hippies(
            &mut hippies,
            0.0,
            &camps,
            &mut flag_state,
            vec2(0.0, 0.0),
            100.0,
            GameMode::Classic,
            &[],
            &[],
            0.0,
        );
        assert!(picked);
        assert_eq!(hippies[0].carried_flags, 2);
        assert_eq!(flag_state.ground_flags().len(), 1);
    }

    #[test]
    fn hippie_does_not_pick_up_when_full() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_hippies_with_flags(&[(vec2(10.0, 10.0), 0)], 0, &square);
        hippies[0].carried_flags = HIPPIE_FLAG_CAPACITY;
        let mut flag_state = FlagState::new(
            vec![flags::Flag {
                pos: vec2(10.0, 10.0),
                phase: 0.0,
            }],
            0,
            3,
        );
        let camps = vec![square.clone()];
        let picked = update_hippies(
            &mut hippies,
            0.0,
            &camps,
            &mut flag_state,
            vec2(0.0, 0.0),
            100.0,
            GameMode::Classic,
            &[],
            &[],
            0.0,
        );
        assert!(!picked);
        assert_eq!(flag_state.ground_flags().len(), 1);
    }

    #[test]
    fn spawn_hippies_with_flags_clamps_capacity() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let hippies = spawn_hippies_with_flags(&[(vec2(5.0, 5.0), 5)], 0, &square);
        assert_eq!(hippies[0].carried_flags, HIPPIE_FLAG_CAPACITY);
    }

    #[test]
    fn hippie_flag_cloth_anchor_is_at_pole_top() {
        let hand = vec2(0.0, 0.0);
        let (rotation, cloth_sign) = hippie_flag_orientation(player::Facing::Right);
        let pole_top = hand + rotate_vec(vec2(0.0, -constants::FLAG_POLE_HEIGHT), rotation);
        let cloth_anchor = pole_top
            + rotate_vec(
                vec2(cloth_sign * constants::FLAG_POLE_WIDTH * 0.5, 0.0),
                rotation,
            );
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
                camp_index: 0,
                target: vec2(0.0, 0.0),
                speed: HIPPIE_SPEED,
                rng_state: 1,
                flag_psychosis: 0.0,
                drunkenness: 0.0,
                dirtiness: 0.0,
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
                camp_index: 0,
                target: vec2(0.0, 0.0),
                speed: HIPPIE_SPEED,
                rng_state: 2,
                flag_psychosis: 0.0,
                drunkenness: 0.0,
                dirtiness: 0.0,
            },
        ];

        let mut flag_state = FlagState::new(Vec::new(), 0, 3);
        let stolen = try_steal_flag(&mut hippies, vec2(2.5, 0.0), 4.0, &mut flag_state);
        assert!(stolen);
        assert_eq!(hippies[1].carried_flags, 1);
        assert_eq!(hippies[0].carried_flags, 1);
        assert_eq!(flag_state.player_inventory(), 1);
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
            camp_index: 0,
            target: vec2(0.0, 0.0),
            speed: HIPPIE_SPEED,
            rng_state: 1,
            flag_psychosis: 0.0,
            drunkenness: 0.0,
            dirtiness: 0.0,
        }];

        let mut flag_state = FlagState::new(Vec::new(), 0, 0);
        let stolen = try_steal_flag(&mut hippies, vec2(0.0, 0.0), 4.0, &mut flag_state);
        assert!(!stolen);
        assert_eq!(flag_state.player_inventory(), 0);
    }

    #[test]
    fn anger_clears_when_timer_elapsed_and_far() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_hippies_with_flags(&[(vec2(5.0, 5.0), 0)], 0, &square);
        hippies[0].angry = true;
        hippies[0].anger_timer = 0.0;
        hippies[0].flee_timer = 0.0;
        hippies[0].anger_delay = 0.0;
        hippies[0].drop_check_timer = 0.0;
        hippies[0].ignore_flags_timer = 0.0;
        let mut flag_state = FlagState::new(Vec::new(), 1, 1);
        let camps = vec![square.clone()];
        update_hippies(
            &mut hippies,
            0.1,
            &camps,
            &mut flag_state,
            vec2(100.0, 100.0),
            100.0,
            GameMode::Classic,
            &[],
            &[],
            0.0,
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
        let mut hippies = spawn_hippies_with_flags(&[(vec2(5.0, 5.0), 0)], 0, &square);
        hippies[0].angry = true;
        hippies[0].anger_timer = 0.0;
        hippies[0].flee_timer = 0.0;
        hippies[0].anger_delay = 0.0;
        hippies[0].drop_check_timer = 0.0;
        hippies[0].ignore_flags_timer = 0.0;
        let mut flag_state = FlagState::new(Vec::new(), 1, 1);
        let camps = vec![square.clone()];
        update_hippies(
            &mut hippies,
            0.1,
            &camps,
            &mut flag_state,
            vec2(12.0, 12.0),
            100.0,
            GameMode::Classic,
            &[],
            &[],
            0.0,
        );
        assert!(hippies[0].angry);
    }

    #[test]
    fn angry_stops_when_player_has_no_flags() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_hippies_with_flags(&[(vec2(5.0, 5.0), 0)], 0, &square);
        hippies[0].angry = true;
        hippies[0].anger_timer = constants::HIPPIE_ANGER_DURATION;
        hippies[0].flee_timer = 0.0;
        hippies[0].anger_delay = 0.0;
        hippies[0].drop_check_timer = 0.0;
        hippies[0].ignore_flags_timer = 0.0;
        let mut flag_state = FlagState::new(Vec::new(), 0, 0);
        let camps = vec![square.clone()];
        update_hippies(
            &mut hippies,
            0.1,
            &camps,
            &mut flag_state,
            vec2(6.0, 6.0),
            100.0,
            GameMode::Classic,
            &[],
            &[],
            0.0,
        );
        assert!(!hippies[0].angry);
        assert_eq!(hippies[0].anger_timer, 0.0);
        assert_eq!(hippies[0].anger_delay, 0.0);
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
        let mut hippies = spawn_hippies_with_flags(&[(vec2(5.0, 5.0), 0)], 0, &square);
        hippies[0].angry = true;
        hippies[0].anger_timer = constants::HIPPIE_ANGER_DURATION;
        hippies[0].flee_timer = 0.0;
        hippies[0].anger_delay = 0.0;
        hippies[0].drop_check_timer = 0.0;
        hippies[0].ignore_flags_timer = 0.0;
        let mut flag_state = FlagState::new(Vec::new(), 3, 3);
        let total_before =
            flag_state.current_total(hippies.iter().map(|h| h.carried_flags as u32).sum::<u32>());
        let camps = vec![square.clone()];
        update_hippies(
            &mut hippies,
            0.0,
            &camps,
            &mut flag_state,
            vec2(5.0, 5.0),
            100.0,
            GameMode::Classic,
            &[],
            &[],
            0.0,
        );
        assert_eq!(flag_state.player_inventory(), 1);
        assert_eq!(hippies[0].carried_flags, 2);
        let total_after =
            flag_state.current_total(hippies.iter().map(|h| h.carried_flags as u32).sum::<u32>());
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
        let mut hippies = spawn_hippies_with_flags(&[(vec2(5.0, 5.0), 2)], 0, &square);
        hippies[0].angry = true;
        hippies[0].anger_timer = constants::HIPPIE_ANGER_DURATION;
        let mut flag_state = FlagState::new(Vec::new(), 2, 4);
        let total_before =
            flag_state.current_total(hippies.iter().map(|h| h.carried_flags as u32).sum::<u32>());
        let camps = vec![square.clone()];
        update_hippies(
            &mut hippies,
            0.0,
            &camps,
            &mut flag_state,
            vec2(5.0, 5.0),
            100.0,
            GameMode::Classic,
            &[],
            &[],
            0.0,
        );
        assert_eq!(flag_state.player_inventory(), 0);
        assert_eq!(hippies[0].carried_flags, 2);
        assert_eq!(flag_state.ground_flags().len(), 2);
        let total_after =
            flag_state.current_total(hippies.iter().map(|h| h.carried_flags as u32).sum::<u32>());
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
        let mut hippies = spawn_hippies_with_flags(&[(vec2(5.0, 5.0), 0)], 0, &square);
        hippies[0].angry = true;
        hippies[0].anger_timer = constants::HIPPIE_ANGER_DURATION;
        hippies[0].flee_timer = 0.0;
        hippies[0].anger_delay = 0.0;
        hippies[0].drop_check_timer = 0.0;
        hippies[0].ignore_flags_timer = 0.0;
        let mut flag_state = FlagState::new(Vec::new(), 2, 2);
        let camps = vec![square.clone()];
        update_hippies(
            &mut hippies,
            0.0,
            &camps,
            &mut flag_state,
            vec2(5.0, 5.0),
            100.0,
            GameMode::Classic,
            &[],
            &[],
            0.0,
        );
        let after_first = flag_state.player_inventory();
        update_hippies(
            &mut hippies,
            0.0,
            &camps,
            &mut flag_state,
            vec2(5.0, 5.0),
            100.0,
            GameMode::Classic,
            &[],
            &[],
            0.0,
        );
        assert_eq!(flag_state.player_inventory(), after_first);
    }

    #[test]
    fn chase_speed_uses_player_speed_factor() {
        let speed = chase_speed(100.0);
        assert!((speed - 100.0 * constants::HIPPIE_CHASE_SPEED_FACTOR).abs() < 1e-6);
    }

    #[test]
    fn angry_hippie_steps_to_player_not_wander_target() {
        let square = vec![
            vec2(0.0, 0.0),
            vec2(200.0, 0.0),
            vec2(200.0, 200.0),
            vec2(0.0, 200.0),
        ];
        let mut hippies = spawn_hippies_with_flags(&[(vec2(50.0, 50.0), 0)], 0, &square);
        hippies[0].angry = true;
        hippies[0].anger_timer = constants::HIPPIE_ANGER_DURATION;
        hippies[0].anger_delay = 0.0;
        hippies[0].flee_timer = 0.0;
        hippies[0].drop_check_timer = 0.0;
        hippies[0].ignore_flags_timer = 0.0;
        hippies[0].target = vec2(150.0, 150.0);

        let mut flag_state = FlagState::new(Vec::new(), 1, 1);
        let player_pos = vec2(60.0, 50.0);
        let camps = vec![square.clone()];
        update_hippies(
            &mut hippies,
            1.0,
            &camps,
            &mut flag_state,
            player_pos,
            1000.0,
            GameMode::Classic,
            &[],
            &[],
            0.0,
        );

        let dist_to_player = hippies[0].pos.distance(player_pos);
        let dist_to_target = hippies[0].pos.distance(hippies[0].target);
        assert!(dist_to_player < dist_to_target);
    }

    #[test]
    fn angry_hippie_can_leave_camp_boundary() {
        let camp = vec![
            vec2(0.0, 0.0),
            vec2(10.0, 0.0),
            vec2(10.0, 10.0),
            vec2(0.0, 10.0),
        ];
        let mut hippies = spawn_hippies_with_flags(&[(vec2(9.0, 5.0), 0)], 0, &camp);
        hippies[0].angry = true;
        hippies[0].anger_timer = constants::HIPPIE_ANGER_DURATION;
        hippies[0].anger_delay = 0.0;
        hippies[0].flee_timer = 0.0;
        hippies[0].drop_check_timer = 0.0;
        hippies[0].ignore_flags_timer = 0.0;

        let mut flag_state = FlagState::new(Vec::new(), 1, 1);
        let camps = vec![camp.clone()];
        update_hippies(
            &mut hippies,
            0.2,
            &camps,
            &mut flag_state,
            vec2(30.0, 5.0),
            100.0,
            GameMode::Classic,
            &[],
            &[],
            0.0,
        );

        assert!(!geom::point_in_polygon(hippies[0].pos, &camp));
    }

    #[test]
    fn calm_hippie_moves_toward_camp_when_outside() {
        let camp = vec![
            vec2(0.0, 0.0),
            vec2(10.0, 0.0),
            vec2(10.0, 10.0),
            vec2(0.0, 10.0),
        ];
        let mut hippies = spawn_hippies_with_flags(&[(vec2(15.0, 5.0), 0)], 0, &camp);
        hippies[0].angry = false;
        hippies[0].anger_timer = 0.0;
        hippies[0].anger_delay = 0.0;
        hippies[0].flee_timer = 0.0;
        hippies[0].drop_check_timer = 0.0;
        hippies[0].ignore_flags_timer = 0.0;
        hippies[0].target = vec2(5.0, 5.0);

        let mut flag_state = FlagState::new(Vec::new(), 1, 1);
        let camps = vec![camp.clone()];
        let before = hippies[0].pos.distance(hippies[0].target);
        update_hippies(
            &mut hippies,
            0.2,
            &camps,
            &mut flag_state,
            vec2(100.0, 100.0),
            100.0,
            GameMode::Classic,
            &[],
            &[],
            0.0,
        );
        let after = hippies[0].pos.distance(hippies[0].target);
        assert!(after < before);
    }

    #[test]
    fn hippie_collision_prevents_overlap() {
        let camp = vec![
            vec2(0.0, 0.0),
            vec2(200.0, 0.0),
            vec2(200.0, 200.0),
            vec2(0.0, 200.0),
        ];
        let mut hippies =
            spawn_hippies_with_flags(&[(vec2(40.0, 50.0), 0), (vec2(60.0, 50.0), 0)], 0, &camp);
        for hippie in &mut hippies {
            hippie.angry = true;
            hippie.anger_timer = constants::HIPPIE_ANGER_DURATION;
            hippie.anger_delay = 0.0;
            hippie.flee_timer = 0.0;
            hippie.drop_check_timer = 0.0;
            hippie.ignore_flags_timer = 0.0;
        }

        let mut flag_state = FlagState::new(Vec::new(), 1, 1);
        let camps = vec![camp.clone()];
        update_hippies(
            &mut hippies,
            1.0,
            &camps,
            &mut flag_state,
            vec2(50.0, 50.0),
            200.0,
            GameMode::Classic,
            &[],
            &[],
            0.0,
        );

        let distance = hippies[0].pos.distance(hippies[1].pos);
        assert!(
            distance >= constants::HIPPIE_COLLISION_RADIUS * 2.0 - 1e-3,
            "hippies too close after collision: {}",
            distance
        );
    }

    // === Dirtiness tests ===

    #[test]
    fn dirty_color_shift_at_zero_unchanged() {
        let base = Color::new(0.95, 0.86, 0.74, 1.0);
        let shifted = dirty_color_shift(base, 0.0);
        assert!((shifted.r - base.r).abs() < 1e-6);
        assert!((shifted.g - base.g).abs() < 1e-6);
        assert!((shifted.b - base.b).abs() < 1e-6);
    }

    #[test]
    fn dirty_color_shift_at_one_is_brownish() {
        let base = Color::new(0.95, 0.86, 0.74, 1.0);
        let shifted = dirty_color_shift(base, 1.0);
        assert!(shifted.r < base.r);
        assert!(shifted.g < base.g);
        assert!(shifted.b < base.b);
    }

    #[test]
    fn dirty_color_shift_preserves_alpha() {
        let base = Color::new(0.95, 0.86, 0.74, 0.5);
        let shifted = dirty_color_shift(base, 0.5);
        assert!((shifted.a - 0.5).abs() < 1e-6);
    }

    // === Drunkenness tests ===

    #[test]
    fn drunk_speed_at_zero_is_full() {
        assert!((drunk_speed(100.0, 0.0) - 100.0).abs() < 1e-6);
    }

    #[test]
    fn drunk_speed_at_one_is_half() {
        let speed = drunk_speed(100.0, 1.0);
        assert!((speed - 100.0 * constants::DRUNK_SPEED_FACTOR_MIN).abs() < 1e-4);
    }

    #[test]
    fn drunk_speed_interpolates() {
        let speed = drunk_speed(100.0, 0.5);
        let expected = 100.0 * (1.0 - 0.5 * (1.0 - constants::DRUNK_SPEED_FACTOR_MIN));
        assert!((speed - expected).abs() < 1e-4);
    }

    #[test]
    fn drunk_vision_range_scales() {
        let full = drunk_vision_range(100.0, 0.0);
        let min = drunk_vision_range(100.0, 1.0);
        assert!((full - 100.0).abs() < 1e-6);
        assert!((min - 100.0 * constants::DRUNK_VISION_FACTOR_MIN).abs() < 1e-4);
    }

    #[test]
    fn drunk_wobble_zero_when_sober() {
        let wobble = drunk_wobble_offset(vec2(1.0, 0.0), 1.0, 0.0, 0.0);
        assert!(wobble.length() < 1e-6);
    }

    #[test]
    fn drunk_wobble_bounded() {
        for t in 0..100 {
            let wobble = drunk_wobble_offset(vec2(1.0, 0.0), t as f32 * 0.1, 1.0, 0.0);
            assert!(
                wobble.length() <= constants::DRUNK_WOBBLE_AMPLITUDE * 1.01,
                "Wobble {} exceeds amplitude {}",
                wobble.length(),
                constants::DRUNK_WOBBLE_AMPLITUDE
            );
        }
    }

    // === Flag Psychosis tests ===

    #[test]
    fn psychosis_increases_near_pentagram() {
        let camp = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_burn_hippies(&[vec2(10.0, 10.0)], 0, &camp, 42);
        hippies[0].flag_psychosis = 0.0;
        let pentagram = vec2(10.0, 10.0);
        update_hippie_psychosis(&mut hippies[0], &[pentagram], 1.0);
        assert!(hippies[0].flag_psychosis > 0.0);
    }

    #[test]
    fn psychosis_does_not_increase_far_from_pentagram() {
        let camp = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_burn_hippies(&[vec2(10.0, 10.0)], 0, &camp, 42);
        hippies[0].flag_psychosis = 0.0;
        let far_pentagram = vec2(10000.0, 10000.0);
        update_hippie_psychosis(&mut hippies[0], &[far_pentagram], 1.0);
        assert!((hippies[0].flag_psychosis).abs() < 1e-6);
    }

    #[test]
    fn psychosis_capped_at_max() {
        let camp = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_burn_hippies(&[vec2(10.0, 10.0)], 0, &camp, 42);
        hippies[0].flag_psychosis = constants::PSYCHOSIS_MAX;
        update_hippie_psychosis(&mut hippies[0], &[vec2(10.0, 10.0)], 100.0);
        assert!(hippies[0].flag_psychosis <= constants::PSYCHOSIS_MAX);
    }

    #[test]
    fn psychosis_target_returns_none_below_threshold() {
        let camp = vec![
            vec2(0.0, 0.0),
            vec2(20.0, 0.0),
            vec2(20.0, 20.0),
            vec2(0.0, 20.0),
        ];
        let mut hippies = spawn_burn_hippies(&[vec2(10.0, 10.0)], 0, &camp, 42);
        hippies[0].flag_psychosis = 0.0;
        let flags = vec![flags::Flag {
            pos: vec2(50.0, 50.0),
            phase: 0.0,
        }];
        assert!(psychosis_target(&hippies[0], &flags, &camp).is_none());
    }

    #[test]
    fn psychosis_target_ignores_flags_inside_camp() {
        let camp = vec![
            vec2(0.0, 0.0),
            vec2(100.0, 0.0),
            vec2(100.0, 100.0),
            vec2(0.0, 100.0),
        ];
        let mut hippies = spawn_burn_hippies(&[vec2(50.0, 50.0)], 0, &camp, 42);
        hippies[0].flag_psychosis = 1.0;
        hippies[0].carried_flags = 0;
        let flags = vec![flags::Flag {
            pos: vec2(50.0, 60.0),
            phase: 0.0,
        }];
        assert!(psychosis_target(&hippies[0], &flags, &camp).is_none());
    }

    #[test]
    fn psychosis_target_finds_nearest_outside_flag() {
        let camp = vec![
            vec2(0.0, 0.0),
            vec2(10.0, 0.0),
            vec2(10.0, 10.0),
            vec2(0.0, 10.0),
        ];
        let mut hippies = spawn_burn_hippies(&[vec2(5.0, 5.0)], 0, &camp, 42);
        hippies[0].flag_psychosis = 1.0;
        hippies[0].carried_flags = 0;
        let flags = vec![
            flags::Flag {
                pos: vec2(15.0, 5.0),
                phase: 0.0,
            },
            flags::Flag {
                pos: vec2(50.0, 50.0),
                phase: 0.0,
            },
        ];
        let target = psychosis_target(&hippies[0], &flags, &camp);
        assert!(target.is_some());
        let t = target.unwrap();
        assert!((t.x - 15.0).abs() < 1e-3);
    }

    #[test]
    fn burn_hippie_spawn_has_properties() {
        let camp = vec![
            vec2(0.0, 0.0),
            vec2(100.0, 0.0),
            vec2(100.0, 100.0),
            vec2(0.0, 100.0),
        ];
        let hippies = spawn_burn_hippies(
            &[vec2(50.0, 50.0), vec2(60.0, 60.0), vec2(70.0, 70.0)],
            0,
            &camp,
            42,
        );
        assert_eq!(hippies.len(), 3);
        // Properties should be initialized (not all zero since seed varies)
        let has_nonzero = hippies
            .iter()
            .any(|h| h.drunkenness > 0.0 || h.dirtiness > 0.0);
        assert!(has_nonzero, "Expected at least some non-zero properties");
    }

    #[test]
    fn classic_mode_hippie_properties_zero() {
        let camp = vec![
            vec2(0.0, 0.0),
            vec2(100.0, 0.0),
            vec2(100.0, 100.0),
            vec2(0.0, 100.0),
        ];
        let hippies = spawn_hippies(&[vec2(50.0, 50.0)], 0, &camp);
        assert_eq!(hippies[0].flag_psychosis, 0.0);
        assert_eq!(hippies[0].drunkenness, 0.0);
        assert_eq!(hippies[0].dirtiness, 0.0);
    }

    // === Pentagram building tests ===

    #[test]
    fn find_empty_pentagram_slot_returns_nearest_empty() {
        let blueprint = vec![
            vec2(100.0, 100.0),
            vec2(200.0, 100.0),
            vec2(300.0, 100.0),
        ];
        // No ground flags - all slots empty
        let slot = find_empty_pentagram_slot(&blueprint, &[], vec2(90.0, 100.0));
        assert!(slot.is_some());
        assert!((slot.unwrap().x - 100.0).abs() < 1e-3);
    }

    #[test]
    fn find_empty_pentagram_slot_skips_occupied() {
        let blueprint = vec![
            vec2(100.0, 100.0),
            vec2(200.0, 100.0),
            vec2(300.0, 100.0),
        ];
        // Flag at first slot
        let ground = vec![flags::Flag {
            pos: vec2(100.0, 100.0),
            phase: 0.0,
        }];
        let slot = find_empty_pentagram_slot(&blueprint, &ground, vec2(90.0, 100.0));
        assert!(slot.is_some());
        assert!((slot.unwrap().x - 200.0).abs() < 1e-3);
    }

    #[test]
    fn find_empty_pentagram_slot_returns_none_when_all_filled() {
        let blueprint = vec![vec2(100.0, 100.0), vec2(200.0, 100.0)];
        let ground = vec![
            flags::Flag {
                pos: vec2(100.0, 100.0),
                phase: 0.0,
            },
            flags::Flag {
                pos: vec2(200.0, 100.0),
                phase: 0.0,
            },
        ];
        let slot = find_empty_pentagram_slot(&blueprint, &ground, vec2(150.0, 100.0));
        assert!(slot.is_none());
    }

    #[test]
    fn hippie_drops_flag_at_pentagram_slot() {
        let camp = vec![
            vec2(0.0, 0.0),
            vec2(200.0, 0.0),
            vec2(200.0, 200.0),
            vec2(0.0, 200.0),
        ];
        // Place hippie at slot position with flags and high psychosis
        let slot = vec2(100.0, 100.0);
        let blueprint = vec![vec![slot]];
        let mut hippies = spawn_burn_hippies(&[slot], 0, &camp, 42);
        hippies[0].flag_psychosis = constants::PSYCHOSIS_MAX;
        hippies[0].carried_flags = 2;
        hippies[0].target = slot;

        let mut flag_state = FlagState::new(Vec::new(), 0, 2);
        let camps = vec![camp.clone()];

        update_hippies(
            &mut hippies,
            0.0,
            &camps,
            &mut flag_state,
            vec2(500.0, 500.0),
            100.0,
            GameMode::Burn,
            &[],
            &blueprint,
            0.0,
        );

        // Hippie should have dropped a flag at the slot
        assert_eq!(hippies[0].carried_flags, 1);
        assert_eq!(flag_state.ground_flags().len(), 1);
        let dropped = &flag_state.ground_flags()[0];
        assert!((dropped.pos.x - slot.x).abs() < 1e-3);
        assert!((dropped.pos.y - slot.y).abs() < 1e-3);
    }
}
