use macroquad::prelude::*;

mod assets;
mod movement;

const SCREEN_W: i32 = 960;
const SCREEN_H: i32 = 540;
const ACCENT: Color = Color::new(1.0, 0.9, 0.0, 1.0);
const FIELD_GREEN: Color = Color::new(0.06, 0.35, 0.12, 1.0);
const PLAYER_SIZE: f32 = 26.0;
const PLAYER_SPEED: f32 = 220.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scene {
    Title,
    ClassSelect,
    Dungeon,
}

#[derive(Clone, Copy, Debug)]
struct Player {
    pos: Vec2,
}

struct Game {
    scene: Scene,
    player: Player,
    class_name: &'static str,
}

struct Assets {
    signifier_mark: Texture2D,
    signifier_size: Vec2,
}

impl Game {
    fn new() -> Self {
        Self {
            scene: Scene::Title,
            player: Player {
                pos: vec2(
                    SCREEN_W as f32 * 0.5 - PLAYER_SIZE * 0.5,
                    SCREEN_H as f32 * 0.55 - PLAYER_SIZE * 0.5,
                ),
            },
            class_name: "Vexillomancer",
        }
    }
}

impl Assets {
    fn load() -> Self {
        let raster = assets::load_png_rgba("assets/png/signifiersmark.png");

        let signifier_mark = Texture2D::from_rgba8(raster.width, raster.height, &raster.pixels);
        signifier_mark.set_filter(FilterMode::Linear);

        let (scaled_w, scaled_h) =
            assets::scale_to_fit(raster.width as f32, raster.height as f32, 260.0);

        Self {
            signifier_mark,
            signifier_size: vec2(scaled_w, scaled_h),
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
    let assets = Assets::load();

    loop {
        if is_key_pressed(KeyCode::Q) {
            break;
        }

        match game.scene {
            Scene::Title => render_title(&mut game, &assets),
            Scene::ClassSelect => render_class_select(&mut game),
            Scene::Dungeon => render_dungeon(&mut game),
        }

        next_frame().await;
    }
}

fn render_title(game: &mut Game, assets: &Assets) {
    clear_background(BLACK);

    let title_size = 64.0;
    let subtitle_size = 28.0;

    draw_centered("FLAGHACK2", 90.0, title_size, ACCENT);

    let rotation = (get_time() as f32) * 0.25;
    let x = (screen_width() - assets.signifier_size.x) * 0.5;
    let y = 150.0;

    draw_texture_ex(
        &assets.signifier_mark,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(assets.signifier_size),
            rotation,
            ..Default::default()
        },
    );

    draw_centered("Press any key", 470.0, subtitle_size, ACCENT);
    draw_centered("Q to quit", 505.0, 20.0, ACCENT);

    if get_last_key_pressed().is_some() {
        game.scene = Scene::ClassSelect;
    }
}

fn render_class_select(game: &mut Game) {
    clear_background(BLACK);

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
    clear_background(FIELD_GREEN);

    draw_centered("Dungeon", 60.0, 44.0, ACCENT);
    draw_centered("WASD to move", 110.0, 20.0, ACCENT);
    draw_centered("Esc to class select", 135.0, 20.0, ACCENT);
    draw_centered("Q to quit", 160.0, 20.0, ACCENT);

    handle_movement(game);

    draw_rectangle(
        game.player.pos.x,
        game.player.pos.y,
        PLAYER_SIZE,
        PLAYER_SIZE,
        BLACK,
    );

    let pos_text = format!(
        "Player position: ({:.1}, {:.1})",
        game.player.pos.x, game.player.pos.y
    );
    draw_centered(&pos_text, 230.0, 22.0, ACCENT);

    if is_key_pressed(KeyCode::Escape) {
        game.scene = Scene::ClassSelect;
    }
}

fn handle_movement(game: &mut Game) {
    let input = movement::InputState {
        up: is_key_down(KeyCode::W),
        down: is_key_down(KeyCode::S),
        left: is_key_down(KeyCode::A),
        right: is_key_down(KeyCode::D),
    };

    let delta = movement::movement_delta(input, PLAYER_SPEED, get_frame_time());
    game.player.pos += delta;

    let max_x = (screen_width() - PLAYER_SIZE).max(0.0);
    let max_y = (screen_height() - PLAYER_SIZE).max(0.0);
    game.player.pos.x = game.player.pos.x.clamp(0.0, max_x);
    game.player.pos.y = game.player.pos.y.clamp(0.0, max_y);
}

fn draw_centered(text: &str, y: f32, size: f32, color: Color) {
    let metrics = measure_text(text, None, size as u16, 1.0);
    let x = (screen_width() - metrics.width) * 0.5;
    draw_text(text, x, y, size, color);
}
