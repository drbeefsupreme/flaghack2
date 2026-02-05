use macroquad::prelude::*;
use macroquad::rand::gen_range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneryKind {
    Tree,
    Tent,
    Chair,
    Campfire,
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
const DOME_PADDING: f32 = 120.0;
pub const DOME_RADIUS: f32 = 100.0;
pub const DOME_HEIGHT: f32 = 100.0;

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
            SceneryKind::Campfire => draw_campfire(item.pos, time),
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

fn draw_tent(pos: Vec2, variant: u8) {
    let color = TENT_COLORS[variant as usize % TENT_COLORS.len()];
    let size = 28.0;

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
        1.5,
        Color::new(0.0, 0.0, 0.0, 0.35),
    );
}

fn draw_chair(pos: Vec2, rotation: f32) {
    let seat_color = Color::new(0.29, 0.33, 0.39, 1.0);
    let back_color = Color::new(0.18, 0.22, 0.28, 1.0);
    let leg_color = Color::new(0.12, 0.16, 0.20, 1.0);

    let seat = vec2(20.0, 12.0);
    let back = vec2(20.0, 10.0);

    draw_rotated_rect(pos, seat, rotation, seat_color);
    draw_rotated_rect(pos - vec2(0.0, seat.y * 0.7), back, rotation, back_color);

    let leg_offset = vec2(6.0, 6.0);
    let leg_len = 8.0;
    let left = rotate_point(pos + vec2(-leg_offset.x, leg_offset.y), pos, rotation);
    let right = rotate_point(pos + vec2(leg_offset.x, leg_offset.y), pos, rotation);

    draw_line(left.x, left.y, left.x - 2.0, left.y + leg_len, 2.0, leg_color);
    draw_line(right.x, right.y, right.x + 2.0, right.y + leg_len, 2.0, leg_color);
}

fn draw_campfire(pos: Vec2, time: f32) {
    let stone_color = Color::new(0.33, 0.33, 0.33, 1.0);
    let log_color = Color::new(0.36, 0.25, 0.20, 1.0);

    let stone_angles: [f32; 8] = [0.0, 0.8, 1.6, 2.4, 3.2, 4.0, 4.8, 5.6];
    for angle in stone_angles {
        let sx = pos.x + angle.cos() * 16.0;
        let sy = pos.y + angle.sin() * 8.0;
        draw_ellipse(sx, sy, 6.0, 4.0, 0.0, stone_color);
    }

    draw_rectangle(pos.x - 12.0, pos.y - 3.0, 24.0, 6.0, log_color);
    draw_rectangle(pos.x - 8.0, pos.y - 6.0, 16.0, 5.0, log_color);

    let flicker = (time * 3.0).sin() * 3.0;

    draw_triangle(
        vec2(pos.x, pos.y - 26.0 - flicker),
        vec2(pos.x - 9.0, pos.y - 6.0),
        vec2(pos.x + 9.0, pos.y - 6.0),
        Color::new(1.0, 0.42, 0.21, 1.0),
    );

    draw_triangle(
        vec2(pos.x, pos.y - 18.0 - flicker * 0.5),
        vec2(pos.x - 5.0, pos.y - 6.0),
        vec2(pos.x + 5.0, pos.y - 6.0),
        Color::new(1.0, 0.85, 0.24, 1.0),
    );
}

fn draw_tree(pos: Vec2, scale: f32) {
    let trunk_color = Color::new(0.36, 0.25, 0.20, 1.0);
    let foliage_dark = Color::new(0.13, 0.55, 0.13, 1.0);
    let foliage_light = Color::new(0.18, 0.63, 0.18, 1.0);

    draw_rectangle(
        pos.x - 4.0 * scale,
        pos.y - 16.0 * scale,
        8.0 * scale,
        22.0 * scale,
        trunk_color,
    );

    draw_triangle(
        vec2(pos.x, pos.y - 42.0 * scale),
        vec2(pos.x - 20.0 * scale, pos.y - 16.0 * scale),
        vec2(pos.x + 20.0 * scale, pos.y - 16.0 * scale),
        foliage_dark,
    );

    draw_triangle(
        vec2(pos.x, pos.y - 32.0 * scale),
        vec2(pos.x - 16.0 * scale, pos.y - 12.0 * scale),
        vec2(pos.x + 16.0 * scale, pos.y - 12.0 * scale),
        foliage_light,
    );
}

