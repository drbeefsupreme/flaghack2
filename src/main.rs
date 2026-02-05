use macroquad::prelude::*;

mod assets;
mod movement;
mod flags;
mod scenery;
mod ley_lines;
mod player;

const SCREEN_W: i32 = 960;
const SCREEN_H: i32 = 540;
const ACCENT: Color = Color::new(1.0, 0.9, 0.0, 1.0);
const FIELD_GREEN: Color = Color::new(0.06, 0.35, 0.12, 1.0);
const PLAYER_SPEED: f32 = 220.0;
const HUD_HEIGHT: f32 = 48.0;
const FLAG_INTERACT_RADIUS: f32 = 48.0;
const FLAG_POLE_HEIGHT: f32 = 36.0;
const FLAG_POLE_WIDTH: f32 = 3.0;
const FLAG_CLOTH_SIZE: Vec2 = Vec2::new(22.0, 14.0);
const FLAG_PLACE_OFFSET: Vec2 = Vec2::new(28.0, 0.0);
const FLAG_COUNT_START: usize = 10;
const LEY_MAX_DISTANCE: f32 = 220.0;
const LEY_BASE_COLOR: Color = Color::new(0.55, 0.42, 0.9, 1.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scene {
    Title,
    ClassSelect,
    Dungeon,
}

#[derive(Clone, Copy, Debug)]
struct Player {
    pos: Vec2,
    facing: player::Facing,
}

struct Game {
    scene: Scene,
    player: Player,
    class_name: &'static str,
    flags: Vec<flags::Flag>,
    flag_inventory: u32,
    wind: flags::Wind,
    scenery: Vec<scenery::SceneryItem>,
    ley_lines: Vec<ley_lines::LeyLine>,
}

struct Assets {
    signifier_mark: Texture2D,
    signifier_size: Vec2,
}

impl Game {
    fn new() -> Self {
        let field_rect = flags::field_rect(SCREEN_W as f32, SCREEN_H as f32, HUD_HEIGHT);
        let flags = flags::spawn_random_flags(FLAG_COUNT_START, field_rect, 40.0);
        let ley_lines = ley_lines::compute_ley_lines(&flags, LEY_MAX_DISTANCE);
        Self {
            scene: Scene::Title,
            player: Player {
                pos: vec2(
                    SCREEN_W as f32 * 0.5 - player::PLAYER_WIDTH * 0.5,
                    SCREEN_H as f32 * 0.55 - player::PLAYER_HEIGHT * 0.5,
                ),
                facing: player::Facing::Down,
            },
            class_name: "Vexillomancer",
            flags,
            flag_inventory: 0,
            wind: flags::Wind::new(vec2(1.0, 0.0), 0.6),
            scenery: scenery::spawn_scenery(field_rect),
            ley_lines,
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

    handle_flag_interactions(game);

    let time = get_time() as f32;
    scenery::draw_scenery(&game.scenery, time);
    draw_ley_lines(&game.ley_lines);
    for flag in &game.flags {
        draw_flag(flag, time, game.wind);
    }

    player::draw_player(game.player.pos, ACCENT, game.player.facing);

    draw_hud(game.flag_inventory);

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

    let direction = movement::input_direction(input);
    if direction.length() > 0.0 {
        game.player.facing = player::facing_from_direction(direction);
    }

    let delta = movement::movement_delta(input, PLAYER_SPEED, get_frame_time());
    game.player.pos += delta;

    let max_x = (screen_width() - player::PLAYER_WIDTH).max(0.0);
    let max_y = (screen_height() - HUD_HEIGHT - player::PLAYER_HEIGHT).max(0.0);
    game.player.pos.x = game.player.pos.x.clamp(0.0, max_x);
    game.player.pos.y = game.player.pos.y.clamp(0.0, max_y);
}

fn handle_flag_interactions(game: &mut Game) {
    let field = flags::field_rect(screen_width(), screen_height(), HUD_HEIGHT);

    if is_mouse_button_pressed(MouseButton::Left) {
        let placed = flags::try_place_flag(
            &mut game.flags,
            &mut game.flag_inventory,
            game.player.pos,
            FLAG_PLACE_OFFSET,
            field,
        );
        if placed {
            game.ley_lines = ley_lines::compute_ley_lines(&game.flags, LEY_MAX_DISTANCE);
            return;
        }
    }

    if is_mouse_button_pressed(MouseButton::Right) {
        if flags::try_pickup_flag(&mut game.flags, game.player.pos, FLAG_INTERACT_RADIUS) {
            game.flag_inventory = game.flag_inventory.saturating_add(1);
            game.ley_lines = ley_lines::compute_ley_lines(&game.flags, LEY_MAX_DISTANCE);
        }
    }
}

fn draw_flag(flag: &flags::Flag, time: f32, wind: flags::Wind) {
    let (pole, cloth) =
        flags::flag_parts(flag.pos, FLAG_POLE_HEIGHT, FLAG_POLE_WIDTH, FLAG_CLOTH_SIZE);
    let wiggle = flags::cloth_offset(time, wind, flag.phase);

    draw_rectangle(
        pole.x,
        pole.y,
        pole.w,
        pole.h,
        Color::new(0.55, 0.44, 0.28, 1.0),
    );

    draw_rectangle(cloth.x + wiggle.x, cloth.y + wiggle.y, cloth.w, cloth.h, ACCENT);
}

fn draw_hud(flag_count: u32) {
    let y = screen_height() - HUD_HEIGHT;
    draw_rectangle(0.0, y, screen_width(), HUD_HEIGHT, BLACK);

    let text = format!("Flags: {}", flag_count);
    draw_text(&text, 16.0, y + 32.0, 26.0, ACCENT);
}

fn draw_ley_lines(lines: &[ley_lines::LeyLine]) {
    for line in lines {
        let color = Color {
            r: LEY_BASE_COLOR.r,
            g: LEY_BASE_COLOR.g,
            b: LEY_BASE_COLOR.b,
            a: line.intensity.clamp(0.1, 1.0),
        };
        let width = 2.0 + 4.0 * line.intensity;
        draw_line(line.a.x, line.a.y, line.b.x, line.b.y, width, color);
    }
}

fn draw_centered(text: &str, y: f32, size: f32, color: Color) {
    let metrics = measure_text(text, None, size as u16, 1.0);
    let x = (screen_width() - metrics.width) * 0.5;
    draw_text(text, x, y, size, color);
}
