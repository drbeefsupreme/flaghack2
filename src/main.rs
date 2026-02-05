use macroquad::prelude::*;

mod assets;
mod camera;
mod fire;
mod scale;
mod movement;
mod flags;
mod scenery;
mod ley_lines;
mod player;
mod map;

const SCREEN_W: i32 = 960;
const SCREEN_H: i32 = 540;
const ACCENT: Color = Color::new(1.0, 0.9, 0.0, 1.0);
const HUD_HEIGHT: f32 = 48.0;
const FLAG_INTERACT_RADIUS: f32 = 48.0 * scale::MODEL_SCALE;
const FLAG_POLE_HEIGHT: f32 = 36.0 * scale::MODEL_SCALE;
const FLAG_POLE_WIDTH: f32 = 3.0 * scale::MODEL_SCALE;
const FLAG_CLOTH_SIZE: Vec2 =
    Vec2::new(22.0 * scale::MODEL_SCALE, 14.0 * scale::MODEL_SCALE);
const FLAG_PLACE_OFFSET: Vec2 =
    Vec2::new(28.0 * scale::MODEL_SCALE, 0.0);
const FLAG_COUNT_START: usize = 10;
const LEY_MAX_DISTANCE: f32 = 220.0;
const LEY_BASE_COLOR: Color = Color::new(0.55, 0.42, 0.9, 1.0);
const CAMERA_ZOOM_MIN: f32 = camera::DEFAULT_ZOOM * 0.25;
const CAMERA_ZOOM_MAX: f32 = camera::DEFAULT_ZOOM * 2.0;
const CAMERA_ZOOM_STEP: f32 = 0.1;
const MAP_TILE_DIR: &str = "assets/map/tiles";
const MAP_TRAVEL_MINUTES: f32 = 10.0;
const SPEED_MULTIPLIER: f32 = 4.0;
const MAP_REGION_COLOR: Color = Color::new(0.1, 0.6, 0.2, 1.0);
const PLAYER_SPAWN_POS: Vec2 = Vec2::new(5015.0, 3292.0);

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
    map: map::TileMap,
    map_regions: Vec<map::MapRegion>,
    camera: camera::CameraState,
    player_speed: f32,
}

struct Assets {
    signifier_mark: Texture2D,
    signifier_size: Vec2,
}

