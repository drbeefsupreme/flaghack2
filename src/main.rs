use macroquad::prelude::*;

const SCREEN_W: i32 = 960;
const SCREEN_H: i32 = 540;
const ACCENT: Color = Color::new(1.0, 0.9, 0.0, 1.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scene {
    Title,
    ClassSelect,
    Dungeon,
}

#[derive(Clone, Copy, Debug)]
struct Player {
    x: i32,
    y: i32,
}

struct Game {
    scene: Scene,
    player: Player,
    class_name: &'static str,
}

impl Game {
    fn new() -> Self {
        Self {
            scene: Scene::Title,
            player: Player { x: 1, y: 1 },
            class_name: "Vexillomancer",
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Flaghack2".to_string(),
        window_width: SCREEN_W,
        window_height: SCREEN_H,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();

    loop {
        if is_key_pressed(KeyCode::Q) {
            break;
        }

        clear_background(BLACK);

        match game.scene {
            Scene::Title => render_title(&mut game),
            Scene::ClassSelect => render_class_select(&mut game),
            Scene::Dungeon => render_dungeon(&mut game),
        }

        next_frame().await;
    }
}

fn render_title(game: &mut Game) {
    let title_size = 64.0;
    let subtitle_size = 28.0;

    draw_centered("FLAGHACK2", 140.0, title_size, ACCENT);
    draw_centered("Press any key", 240.0, subtitle_size, ACCENT);
    draw_centered("Q to quit", 280.0, 22.0, ACCENT);

    if get_last_key_pressed().is_some() {
        game.scene = Scene::ClassSelect;
    }
}

fn render_class_select(game: &mut Game) {
    draw_centered("Choose Your Class", 120.0, 44.0, ACCENT);

    let class_line = format!("> {} <", game.class_name);
    draw_centered(&class_line, 220.0, 32.0, ACCENT);

    draw_centered("Enter to begin", 300.0, 24.0, ACCENT);
    draw_centered("Esc to go back", 332.0, 20.0, ACCENT);
    draw_centered("Q to quit", 360.0, 20.0, ACCENT);

    if is_key_pressed(KeyCode::Escape) {
        game.scene = Scene::Title;
        return;
    }

    if is_key_pressed(KeyCode::Enter) {
        game.scene = Scene::Dungeon;
    }
}

fn render_dungeon(game: &mut Game) {
    draw_centered("Dungeon", 60.0, 44.0, ACCENT);
    draw_centered("Arrow keys to move", 110.0, 20.0, ACCENT);
    draw_centered("Esc to class select", 135.0, 20.0, ACCENT);
    draw_centered("Q to quit", 160.0, 20.0, ACCENT);

    handle_movement(game);

    let pos_text = format!("Player position: ({}, {})", game.player.x, game.player.y);
    draw_centered(&pos_text, 230.0, 24.0, ACCENT);

    if is_key_pressed(KeyCode::Escape) {
        game.scene = Scene::ClassSelect;
    }
}

fn handle_movement(game: &mut Game) {
    let mut dx = 0;
    let mut dy = 0;

    if is_key_pressed(KeyCode::Up) {
        dy = -1;
    } else if is_key_pressed(KeyCode::Down) {
        dy = 1;
    } else if is_key_pressed(KeyCode::Left) {
        dx = -1;
    } else if is_key_pressed(KeyCode::Right) {
        dx = 1;
    }

    if dx != 0 || dy != 0 {
        let new_x = (game.player.x + dx).clamp(0, 19);
        let new_y = (game.player.y + dy).clamp(0, 11);
        game.player.x = new_x;
        game.player.y = new_y;
    }
}

fn draw_centered(text: &str, y: f32, size: f32, color: Color) {
    let metrics = measure_text(text, None, size as u16, 1.0);
    let x = (SCREEN_W as f32 - metrics.width) * 0.5;
    draw_text(text, x, y, size, color);
}
