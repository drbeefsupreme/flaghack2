use macroquad::prelude::*;

use crate::constants;

pub fn draw_hud(
    flag_count: u32,
    speed: f32,
    player_pos: Vec2,
    total_flags: u32,
    flagic: u8,
) {
    let y = screen_height() - constants::HUD_HEIGHT;
    draw_rectangle(0.0, y, screen_width(), constants::HUD_HEIGHT, BLACK);

    let text = format!("Flags: {}", flag_count);
    draw_text(&text, 16.0, y + 32.0, 26.0, constants::ACCENT);

    let speed_text = format!("Speed: {:.1}px/s", speed);
    draw_text(&speed_text, 180.0, y + 32.0, 20.0, constants::ACCENT);

    let total_text = format!("Total: {}", total_flags);
    draw_text(&total_text, 360.0, y + 32.0, 20.0, constants::ACCENT);

    let flagic_text = format!("Flagic: {}", flagic);
    draw_text(&flagic_text, 480.0, y + 32.0, 20.0, constants::ACCENT);

    let coords = format_player_coords(player_pos);
    let metrics = measure_text(&coords, None, 20, 1.0);
    let x = screen_width() - metrics.width - 16.0;
    draw_text(&coords, x, y + 32.0, 20.0, constants::ACCENT);
}

fn format_player_coords(pos: Vec2) -> String {
    format!("X: {:.0}  Y: {:.0}", pos.x, pos.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_player_coords_rounds_to_whole_numbers() {
        let text = format_player_coords(vec2(12.4, 13.6));
        assert_eq!(text, "X: 12  Y: 14");
    }
}