fn draw_geodesic_dome(center: Vec2, time: f32, decorations: &[DomeDecoration]) {
    let radius = DOME_RADIUS;
    let height = DOME_HEIGHT;
    let squash = 0.4;

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
        draw_line(v1.pos.x, v1.pos.y, v2.pos.x, v2.pos.y, 1.0, edge_color(alpha));
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
        center.y + 2.0,
        radius + 5.0,
        (radius + 5.0) * squash,
        0.0,
        1.0,
        Color::new(160.0 / 255.0, 210.0 / 255.0, 250.0 / 255.0, 0.3),
    );

    if decorations.contains(&DomeDecoration::Crystal) {
        draw_big_red_crystal(vec2(center.x, center.y - height * 0.35), time);
    }
}

fn draw_big_red_crystal(center: Vec2, time: f32) {
    let pulse = ((time * 1.1).sin() + 1.0) * 0.5;
    let glow_alpha = 0.18 + pulse * 0.12;

    let tip_h = 22.0;
    let body_h = 56.0;
    let half_w = 14.0;
    let top_tip = center.y - (body_h * 0.5 + tip_h);
    let body_top = center.y - body_h * 0.5;
    let body_bottom = center.y + body_h * 0.5;
    let bottom_tip = center.y + (body_h * 0.5 + tip_h);

    draw_circle(
        center.x,
        center.y + 4.0,
        42.0,
        Color::new(1.0, 0.05, 0.05, glow_alpha),
    );
    draw_circle(
        center.x,
        center.y + 4.0,
        26.0,
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
        vec2(center.x - half_w * 0.15, body_top + 6.0),
        vec2(center.x + half_w * 0.55, body_top + body_h * 0.45),
        vec2(center.x - half_w * 0.05, body_bottom - 6.0),
        Color::new(1.0, 0.65, 0.65, highlight),
    );
}

fn draw_crystal_strings(top1: Vec2, top2: Vec2, bottom1: Vec2, bottom2: Vec2, pulse: f32) {
    let count = 18;
    let glow = Color::new(0.95, 0.9, 0.6, 0.25 + pulse * 0.2);
    for i in 0..count {
        let t = i as f32 / (count - 1) as f32;
        let top = lerp_point(top1, top2, t);
        let bottom = lerp_point(bottom1, bottom2, 1.0 - t);
        draw_line(top.x, top.y + 2.0, bottom.x, bottom.y - 2.0, 1.0, glow);
    }
}

fn lerp_point(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a + (b - a) * t
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
        let field = Rect::new(0.0, 0.0, 960.0, 480.0);
        let items = spawn_scenery(field);

        let tents = items.iter().filter(|i| i.kind == SceneryKind::Tent).count();
        let chairs = items.iter().filter(|i| i.kind == SceneryKind::Chair).count();
        let campfires = items
            .iter()
            .filter(|i| i.kind == SceneryKind::Campfire)
            .count();
        let trees = items.iter().filter(|i| i.kind == SceneryKind::Tree).count();
        let domes = items.iter().filter(|i| i.kind == SceneryKind::Dome).count();
        let domes_with_crystal = items
            .iter()
            .filter(|i| i.kind == SceneryKind::Dome)
            .filter(|i| i.decorations.contains(&DomeDecoration::Crystal))
            .count();

        assert_eq!(tents, 5);
        assert_eq!(chairs, 5);
        assert_eq!(campfires, 2);
        assert_eq!(trees, 5);
        assert_eq!(domes, 2);
        assert_eq!(domes_with_crystal, 1);
    }

    #[test]
    fn spawn_scenery_within_field() {
        let field = Rect::new(0.0, 0.0, 960.0, 480.0);
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
