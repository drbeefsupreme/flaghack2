use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::constants;
use crate::fire;
use crate::scale;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneryKind {
    Tree,
    Tent,
    Chair,
    Campfire,
    CrowBase,
    Crow,
    Dome,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DomeDecoration {
    Crystal,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneryItem {
    pub kind: SceneryKind,
    pub pos: Vec2,
    pub scale: f32,
    pub rotation: f32,
    pub variant: u8,
    pub decorations: Vec<DomeDecoration>,
}

const BASE_W: f32 = 800.0;
const BASE_H: f32 = 600.0;
const DOME_COUNT: usize = 2;
const DOME_PADDING: f32 = 120.0 * scale::MODEL_SCALE;
const DOME_SCALE: f32 = 1.5;
const CRYSTAL_SCALE: f32 = 1.5;
pub const DOME_RADIUS: f32 = 100.0 * scale::MODEL_SCALE * DOME_SCALE;
pub const DOME_HEIGHT: f32 = 100.0 * scale::MODEL_SCALE * DOME_SCALE;
const CRYSTAL_DOME_POS: Vec2 = Vec2::new(4900.0, 3184.0);
const LARGE_CAMPFIRE_SCALE: f32 = 1.5;
const T3MPCAMP_TENT_SPACING: f32 = 14.0;
const T3MPCAMP_ROW1_START: Vec2 = Vec2::new(4926.0, 3300.0);
const T3MPCAMP_ROW1_END: Vec2 = Vec2::new(5000.0, 3300.0);
const T3MPCAMP_ROW2_START: Vec2 = Vec2::new(4926.0, 3317.0);
const T3MPCAMP_ROW2_END: Vec2 = Vec2::new(5000.0, 3317.0);

const TENT_COLORS: [Color; 5] = [
    Color::new(0.88, 0.48, 0.22, 1.0),
    Color::new(0.23, 0.51, 0.96, 1.0),
    Color::new(0.13, 0.77, 0.37, 1.0),
    Color::new(0.96, 0.62, 0.04, 1.0),
    Color::new(0.92, 0.28, 0.60, 1.0),
];

pub fn spawn_scenery(field: Rect) -> Vec<SceneryItem> {
    let mut items = Vec::new();

    let tents = [
        vec2(80.0, 120.0),
        vec2(700.0, 100.0),
        vec2(650.0, 480.0),
        vec2(100.0, 450.0),
        vec2(400.0, 80.0),
    ];

    for (i, base) in tents.iter().enumerate() {
        let pos = map_position(field, *base);
        items.push(SceneryItem {
            kind: SceneryKind::Tent,
            pos,
            scale: 1.0,
            rotation: 0.0,
            variant: i as u8 % TENT_COLORS.len() as u8,
            decorations: Vec::new(),
        });
    }

    // t3mpcamp: special camp area within the hand-authored region polygon.
    add_t3mpcamp_tents(&mut items);

    let chairs = [
        vec2(150.0, 200.0),
        vec2(620.0, 180.0),
        vec2(180.0, 380.0),
        vec2(580.0, 420.0),
        vec2(350.0, 520.0),
    ];
    let chair_rotations = [-0.4, 0.6, 0.2, -0.7, 1.1];

    for (i, base) in chairs.iter().enumerate() {
        let pos = map_position(field, *base);
        items.push(SceneryItem {
            kind: SceneryKind::Chair,
            pos,
            scale: 1.0,
            rotation: chair_rotations[i % chair_rotations.len()],
            variant: 0,
            decorations: Vec::new(),
        });
    }

    let campfires = [vec2(200.0, 300.0), vec2(550.0, 350.0)];
    for base in campfires {
        let pos = map_position(field, base);
        items.push(SceneryItem {
            kind: SceneryKind::Campfire,
            pos,
            scale: 1.0,
            rotation: 0.0,
            variant: 0,
            decorations: Vec::new(),
        });
    }

    items.push(SceneryItem {
        kind: SceneryKind::Campfire,
        pos: constants::T3MPCAMP_CAMPFIRE_POS,
        scale: LARGE_CAMPFIRE_SCALE,
        rotation: 0.0,
        variant: 0,
        decorations: Vec::new(),
    });

    let crow_base_pos = vec2(5065.0, 3327.0);
    items.push(SceneryItem {
        kind: SceneryKind::CrowBase,
        pos: crow_base_pos,
        scale: 1.0,
        rotation: 0.0,
        variant: 0,
        decorations: Vec::new(),
    });
    items.push(SceneryItem {
        kind: SceneryKind::Crow,
        pos: crow_base_pos,
        scale: 1.0,
        rotation: 0.0,
        variant: 0,
        decorations: Vec::new(),
    });

    items.push(SceneryItem {
        kind: SceneryKind::Dome,
        pos: CRYSTAL_DOME_POS,
        scale: 1.0,
        rotation: 0.0,
        variant: 0,
        decorations: vec![DomeDecoration::Crystal],
    });

    let trees = [
        vec2(30.0, 50.0),
        vec2(770.0, 40.0),
        vec2(20.0, 550.0),
        vec2(750.0, 560.0),
        vec2(400.0, 30.0),
    ];

    for base in trees {
        let pos = map_position(field, base);
        let scale = tree_scale_from_pos(pos);
        items.push(SceneryItem {
            kind: SceneryKind::Tree,
            pos,
            scale,
            rotation: 0.0,
            variant: 0,
            decorations: Vec::new(),
        });
    }

    for i in 0..DOME_COUNT {
        let pos = random_position(field, DOME_PADDING);
        let decorations = if i == 0 {
            vec![DomeDecoration::Crystal]
        } else {
            Vec::new()
        };
        items.push(SceneryItem {
            kind: SceneryKind::Dome,
            pos,
            scale: 1.0,
            rotation: 0.0,
            variant: 0,
            decorations,
        });
    }

    items
}

pub fn draw_scenery(items: &[SceneryItem], time: f32) {
    for item in items {
        match item.kind {
            SceneryKind::Tree => draw_tree(item.pos, item.scale),
            SceneryKind::Tent => draw_tent(item.pos, item.variant),
            SceneryKind::Chair => draw_chair(item.pos, item.rotation),
            SceneryKind::Campfire => draw_campfire(item.pos, time, item.scale),
            SceneryKind::CrowBase => draw_crow_base(item.pos, time),
            SceneryKind::Crow => draw_crow(item.pos, time),
            SceneryKind::Dome => draw_geodesic_dome(item.pos, time, &item.decorations),
        }
    }
}

fn map_position(field: Rect, base: Vec2) -> Vec2 {
    vec2(
        field.x + (base.x / BASE_W) * field.w,
        field.y + (base.y / BASE_H) * field.h,
    )
}

fn tree_scale_from_pos(pos: Vec2) -> f32 {
    let seed = (pos.x * 0.037 + pos.y * 0.051).sin().abs();
    0.8 + seed * 0.4
}

fn random_position(field: Rect, padding: f32) -> Vec2 {
    let min_x = field.x + padding;
    let max_x = field.x + field.w - padding;
    let min_y = field.y + padding;
    let max_y = field.y + field.h - padding;

    if max_x <= min_x || max_y <= min_y {
        return vec2(field.x + field.w * 0.5, field.y + field.h * 0.5);
    }

    vec2(gen_range(min_x, max_x), gen_range(min_y, max_y))
}

fn add_t3mpcamp_tents(items: &mut Vec<SceneryItem>) {
    let rows = [
        (T3MPCAMP_ROW1_START, T3MPCAMP_ROW1_END),
        (T3MPCAMP_ROW2_START, T3MPCAMP_ROW2_END),
    ];

    for (row_index, (start, end)) in rows.iter().enumerate() {
        let positions = line_points(*start, *end, T3MPCAMP_TENT_SPACING);
        for (i, pos) in positions.into_iter().enumerate() {
            items.push(SceneryItem {
                kind: SceneryKind::Tent,
                pos,
                scale: 1.0,
                rotation: 0.0,
                variant: ((row_index + i) % TENT_COLORS.len()) as u8,
                decorations: Vec::new(),
            });
        }
    }
}

fn line_points(start: Vec2, end: Vec2, spacing: f32) -> Vec<Vec2> {
    let length = (end - start).length();
    if length < f32::EPSILON || spacing <= f32::EPSILON {
        return vec![start];
    }

    let segments = ((length / spacing).round() as usize).max(1);
    let count = segments + 1;
    let mut points = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / (count - 1) as f32;
        points.push(start + (end - start) * t);
    }
    points
}

fn draw_tent(pos: Vec2, variant: u8) {
    let color = TENT_COLORS[variant as usize % TENT_COLORS.len()];
    let size = 28.0 * scale::MODEL_SCALE;

    draw_triangle(
        vec2(pos.x, pos.y - size * 0.9),
        vec2(pos.x - size, pos.y),
        vec2(pos.x + size, pos.y),
        color,
    );

    draw_triangle(
        vec2(pos.x, pos.y - size * 0.2),
        vec2(pos.x - size * 0.3, pos.y),
        vec2(pos.x + size * 0.3, pos.y),
        Color::new(0.0, 0.0, 0.0, 0.3),
    );

    draw_triangle_lines(
        vec2(pos.x, pos.y - size * 0.9),
        vec2(pos.x - size, pos.y),
        vec2(pos.x + size, pos.y),
        1.5 * scale::MODEL_SCALE,
        Color::new(0.0, 0.0, 0.0, 0.35),
    );
}

fn draw_chair(pos: Vec2, rotation: f32) {
    let seat_color = Color::new(0.29, 0.33, 0.39, 1.0);
    let back_color = Color::new(0.18, 0.22, 0.28, 1.0);
    let leg_color = Color::new(0.12, 0.16, 0.20, 1.0);
    let s = scale::MODEL_SCALE;
    let seat = vec2(20.0 * s, 12.0 * s);
    let back = vec2(20.0 * s, 10.0 * s);

    draw_rotated_rect(pos, seat, rotation, seat_color);
    draw_rotated_rect(pos - vec2(0.0, seat.y * 0.7), back, rotation, back_color);

    let leg_offset = vec2(6.0 * s, 6.0 * s);
    let leg_len = 8.0 * s;
    let left = rotate_point(pos + vec2(-leg_offset.x, leg_offset.y), pos, rotation);
    let right = rotate_point(pos + vec2(leg_offset.x, leg_offset.y), pos, rotation);

    draw_line(
        left.x,
        left.y,
        left.x - 2.0 * s,
        left.y + leg_len,
        2.0 * s,
        leg_color,
    );
    draw_line(
        right.x,
        right.y,
        right.x + 2.0 * s,
        right.y + leg_len,
        2.0 * s,
        leg_color,
    );
}

fn draw_campfire(pos: Vec2, time: f32, scale: f32) {
    let stone_color = Color::new(0.33, 0.33, 0.33, 1.0);
    let log_color = Color::new(0.36, 0.25, 0.20, 1.0);
    let s = scale::MODEL_SCALE * scale;

    let stone_angles: [f32; 8] = [0.0, 0.8, 1.6, 2.4, 3.2, 4.0, 4.8, 5.6];
    for angle in stone_angles {
        let sx = pos.x + angle.cos() * 16.0 * s;
        let sy = pos.y + angle.sin() * 8.0 * s;
        draw_ellipse(sx, sy, 6.0 * s, 4.0 * s, 0.0, stone_color);
    }

    draw_rectangle(pos.x - 12.0 * s, pos.y - 3.0 * s, 24.0 * s, 6.0 * s, log_color);
    draw_rectangle(pos.x - 8.0 * s, pos.y - 6.0 * s, 16.0 * s, 5.0 * s, log_color);

    let fire_pos = vec2(pos.x, pos.y - 8.0 * s);
    let fire_size = vec2(22.0 * s, 34.0 * s);
    fire::draw_fire(
        fire::Fire::new(fire_pos, fire_size),
        time,
    );
}

fn draw_crow_base(pos: Vec2, time: f32) {
    let s = scale::MODEL_SCALE;
    let bottom_w = 280.0 * s;
    let top_w = 170.0 * s;
    let height = 140.0 * s;
    let base_y = pos.y;
    let top_y = pos.y - height;

    let bl = vec2(pos.x - bottom_w * 0.5, base_y);
    let br = vec2(pos.x + bottom_w * 0.5, base_y);
    let tl = vec2(pos.x - top_w * 0.5, top_y);
    let tr = vec2(pos.x + top_w * 0.5, top_y);
    let top_mid = lerp_point(tl, tr, 0.5);
    let bottom_mid = lerp_point(bl, br, 0.5);

    let shadow = Color::new(0.0, 0.0, 0.0, 0.35);
    draw_ellipse(
        pos.x,
        pos.y + 4.0 * s,
        bottom_w * 0.55,
        height * 0.22,
        0.0,
        shadow,
    );

    let left_color = Color::new(0.08, 0.08, 0.10, 1.0);
    let right_color = Color::new(0.12, 0.12, 0.15, 1.0);

    draw_triangle(tl, top_mid, bottom_mid, left_color);
    draw_triangle(tl, bottom_mid, bl, left_color);
    draw_triangle(top_mid, tr, br, right_color);
    draw_triangle(top_mid, br, bottom_mid, right_color);

    let inner_tl = trapezoid_point(tl, tr, bl, br, 0.08, 0.12);
    let inner_tr = trapezoid_point(tl, tr, bl, br, 0.92, 0.12);
    let inner_br = trapezoid_point(tl, tr, bl, br, 0.92, 0.88);
    let inner_bl = trapezoid_point(tl, tr, bl, br, 0.08, 0.88);
    let inner_fill = Color::new(0.06, 0.06, 0.08, 1.0);
    draw_triangle(inner_tl, inner_tr, inner_br, inner_fill);
    draw_triangle(inner_tl, inner_br, inner_bl, inner_fill);

    let edge = Color::new(0.25, 0.28, 0.34, 0.8);
    let edge_w = 1.4 * s;
    draw_line(tl.x, tl.y, tr.x, tr.y, edge_w, edge);
    draw_line(tr.x, tr.y, br.x, br.y, edge_w, edge);
    draw_line(br.x, br.y, bl.x, bl.y, edge_w, edge);
    draw_line(bl.x, bl.y, tl.x, tl.y, edge_w, edge);

    let inner_edge = Color::new(0.18, 0.2, 0.28, 0.8);
    let inner_w = 1.1 * s;
    draw_line(inner_tl.x, inner_tl.y, inner_tr.x, inner_tr.y, inner_w, inner_edge);
    draw_line(inner_tr.x, inner_tr.y, inner_br.x, inner_br.y, inner_w, inner_edge);
    draw_line(inner_br.x, inner_br.y, inner_bl.x, inner_bl.y, inner_w, inner_edge);
    draw_line(inner_bl.x, inner_bl.y, inner_tl.x, inner_tl.y, inner_w, inner_edge);

    draw_crow_base_flame(tl, tr, bl, br, time);
    draw_crow_base_runes(inner_tl, inner_tr, inner_bl, inner_br, time);
}

fn draw_crow_base_flame(tl: Vec2, tr: Vec2, bl: Vec2, br: Vec2, time: f32) {
    let center = trapezoid_point(tl, tr, bl, br, 0.5, 0.58);
    let s = scale::MODEL_SCALE;
    let width = 28.0 * s;
    let height = 36.0 * s;
    let flicker = (time * 3.2).sin() * 1.8 * s;

    let glow = Color::new(1.0, 0.32, 0.05, 0.35);
    draw_circle(center.x, center.y + 8.0 * s, 22.0 * s, glow);

    draw_triangle(
        vec2(center.x, center.y - height * 0.55 - flicker),
        vec2(center.x - width * 0.5, center.y + height * 0.3),
        vec2(center.x + width * 0.5, center.y + height * 0.3),
        Color::new(0.95, 0.38, 0.12, 0.9),
    );

    draw_triangle(
        vec2(center.x, center.y - height * 0.25 - flicker * 0.5),
        vec2(center.x - width * 0.3, center.y + height * 0.2),
        vec2(center.x + width * 0.3, center.y + height * 0.2),
        Color::new(1.0, 0.8, 0.3, 0.9),
    );
}

fn draw_crow(pos: Vec2, time: f32) {
    let s = scale::MODEL_SCALE;
    let base_height = 140.0 * s;
    let perch = vec2(pos.x, pos.y - base_height - 16.0 * s);

    let metal = Color::new(1.0, 0.45, 0.12, 1.0);
    let metal_dim = Color::new(0.7, 0.25, 0.08, 0.9);
    let glow = Color::new(1.0, 0.35, 0.15, 0.28);

    let body_center = perch + vec2(0.0, -18.0 * s);
    let body_top = body_center + vec2(0.0, -22.0 * s);
    let body_bottom = body_center + vec2(0.0, 26.0 * s);
    let head_radius = 6.0 * s;
    let head = body_top + vec2(8.0 * s, -16.0 * s);
    let beak = head + vec2(18.0 * s, -2.0 * s);
    let tail = body_bottom + vec2(-10.0 * s, 16.0 * s);

    let mast_left = vec2(head.x - 6.0 * s, head.y + head_radius * 0.2);
    let mast_right = vec2(head.x + 6.0 * s, head.y + head_radius * 0.2);
    draw_crow_glow_line(vec2(mast_left.x, perch.y), mast_left, 3.0 * s, metal, glow);
    draw_crow_glow_line(vec2(mast_right.x, perch.y), mast_right, 3.0 * s, metal, glow);

    draw_triangle(
        body_top + vec2(-10.0 * s, 4.0 * s),
        body_top + vec2(12.0 * s, 4.0 * s),
        body_bottom + vec2(0.0, -2.0 * s),
        metal_dim,
    );
    draw_crow_glow_line(body_top + vec2(-10.0 * s, 4.0 * s), body_top + vec2(12.0 * s, 4.0 * s), 2.2 * s, metal, glow);
    draw_crow_glow_line(body_top + vec2(12.0 * s, 4.0 * s), body_bottom + vec2(0.0, -2.0 * s), 2.2 * s, metal, glow);
    draw_crow_glow_line(body_bottom + vec2(0.0, -2.0 * s), body_top + vec2(-10.0 * s, 4.0 * s), 2.2 * s, metal, glow);

    draw_circle(head.x, head.y, head_radius, metal_dim);
    draw_crow_glow_line(head, beak, 2.0 * s, metal, glow);

    draw_crow_glow_line(body_bottom, tail, 2.0 * s, metal, glow);
    draw_crow_glow_line(body_bottom + vec2(4.0 * s, 6.0 * s), tail + vec2(10.0 * s, 10.0 * s), 2.0 * s, metal, glow);

    let left_upper = [
        body_center + vec2(-10.0 * s, -8.0 * s),
        body_center + vec2(-60.0 * s, -24.0 * s),
        body_center + vec2(-120.0 * s, -10.0 * s),
        body_center + vec2(-180.0 * s, 8.0 * s),
        body_center + vec2(-220.0 * s, -6.0 * s),
    ];
    let left_lower = [
        body_center + vec2(-6.0 * s, 18.0 * s),
        body_center + vec2(-50.0 * s, 8.0 * s),
        body_center + vec2(-110.0 * s, 22.0 * s),
        body_center + vec2(-170.0 * s, 26.0 * s),
        body_center + vec2(-210.0 * s, 14.0 * s),
    ];

    draw_wing_wire(&left_upper, &left_lower, metal, glow, 2.0 * s);

    let right_upper = [
        body_center + vec2(10.0 * s, -8.0 * s),
        body_center + vec2(60.0 * s, -24.0 * s),
        body_center + vec2(120.0 * s, -10.0 * s),
        body_center + vec2(180.0 * s, 8.0 * s),
        body_center + vec2(220.0 * s, -6.0 * s),
    ];
    let right_lower = [
        body_center + vec2(6.0 * s, 18.0 * s),
        body_center + vec2(50.0 * s, 8.0 * s),
        body_center + vec2(110.0 * s, 22.0 * s),
        body_center + vec2(170.0 * s, 26.0 * s),
        body_center + vec2(210.0 * s, 14.0 * s),
    ];

    draw_wing_wire(&right_upper, &right_lower, metal, glow, 2.0 * s);

    let fire_pos = beak + vec2(2.0 * s, 0.0);
    let mut flame = fire::Fire::new(fire_pos, vec2(26.0 * s, 52.0 * s));
    flame.angle = -std::f32::consts::FRAC_PI_4;
    flame.intensity = 1.2;
    fire::draw_fire(flame, time);
}

fn draw_wing_wire(upper: &[Vec2; 5], lower: &[Vec2; 5], core: Color, glow: Color, width: f32) {
    for i in 0..upper.len() - 1 {
        draw_crow_glow_line(upper[i], upper[i + 1], width, core, glow);
        draw_crow_glow_line(lower[i], lower[i + 1], width, core, glow);
        draw_crow_glow_line(upper[i], lower[i], width, core, glow);
    }
    draw_crow_glow_line(*upper.last().unwrap(), *lower.last().unwrap(), width, core, glow);
}

fn draw_crow_glow_line(a: Vec2, b: Vec2, width: f32, core: Color, glow: Color) {
    draw_line(a.x, a.y, b.x, b.y, width * 2.2, glow);
    draw_line(a.x, a.y, b.x, b.y, width, core);
}

fn draw_crow_base_runes(tl: Vec2, tr: Vec2, bl: Vec2, br: Vec2, time: f32) {
    let rows = [
        (0.32, 7, 0.55, 1usize),
        (0.72, 10, 0.5, 5usize),
    ];

    for (v, count, size_scale, seed) in rows {
        draw_rune_row(tl, tr, bl, br, time, v, count, size_scale, seed);
    }

    let left = trapezoid_point(tl, tr, bl, br, 0.12, 0.68);
    let right = trapezoid_point(tl, tr, bl, br, 0.88, 0.68);
    let side_size = vec2(14.0 * scale::MODEL_SCALE, 20.0 * scale::MODEL_SCALE);
    draw_rune_glyph(6, left, side_size, time, 2.4);
    draw_rune_glyph(3, right, side_size, time, 4.1);
}

fn draw_rune_row(
    tl: Vec2,
    tr: Vec2,
    bl: Vec2,
    br: Vec2,
    time: f32,
    v: f32,
    count: usize,
    size_scale: f32,
    seed: usize,
) {
    let left = lerp_point(tl, bl, v);
    let right = lerp_point(tr, br, v);
    let row_w = right.x - left.x;
    let step = row_w / count as f32;
    let glyph_w = step * size_scale;
    let glyph_h = glyph_w * 1.25;
    let base_y = left.y;

    for i in 0..count {
        let center = vec2(left.x + step * (i as f32 + 0.5), base_y);
        let phase = seed as f32 * 0.9 + i as f32 * 0.65 + v * 4.2;
        draw_rune_glyph((i + seed) % RUNE_STROKES.len(), center, vec2(glyph_w, glyph_h), time, phase);
    }
}

fn draw_rune_glyph(index: usize, center: Vec2, size: Vec2, time: f32, phase: f32) {
    let intensity = 0.55 + (time * 2.1 + phase).sin() * 0.25 + (time * 0.6 + phase).sin() * 0.2;
    let pulse = intensity.clamp(0.2, 0.95);
    let base = Color::new(0.92, 0.25, 1.0, 1.0);
    let alt = Color::new(0.2, 0.9, 1.0, 1.0);
    let mix = ((time * 0.45 + phase * 0.7).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let color = lerp_color(base, alt, mix);

    let line_w = (size.y * 0.06).max(0.6 * scale::MODEL_SCALE);
    let glow_w = line_w * 3.2;
    let core = Color::new(color.r, color.g, color.b, 0.7 + pulse * 0.3);
    let glow = Color::new(color.r, color.g, color.b, 0.18 + pulse * 0.35);

    let strokes = &RUNE_STROKES[index % RUNE_STROKES.len()];
    let stroke_count = RUNE_COUNTS[index % RUNE_COUNTS.len()];
    for stroke in strokes.iter().take(stroke_count) {
        let a = vec2(
            center.x + (stroke.0 - 0.5) * size.x,
            center.y + (stroke.1 - 0.5) * size.y,
        );
        let b = vec2(
            center.x + (stroke.2 - 0.5) * size.x,
            center.y + (stroke.3 - 0.5) * size.y,
        );
        draw_line(a.x, a.y, b.x, b.y, glow_w, glow);
        draw_line(a.x, a.y, b.x, b.y, line_w, core);
    }

    if index % 3 == 0 {
        draw_circle(center.x, center.y + size.y * 0.2, size.x * 0.08, glow);
    }
}

type RuneStroke = (f32, f32, f32, f32);

const RUNE_STROKES: [[RuneStroke; 5]; 8] = [
    [
        (0.2, 0.1, 0.2, 0.9),
        (0.2, 0.1, 0.85, 0.4),
        (0.2, 0.5, 0.85, 0.9),
        (0.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 0.0, 0.0),
    ],
    [
        (0.5, 0.1, 0.5, 0.9),
        (0.5, 0.1, 0.2, 0.4),
        (0.5, 0.1, 0.8, 0.4),
        (0.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 0.0, 0.0),
    ],
    [
        (0.2, 0.2, 0.8, 0.2),
        (0.2, 0.2, 0.2, 0.8),
        (0.8, 0.2, 0.8, 0.8),
        (0.2, 0.8, 0.8, 0.8),
        (0.0, 0.0, 0.0, 0.0),
    ],
    [
        (0.2, 0.2, 0.8, 0.8),
        (0.8, 0.2, 0.2, 0.8),
        (0.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 0.0, 0.0),
    ],
    [
        (0.2, 0.2, 0.8, 0.2),
        (0.8, 0.2, 0.2, 0.5),
        (0.2, 0.5, 0.8, 0.8),
        (0.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 0.0, 0.0),
    ],
    [
        (0.2, 0.1, 0.2, 0.9),
        (0.8, 0.1, 0.8, 0.9),
        (0.2, 0.3, 0.8, 0.3),
        (0.2, 0.6, 0.8, 0.6),
        (0.0, 0.0, 0.0, 0.0),
    ],
    [
        (0.2, 0.15, 0.2, 0.9),
        (0.2, 0.55, 0.8, 0.25),
        (0.2, 0.55, 0.8, 0.85),
        (0.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 0.0, 0.0),
    ],
    [
        (0.2, 0.2, 0.2, 0.8),
        (0.8, 0.2, 0.8, 0.8),
        (0.2, 0.8, 0.8, 0.8),
        (0.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 0.0, 0.0),
    ],
];

const RUNE_COUNTS: [usize; 8] = [3, 3, 4, 2, 3, 4, 3, 3];

fn draw_tree(pos: Vec2, scale: f32) {
    let trunk_color = Color::new(0.36, 0.25, 0.20, 1.0);
    let foliage_dark = Color::new(0.13, 0.55, 0.13, 1.0);
    let foliage_light = Color::new(0.18, 0.63, 0.18, 1.0);
    let s = scale::MODEL_SCALE * scale;

    draw_rectangle(
        pos.x - 4.0 * s,
        pos.y - 16.0 * s,
        8.0 * s,
        22.0 * s,
        trunk_color,
    );

    draw_triangle(
        vec2(pos.x, pos.y - 42.0 * s),
        vec2(pos.x - 20.0 * s, pos.y - 16.0 * s),
        vec2(pos.x + 20.0 * s, pos.y - 16.0 * s),
        foliage_dark,
    );

    draw_triangle(
        vec2(pos.x, pos.y - 32.0 * s),
        vec2(pos.x - 16.0 * s, pos.y - 12.0 * s),
        vec2(pos.x + 16.0 * s, pos.y - 12.0 * s),
        foliage_light,
    );
}

fn draw_geodesic_dome(center: Vec2, time: f32, decorations: &[DomeDecoration]) {
    let radius = DOME_RADIUS;
    let height = DOME_HEIGHT;
    let squash = 0.4;
    let line_w = 1.0 * scale::MODEL_SCALE;
    let rim = 5.0 * scale::MODEL_SCALE;

    #[derive(Clone, Copy)]
    struct DomeVertex {
        pos: Vec2,
        depth: f32,
    }

    let make_ring = |count: usize, r_frac: f32, h_frac: f32, offset: f32| -> Vec<DomeVertex> {
        let r = radius * r_frac;
        let ring_squash = squash * (1.0 - h_frac * 0.4);
        let cy = center.y - height * h_frac;
        let mut verts = Vec::with_capacity(count);
        for i in 0..count {
            let a = (i as f32 / count as f32) * std::f32::consts::TAU + offset;
            verts.push(DomeVertex {
                pos: vec2(center.x + a.cos() * r, cy + a.sin() * r * ring_squash),
                depth: a.sin(),
            });
        }
        verts
    };

    let base = make_ring(10, 1.0, 0.0, 0.0);
    let mid = make_ring(8, 0.72, 0.38, std::f32::consts::PI / 8.0);
    let top = make_ring(5, 0.38, 0.72, std::f32::consts::PI / 10.0);
    let apex = DomeVertex {
        pos: vec2(center.x, center.y - height),
        depth: 0.0,
    };

    let edge_color = |d: f32| Color::new(160.0 / 255.0, 210.0 / 255.0, 250.0 / 255.0, d);

    let mut edge = |v1: DomeVertex, v2: DomeVertex| {
        let d = (v1.depth + v2.depth) * 0.5;
        let alpha = 0.2 + d.max(0.0) * 0.4;
        draw_line(
            v1.pos.x,
            v1.pos.y,
            v2.pos.x,
            v2.pos.y,
            line_w,
            edge_color(alpha),
        );
    };

    let ring = |verts: &[DomeVertex], edge_fn: &mut dyn FnMut(DomeVertex, DomeVertex)| {
        for i in 0..verts.len() {
            let next = (i + 1) % verts.len();
            edge_fn(verts[i], verts[next]);
        }
    };

    let connect =
        |lower: &[DomeVertex], upper: &[DomeVertex], edge_fn: &mut dyn FnMut(DomeVertex, DomeVertex)| {
            for i in 0..upper.len() {
                let li = ((i as f32 / upper.len() as f32) * lower.len() as f32).round() as usize
                    % lower.len();
                edge_fn(upper[i], lower[li]);
                edge_fn(upper[i], lower[(li + 1) % lower.len()]);
            }
        };

    ring(&base, &mut edge);
    ring(&mid, &mut edge);
    ring(&top, &mut edge);
    connect(&base, &mid, &mut edge);
    connect(&mid, &top, &mut edge);
    for v in &top {
        edge(*v, apex);
    }

    draw_ellipse_lines(
        center.x,
        center.y + 2.0 * scale::MODEL_SCALE,
        radius + rim,
        (radius + rim) * squash,
        0.0,
        line_w,
        Color::new(160.0 / 255.0, 210.0 / 255.0, 250.0 / 255.0, 0.3),
    );

    if decorations.contains(&DomeDecoration::Crystal) {
        draw_big_red_crystal(vec2(center.x, center.y - height * 0.35), time);
    }
}

fn draw_big_red_crystal(center: Vec2, time: f32) {
    let pulse = ((time * 1.1).sin() + 1.0) * 0.5;
    let glow_alpha = 0.18 + pulse * 0.12;
    let s = scale::MODEL_SCALE * CRYSTAL_SCALE;

    let tip_h = 22.0 * s;
    let body_h = 56.0 * s;
    let half_w = 14.0 * s;
    let top_tip = center.y - (body_h * 0.5 + tip_h);
    let body_top = center.y - body_h * 0.5;
    let body_bottom = center.y + body_h * 0.5;
    let bottom_tip = center.y + (body_h * 0.5 + tip_h);

    draw_circle(
        center.x,
        center.y + 4.0 * s,
        42.0 * s,
        Color::new(1.0, 0.05, 0.05, glow_alpha),
    );
    draw_circle(
        center.x,
        center.y + 4.0 * s,
        26.0 * s,
        Color::new(0.7, 0.0, 0.0, glow_alpha * 0.8),
    );

    #[derive(Clone, Copy)]
    struct Edge {
        x: f32,
        depth: f32,
    }

    let rot = time * 0.5;
    let edge_count = 6usize;
    let mut edges: Vec<Edge> = Vec::with_capacity(edge_count);
    for i in 0..edge_count {
        let a = rot + (i as f32 / edge_count as f32) * std::f32::consts::TAU;
        edges.push(Edge {
            x: a.sin() * half_w,
            depth: a.cos(),
        });
    }

    #[derive(Clone, Copy)]
    struct Face {
        i1: usize,
        i2: usize,
        depth: f32,
        string_face: bool,
    }

    let mut faces: Vec<Face> = Vec::with_capacity(edge_count);
    for i in 0..edge_count {
        let i2 = (i + 1) % edge_count;
        let depth = (edges[i].depth + edges[i2].depth) * 0.5;
        faces.push(Face {
            i1: i,
            i2,
            depth,
            string_face: i == 0,
        });
    }

    faces.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap());

    for face in faces {
        let e1 = edges[face.i1];
        let e2 = edges[face.i2];
        let brightness = face.depth * 0.35 + 0.55 + pulse * 0.1;
        let r = (80.0 + brightness * 175.0).min(255.0) / 255.0;
        let g = (brightness * 25.0).min(60.0) / 255.0;
        let b = (brightness * 30.0).min(80.0) / 255.0;
        let alpha = if face.string_face { 0.85 } else { 0.95 };
        let color = Color::new(r, g, b, alpha);

        let top1 = vec2(center.x + e1.x, body_top);
        let top2 = vec2(center.x + e2.x, body_top);
        let bottom1 = vec2(center.x + e1.x, body_bottom);
        let bottom2 = vec2(center.x + e2.x, body_bottom);

        draw_triangle(top1, top2, bottom2, color);
        draw_triangle(top1, bottom2, bottom1, color);

        draw_triangle(
            vec2(center.x, top_tip),
            top1,
            top2,
            Color::new(r * 0.95, g * 0.7, b * 0.7, alpha),
        );
        draw_triangle(
            vec2(center.x, bottom_tip),
            bottom2,
            bottom1,
            Color::new(r * 0.75, g * 0.5, b * 0.5, alpha),
        );

        if face.string_face && face.depth > 0.1 {
            draw_crystal_strings(top1, top2, bottom1, bottom2, pulse);
        }
    }

    let highlight = 0.25 + pulse * 0.2;
    draw_triangle(
        vec2(center.x - half_w * 0.15, body_top + 6.0 * s),
        vec2(center.x + half_w * 0.55, body_top + body_h * 0.45),
        vec2(center.x - half_w * 0.05, body_bottom - 6.0 * s),
        Color::new(1.0, 0.65, 0.65, highlight),
    );
}

fn draw_crystal_strings(top1: Vec2, top2: Vec2, bottom1: Vec2, bottom2: Vec2, pulse: f32) {
    let count = 18;
    let glow = Color::new(0.95, 0.9, 0.6, 0.25 + pulse * 0.2);
    let s = scale::MODEL_SCALE;
    for i in 0..count {
        let t = i as f32 / (count - 1) as f32;
        let top = lerp_point(top1, top2, t);
        let bottom = lerp_point(bottom1, bottom2, 1.0 - t);
        draw_line(
            top.x,
            top.y + 2.0 * s,
            bottom.x,
            bottom.y - 2.0 * s,
            1.0 * s,
            glow,
        );
    }
}

fn lerp_point(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a + (b - a) * t
}

fn trapezoid_point(tl: Vec2, tr: Vec2, bl: Vec2, br: Vec2, u: f32, v: f32) -> Vec2 {
    let top = lerp_point(tl, tr, u);
    let bottom = lerp_point(bl, br, u);
    lerp_point(top, bottom, v)
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn draw_rotated_rect(center: Vec2, size: Vec2, rotation: f32, color: Color) {
    draw_rectangle_ex(
        center.x,
        center.y,
        size.x,
        size.y,
        DrawRectangleParams {
            offset: vec2(0.5, 0.5),
            rotation,
            color,
        },
    );
}

fn rotate_point(point: Vec2, origin: Vec2, angle: f32) -> Vec2 {
    let translated = point - origin;
    let rotated = vec2(
        translated.x * angle.cos() - translated.y * angle.sin(),
        translated.x * angle.sin() + translated.y * angle.cos(),
    );
    rotated + origin
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player;

    #[test]
    fn spawn_scenery_has_expected_counts() {
        let field = Rect::new(0.0, 0.0, 10000.0, 7000.0);
        let items = spawn_scenery(field);

        let tents = items.iter().filter(|i| i.kind == SceneryKind::Tent).count();
        let chairs = items.iter().filter(|i| i.kind == SceneryKind::Chair).count();
        let campfires = items
            .iter()
            .filter(|i| i.kind == SceneryKind::Campfire)
            .count();
        let crow_bases = items.iter().filter(|i| i.kind == SceneryKind::CrowBase).count();
        let crows = items.iter().filter(|i| i.kind == SceneryKind::Crow).count();
        let trees = items.iter().filter(|i| i.kind == SceneryKind::Tree).count();
        let domes = items.iter().filter(|i| i.kind == SceneryKind::Dome).count();
        let domes_with_crystal = items
            .iter()
            .filter(|i| i.kind == SceneryKind::Dome)
            .filter(|i| i.decorations.contains(&DomeDecoration::Crystal))
            .count();

        let row1 = line_points(T3MPCAMP_ROW1_START, T3MPCAMP_ROW1_END, T3MPCAMP_TENT_SPACING);
        let row2 = line_points(T3MPCAMP_ROW2_START, T3MPCAMP_ROW2_END, T3MPCAMP_TENT_SPACING);
        let expected_tents = 5 + row1.len() + row2.len();
        assert_eq!(tents, expected_tents);
        assert_eq!(chairs, 5);
        assert_eq!(campfires, 3);
        assert_eq!(crow_bases, 1);
        assert_eq!(crows, 1);
        assert_eq!(trees, 5);
        assert_eq!(domes, 3);
        assert_eq!(domes_with_crystal, 2);
    }

    #[test]
    fn spawn_scenery_within_field() {
        let field = Rect::new(0.0, 0.0, 10000.0, 7000.0);
        let items = spawn_scenery(field);

        for item in items {
            assert!(item.pos.x >= field.x && item.pos.x <= field.x + field.w);
            assert!(item.pos.y >= field.y && item.pos.y <= field.y + field.h);
        }
    }

    #[test]
    fn tree_scale_in_range() {
        let scale = tree_scale_from_pos(vec2(120.0, 260.0));
        assert!(scale >= 0.8);
        assert!(scale <= 1.2);
    }

    #[test]
    fn dome_large_enough_for_multiple_players() {
        let diameter = DOME_RADIUS * 2.0;
        assert!(diameter >= player::PLAYER_WIDTH * 4.0);
        assert!(DOME_HEIGHT >= player::PLAYER_HEIGHT * 2.0);
    }
}
