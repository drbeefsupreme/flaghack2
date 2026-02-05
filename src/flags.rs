use macroquad::prelude::{Rect, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flag {
    pub pos: Vec2,
}

pub fn field_rect(screen_w: f32, screen_h: f32, hud_height: f32) -> Rect {
    Rect::new(0.0, 0.0, screen_w, (screen_h - hud_height).max(0.0))
}

pub fn spawn_initial_flags(count: usize, field: Rect, padding: f32) -> Vec<Flag> {
    if count == 0 {
        return Vec::new();
    }

    let columns = (count as f32).sqrt().ceil() as usize;
    let rows = (count + columns - 1) / columns;

    let usable_w = (field.w - padding * 2.0).max(1.0);
    let usable_h = (field.h - padding * 2.0).max(1.0);

    let cell_w = usable_w / columns as f32;
    let cell_h = usable_h / rows as f32;

    let mut flags = Vec::with_capacity(count);
    for i in 0..count {
        let col = i % columns;
        let row = i / columns;

        let x = field.x + padding + cell_w * (col as f32 + 0.5);
        let y = field.y + padding + cell_h * (row as f32 + 0.5);

        flags.push(Flag { pos: Vec2::new(x, y) });
    }

    flags
}

pub fn try_pickup_flag(flags: &mut Vec<Flag>, origin: Vec2, radius: f32) -> bool {
    if let Some(index) = nearest_flag_index(flags, origin, radius) {
        flags.swap_remove(index);
        true
    } else {
        false
    }
}

pub fn try_place_flag(
    flags: &mut Vec<Flag>,
    inventory: &mut u32,
    origin: Vec2,
    offset: Vec2,
    field: Rect,
) -> bool {
    if *inventory == 0 {
        return false;
    }

    *inventory -= 1;

    let mut pos = origin + offset;
    pos.x = pos.x.clamp(field.x, field.x + field.w);
    pos.y = pos.y.clamp(field.y, field.y + field.h);

    flags.push(Flag { pos });
    true
}

pub fn flag_parts(base: Vec2, pole_height: f32, pole_width: f32, cloth_size: Vec2) -> (Rect, Rect) {
    let pole = Rect::new(
        base.x - pole_width * 0.5,
        base.y - pole_height,
        pole_width,
        pole_height,
    );

    let cloth = Rect::new(
        pole.x + pole.w,
        pole.y,
        cloth_size.x,
        cloth_size.y,
    );

    (pole, cloth)
}

fn nearest_flag_index(flags: &[Flag], origin: Vec2, radius: f32) -> Option<usize> {
    let mut best_index = None;
    let mut best_dist = radius * radius;

    for (i, flag) in flags.iter().enumerate() {
        let dist_sq = origin.distance_squared(flag.pos);
        if dist_sq <= best_dist {
            best_dist = dist_sq;
            best_index = Some(i);
        }
    }

    best_index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_initial_flags_returns_requested_count() {
        let field = Rect::new(0.0, 0.0, 400.0, 300.0);
        let flags = spawn_initial_flags(10, field, 20.0);
        assert_eq!(flags.len(), 10);
    }

    #[test]
    fn spawn_initial_flags_within_field() {
        let field = Rect::new(0.0, 0.0, 400.0, 300.0);
        let flags = spawn_initial_flags(6, field, 10.0);
        for flag in flags {
            assert!(flag.pos.x >= field.x && flag.pos.x <= field.x + field.w);
            assert!(flag.pos.y >= field.y && flag.pos.y <= field.y + field.h);
        }
    }

    #[test]
    fn try_pickup_flag_removes_nearest() {
        let mut flags = vec![
            Flag { pos: Vec2::new(10.0, 10.0) },
            Flag { pos: Vec2::new(30.0, 10.0) },
        ];
        let picked = try_pickup_flag(&mut flags, Vec2::new(12.0, 10.0), 10.0);
        assert!(picked);
        assert_eq!(flags.len(), 1);
        assert_eq!(flags[0].pos, Vec2::new(30.0, 10.0));
    }

    #[test]
    fn try_pickup_flag_fails_when_out_of_range() {
        let mut flags = vec![Flag { pos: Vec2::new(100.0, 100.0) }];
        let picked = try_pickup_flag(&mut flags, Vec2::new(0.0, 0.0), 10.0);
        assert!(!picked);
        assert_eq!(flags.len(), 1);
    }

    #[test]
    fn try_place_flag_consumes_inventory() {
        let mut flags = Vec::new();
        let mut inventory = 1;
        let field = Rect::new(0.0, 0.0, 200.0, 200.0);

        let placed = try_place_flag(
            &mut flags,
            &mut inventory,
            Vec2::new(50.0, 50.0),
            Vec2::new(10.0, 0.0),
            field,
        );

        assert!(placed);
        assert_eq!(inventory, 0);
        assert_eq!(flags.len(), 1);
    }

    #[test]
    fn try_place_flag_fails_without_inventory() {
        let mut flags = Vec::new();
        let mut inventory = 0;
        let field = Rect::new(0.0, 0.0, 200.0, 200.0);

        let placed = try_place_flag(
            &mut flags,
            &mut inventory,
            Vec2::new(50.0, 50.0),
            Vec2::new(10.0, 0.0),
            field,
        );

        assert!(!placed);
        assert_eq!(flags.len(), 0);
    }

    #[test]
    fn flag_parts_aligns_cloth_with_pole() {
        let base = Vec2::new(100.0, 100.0);
        let (pole, cloth) = flag_parts(base, 40.0, 4.0, Vec2::new(20.0, 12.0));

        assert!((cloth.y - pole.y).abs() < f32::EPSILON);
        assert!((cloth.x - (pole.x + pole.w)).abs() < f32::EPSILON);
    }
}
