use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneryKind {
    Tree,
    Tent,
    Chair,
    Campfire,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneryItem {
    pub kind: SceneryKind,
    pub pos: Vec2,
    pub scale: f32,
    pub rotation: f32,
    pub variant: u8,
}

const BASE_W: f32 = 800.0;
const BASE_H: f32 = 600.0;

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

        assert_eq!(tents, 5);
        assert_eq!(chairs, 5);
        assert_eq!(campfires, 2);
        assert_eq!(trees, 5);
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
}