impl Game {
    fn new() -> Self {
        let map = map::TileMap::load_from_dir(MAP_TILE_DIR);
        let field_rect = map.field_rect();
        let map_regions = vec![map::MapRegion::new(
            vec![
                vec2(4858.0, 3168.0),
                vec2(5042.0, 3107.0),
                vec2(5123.0, 3345.0),
                vec2(5054.0, 3367.0),
                vec2(4911.0, 3322.0),
            ],
            MAP_REGION_COLOR,
        )];
        let flags = flags::spawn_random_flags(
            FLAG_COUNT_START,
            field_rect,
            40.0 * scale::MODEL_SCALE,
        );
        let ley_lines = ley_lines::compute_ley_lines(&flags, LEY_MAX_DISTANCE);
        let player_speed = map::adjusted_travel_speed(
            map.width,
            map.height,
            MAP_TRAVEL_MINUTES,
            SPEED_MULTIPLIER,
        );
        Self {
            scene: Scene::Title,
            player: Player {
                pos: PLAYER_SPAWN_POS,
                facing: player::Facing::Down,
            },
            class_name: "Vexillomancer",
            flags,
            flag_inventory: 0,
            wind: flags::Wind::new(vec2(1.0, 0.0), 0.6),
            scenery: scenery::spawn_scenery(field_rect),
            ley_lines,
            map,
            map_regions,
            camera: camera::CameraState::new(),
            player_speed,
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
    clear_background(BLACK);

    handle_camera(game);
    handle_movement(game);

    handle_flag_interactions(game);

    let time = get_time() as f32;
    let camera = build_camera(game);
    let view_rect = camera_view_rect(game, camera.target);
    set_camera(&camera);

    game.map.draw(view_rect);
    for region in &game.map_regions {
        region.draw();
    }
    scenery::draw_scenery(&game.scenery, time);
    draw_ley_lines(&game.ley_lines);
    for flag in &game.flags {
        draw_flag(flag, time, game.wind);
    }

    player::draw_player(game.player.pos, ACCENT, game.player.facing);

    set_default_camera();
    draw_centered("Dungeon", 60.0, 44.0, ACCENT);
    draw_centered("WASD to move", 110.0, 20.0, ACCENT);
    draw_centered("Esc to class select", 135.0, 20.0, ACCENT);
    draw_centered("Q to quit", 160.0, 20.0, ACCENT);
    draw_hud(game.flag_inventory, game.player_speed, game.player.pos);

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

    let delta = movement::movement_delta(input, game.player_speed, get_frame_time());
    game.player.pos += delta;

    let max_x = (game.map.width - player::PLAYER_WIDTH).max(0.0);
    let max_y = (game.map.height - player::PLAYER_HEIGHT).max(0.0);
    game.player.pos.x = game.player.pos.x.clamp(0.0, max_x);
    game.player.pos.y = game.player.pos.y.clamp(0.0, max_y);
}

fn handle_flag_interactions(game: &mut Game) {
    let field = game.map.field_rect();

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

fn draw_hud(flag_count: u32, speed: f32, player_pos: Vec2) {
    let y = screen_height() - HUD_HEIGHT;
    draw_rectangle(0.0, y, screen_width(), HUD_HEIGHT, BLACK);

    let text = format!("Flags: {}", flag_count);
    draw_text(&text, 16.0, y + 32.0, 26.0, ACCENT);

    let speed_text = format!("Speed: {:.1}px/s", speed);
    draw_text(&speed_text, 180.0, y + 32.0, 20.0, ACCENT);

    let coords = format_player_coords(player_pos);
    let metrics = measure_text(&coords, None, 20, 1.0);
    let x = screen_width() - metrics.width - 16.0;
    draw_text(&coords, x, y + 32.0, 20.0, ACCENT);
}

fn draw_ley_lines(lines: &[ley_lines::LeyLine]) {
    for line in lines {
        let color = Color {
            r: LEY_BASE_COLOR.r,
            g: LEY_BASE_COLOR.g,
            b: LEY_BASE_COLOR.b,
            a: line.intensity.clamp(0.1, 1.0),
        };
        let width = scale::scaled(2.0 + 4.0 * line.intensity);
        draw_line(line.a.x, line.a.y, line.b.x, line.b.y, width, color);
    }
}

fn handle_camera(game: &mut Game) {
    let (_, wheel_y) = mouse_wheel();
    if wheel_y.abs() > 0.0 {
        let zoom = game.camera.zoom * (1.0 + wheel_y * CAMERA_ZOOM_STEP);
        game.camera.zoom = zoom.clamp(CAMERA_ZOOM_MIN, CAMERA_ZOOM_MAX);
    }

    let mouse = vec2(mouse_position().0, mouse_position().1);
    if is_mouse_button_pressed(MouseButton::Middle) {
        game.camera.begin_drag(mouse);
    }
    if is_mouse_button_down(MouseButton::Middle) {
        if let Some(delta) = game.camera.drag(mouse) {
            game.camera.pan -= delta / game.camera.zoom;
        }
    } else {
        game.camera.end_drag();
    }
}

fn build_camera(game: &Game) -> Camera2D {
    let screen = vec2(screen_width(), screen_height());
    let view = camera::view_size(screen, game.camera.zoom);
    let player_center = game.player.pos + vec2(player::PLAYER_WIDTH * 0.5, player::PLAYER_HEIGHT * 0.5);
    let target = camera::clamp_target(
        player_center + game.camera.pan,
        vec2(game.map.width, game.map.height),
        view,
    );

    let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, screen.x, screen.y));
    cam.target = target;
    cam.zoom *= game.camera.zoom;
    cam.zoom = camera::flip_zoom_y(cam.zoom);
    cam
}

fn camera_view_rect(game: &Game, target: Vec2) -> Rect {
    let screen = vec2(screen_width(), screen_height());
    let view = camera::view_size(screen, game.camera.zoom);
    Rect::new(target.x - view.x * 0.5, target.y - view.y * 0.5, view.x, view.y)
}

fn draw_centered(text: &str, y: f32, size: f32, color: Color) {
    let metrics = measure_text(text, None, size as u16, 1.0);
    let x = (screen_width() - metrics.width) * 0.5;
    draw_text(text, x, y, size, color);
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
