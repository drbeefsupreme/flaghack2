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
mod constants;
mod npc;
mod hud;
mod geom;
mod flag_state;
mod regions;

use constants::*;

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
    flag_state: flag_state::FlagState,
    wind: flags::Wind,
    scenery: Vec<scenery::SceneryItem>,
    ley_lines: Vec<ley_lines::LeyLine>,
    pentagram_centers: Vec<Vec2>,
    pentagram_sparkles: Vec<PentagramSparkle>,
    sparkle_spawn_accum: f32,
    sparkle_spawn_counter: u32,
    flagic: u8,
    flagic_accum: f32,
    region_notices: Vec<RegionNotice>,
    region_vertices: Vec<Vec<Vec2>>,
    hippies: Vec<npc::Hippie>,
    map: map::TileMap,
    map_regions: Vec<map::MapRegion>,
    camera: camera::CameraState,
    player_speed: f32,
}

struct RegionNotice {
    region_name: &'static str,
    text: &'static str,
    inside: bool,
    timer: f32,
}

impl RegionNotice {
    fn new(region_name: &'static str, text: &'static str) -> Self {
        Self {
            region_name,
            text,
            inside: false,
            timer: -1.0,
        }
    }
}

struct Assets {
    signifier_mark: Texture2D,
    signifier_size: Vec2,
}

impl Game {
    fn new() -> Self {
        let map = map::TileMap::load_from_dir(MAP_TILE_DIR);
        let field_rect = map.field_rect();
        let region_configs = regions::region_configs();
        let map_regions = region_configs
            .iter()
            .map(|region| map::MapRegion::new(region.name, region.vertices.clone(), region.color))
            .collect::<Vec<_>>();
        let region_notices = region_configs
            .iter()
            .map(|region| RegionNotice::new(region.name, region.notice_text))
            .collect::<Vec<_>>();
        let region_vertices = regions::collect_region_vertices(&region_configs);
        let mut hippies = Vec::new();
        for (region_index, region) in region_configs.iter().enumerate() {
            if !region.spawns.hippies.is_empty() {
                hippies.extend(npc::spawn_hippies_with_flags(
                    &region.spawns.hippies,
                    region_index,
                    &region.vertices,
                ));
            }
        }
        let region_spawns = regions::collect_scenery_spawns(&region_configs);
        let ground_flags = flags::spawn_random_flags(
            FLAG_COUNT_START,
            field_rect,
            40.0 * scale::MODEL_SCALE,
        );
        let mut ground_flags = ground_flags;
        for pos in regions::collect_flag_spawns(&region_configs) {
            ground_flags.push(flags::make_flag(pos));
        }
        let total_flags =
            ground_flags.len() as u32 + STARTING_FLAG_INVENTORY + total_hippie_flags(&hippies);
        let flag_state = flag_state::FlagState::new(
            ground_flags,
            STARTING_FLAG_INVENTORY,
            total_flags,
        );
        let ley_state = ley_lines::compute_ley_state(flag_state.ground_flags(), LEY_MAX_DISTANCE);
        let scenery = scenery::spawn_scenery(field_rect, &region_spawns);
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
            flag_state,
            wind: flags::Wind::new(vec2(1.0, 0.0), 0.6),
            scenery,
            ley_lines: ley_state.lines,
            pentagram_centers: ley_state.pentagram_centers,
            pentagram_sparkles: Vec::new(),
            sparkle_spawn_accum: 0.0,
            sparkle_spawn_counter: 0,
            flagic: 0,
            flagic_accum: 0.0,
            region_notices,
            region_vertices,
            hippies,
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
    let dt = get_frame_time();
    let player_center = game.player.pos
        + vec2(player::PLAYER_WIDTH * 0.5, player::PLAYER_HEIGHT * 0.5);
    update_region_notices(game, player_center, dt);
    let hippies_picked = npc::update_hippies(
        &mut game.hippies,
        dt,
        &game.region_vertices,
        &mut game.flag_state,
        player_center,
        game.player_speed,
    );
    if hippies_picked {
        recompute_ley_state(game);
    }

    let camera = build_camera(game);
    let view_rect = camera_view_rect(game, camera.target);
    set_camera(&camera);

    game.map.draw(view_rect);
    for region in &game.map_regions {
        region.draw();
    }
    scenery::draw_scenery(&game.scenery, time);
    npc::draw_hippies(&game.hippies);
    draw_ley_lines(&game.ley_lines, time);
    for flag in game.flag_state.ground_flags() {
        draw_flag(flag, time, game.wind);
    }

    player::draw_player(game.player.pos, ACCENT, game.player.facing);
    let in_pentagram = player_in_pentagram(player_center, &game.pentagram_centers);
    update_flagic(&mut game.flagic, &mut game.flagic_accum, in_pentagram, dt);
    update_pentagram_sparkles(
        &mut game.pentagram_sparkles,
        &mut game.sparkle_spawn_accum,
        &mut game.sparkle_spawn_counter,
        player_center,
        in_pentagram,
        time,
        dt,
        view_rect,
    );

    set_default_camera();
    draw_region_notices(game);
    draw_centered("FLAGHACK2", 60.0, 64.0, ACCENT);
    draw_centered("WASD to move", 110.0, 20.0, ACCENT);
    draw_centered("Esc to class select", 135.0, 20.0, ACCENT);
    draw_centered("Q to quit", 160.0, 20.0, ACCENT);
    hud::draw_hud(
        game.flag_state.player_inventory(),
        game.player_speed,
        game.player.pos,
        current_total_flags(game),
        game.flagic,
    );

    game.flag_state
        .debug_assert_invariant(total_hippie_flags(&game.hippies));

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
        let placed =
            game.flag_state
                .try_place_from_player(game.player.pos, FLAG_PLACE_OFFSET, field);
        if placed {
            recompute_ley_state(game);
            return;
        }
    }

    if is_mouse_button_pressed(MouseButton::Right) {
        if game
            .flag_state
            .try_pickup_to_player(game.player.pos, FLAG_INTERACT_RADIUS)
        {
            recompute_ley_state(game);
        }

        let player_center = game.player.pos
            + vec2(player::PLAYER_WIDTH * 0.5, player::PLAYER_HEIGHT * 0.5);
        npc::try_steal_flag(
            &mut game.hippies,
            player_center,
            HIPPIE_STEAL_RADIUS,
            &mut game.flag_state,
        );
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

fn draw_ley_lines(lines: &[ley_lines::LeyLine], time: f32) {
    let cycle = 0.5 + 0.5 * (time * LEY_COLOR_CYCLE_SPEED).sin();
    let pent_cycle = 0.5 + 0.5 * (time * PENTAGRAM_COLOR_CYCLE_SPEED).sin();
    for line in lines {
        let sparkle_phase = (line.a.x + line.b.y) * LEY_SPARKLE_SPATIAL;
        let sparkle = 0.5 + 0.5 * (time * LEY_SPARKLE_SPEED + sparkle_phase).sin();
        let (base_a, base_b, cycle_t, sparkle_strength, min_alpha, width_base, width_scale, sat_base, sat_scale, bright_base, bright_scale, highlight) =
            match line.kind {
                ley_lines::LeyLineKind::Pentagram => (
                    PENTAGRAM_COLOR_RED,
                    PENTAGRAM_COLOR_ORANGE,
                    pent_cycle,
                    0.6,
                    PENTAGRAM_MIN_ALPHA,
                    1.8,
                    4.0,
                    0.9,
                    0.8,
                    0.9,
                    0.7,
                    Color::new(1.0, 0.9, 0.65, 1.0),
                ),
                ley_lines::LeyLineKind::Normal => (
                    LEY_COLOR_PURPLE,
                    LEY_COLOR_PINK,
                    cycle,
                    LEY_SPARKLE_STRENGTH,
                    LEY_MIN_ALPHA,
                    1.0,
                    3.0,
                    0.6,
                    0.6,
                    0.7,
                    0.5,
                    Color::new(1.0, 0.85, 1.0, 1.0),
                ),
            };

        let sparkle_mix = sparkle * sparkle_strength;
        let base = lerp_color(base_a, base_b, cycle_t);
        let mut color = lerp_color(base, highlight, sparkle_mix);

        let saturation = sat_base + sat_scale * line.intensity;
        color = saturate_color(color, saturation);
        let brightness = (bright_base + bright_scale * line.intensity).min(1.35);
        color.r = (color.r * brightness).min(1.0);
        color.g = (color.g * brightness).min(1.0);
        color.b = (color.b * brightness).min(1.0);

        let alpha = (min_alpha + (1.0 - min_alpha) * line.intensity).clamp(0.0, 1.0);
        let alpha = (alpha * (0.85 + 0.15 * sparkle)).clamp(0.0, 1.0);
        color.a = alpha;

        let width = scale::scaled(width_base + width_scale * line.intensity);
        draw_line(line.a.x, line.a.y, line.b.x, line.b.y, width, color);
    }
}

fn player_in_pentagram(pos: Vec2, centers: &[Vec2]) -> bool {
    centers
        .iter()
        .any(|center| center.distance(pos) <= PENTAGRAM_CENTER_RADIUS)
}

fn update_flagic(flagic: &mut u8, accum: &mut f32, in_pentagram: bool, dt: f32) {
    if !in_pentagram || dt <= 0.0 {
        return;
    }

    if *flagic >= FLAGIC_MAX {
        *flagic = FLAGIC_MAX;
        *accum = 0.0;
        return;
    }

    *accum += dt * FLAGIC_GAIN_RATE;
    let inc = accum.floor() as u32;
    if inc == 0 {
        return;
    }
    let next = (*flagic as u32 + inc).min(FLAGIC_MAX as u32);
    *flagic = next as u8;
    *accum -= inc as f32;
    if *flagic >= FLAGIC_MAX {
        *flagic = FLAGIC_MAX;
        *accum = 0.0;
    }
}

fn current_total_flags(game: &Game) -> u32 {
    game.flag_state
        .current_total(total_hippie_flags(&game.hippies))
}

fn total_hippie_flags(hippies: &[npc::Hippie]) -> u32 {
    hippies.iter().map(|h| h.carried_flags as u32).sum()
}

fn recompute_ley_state(game: &mut Game) {
    let state = ley_lines::compute_ley_state(game.flag_state.ground_flags(), LEY_MAX_DISTANCE);
    game.ley_lines = state.lines;
    game.pentagram_centers = state.pentagram_centers;
}

fn update_region_notices(game: &mut Game, player_center: Vec2, dt: f32) {
    let mut now_inside = Vec::with_capacity(game.region_notices.len());
    for notice in &game.region_notices {
        let in_region = game
            .map_regions
            .iter()
            .filter(|region| region.name == notice.region_name)
            .any(|region| region.contains_point(player_center));
        now_inside.push(in_region);
    }

    update_region_notice_states(&mut game.region_notices, &now_inside, dt);
}

fn draw_region_notices(game: &Game) {
    for notice in &game.region_notices {
        if notice.timer < 0.0 {
            continue;
        }
        let alpha = region_notice_alpha(notice.timer);
        if alpha <= 0.0 {
            continue;
        }
        let mut color = ACCENT;
        color.a = alpha;
        draw_centered(notice.text, screen_height() * 0.5, REGION_NOTICE_SIZE, color);
    }
}

fn update_region_notice_states(
    notices: &mut [RegionNotice],
    now_inside: &[bool],
    dt: f32,
) {
    debug_assert_eq!(notices.len(), now_inside.len());

    let mut entered_index = None;
    for (i, notice) in notices.iter().enumerate() {
        if now_inside[i] && !notice.inside {
            entered_index = Some(i);
            break;
        }
    }

    if let Some(index) = entered_index {
        for (i, notice) in notices.iter_mut().enumerate() {
            notice.inside = now_inside[i];
            if i == index {
                notice.timer = 0.0;
            } else {
                notice.timer = -1.0;
            }
        }
        return;
    }

    for (i, notice) in notices.iter_mut().enumerate() {
        notice.inside = now_inside[i];
        if notice.timer >= 0.0 {
            notice.timer += dt;
            if notice.timer > REGION_NOTICE_DURATION {
                notice.timer = -1.0;
            }
        }
    }
}

fn region_notice_alpha(elapsed: f32) -> f32 {
    if elapsed < 0.0 || elapsed > REGION_NOTICE_DURATION {
        return 0.0;
    }

    if elapsed < REGION_NOTICE_FADE {
        return (elapsed / REGION_NOTICE_FADE).clamp(0.0, 1.0);
    }

    if elapsed > REGION_NOTICE_DURATION - REGION_NOTICE_FADE {
        let remaining = REGION_NOTICE_DURATION - elapsed;
        return (remaining / REGION_NOTICE_FADE).clamp(0.0, 1.0);
    }

    1.0
}

fn update_pentagram_sparkles(
    sparkles: &mut Vec<PentagramSparkle>,
    spawn_accum: &mut f32,
    spawn_counter: &mut u32,
    origin: Vec2,
    in_pentagram: bool,
    time: f32,
    dt: f32,
    view: Rect,
) {
    if in_pentagram {
        spawn_pentagram_sparkles(
            sparkles,
            spawn_accum,
            spawn_counter,
            origin,
            time,
            dt,
            sparkle_max_radius(view),
        );
    }

    sparkles.retain(|sparkle| {
        let radius = sparkle_radius(sparkle, time);
        if radius > sparkle.max_radius {
            return false;
        }
        let pos = sparkle.origin + sparkle.dir * radius;
        let mut color = sparkle_color(sparkle, time, radius);
        color.a = sparkle_alpha(sparkle.base_alpha, radius, sparkle.max_radius);
        draw_circle(pos.x, pos.y, sparkle.size, color);
        true
    });
}

#[derive(Clone, Copy, Debug)]
struct PentagramSparkle {
    origin: Vec2,
    dir: Vec2,
    speed: f32,
    size: f32,
    hue_seed: f32,
    base_alpha: f32,
    spawn_time: f32,
    max_radius: f32,
}

fn spawn_pentagram_sparkles(
    sparkles: &mut Vec<PentagramSparkle>,
    spawn_accum: &mut f32,
    spawn_counter: &mut u32,
    origin: Vec2,
    time: f32,
    dt: f32,
    max_radius: f32,
) {
    let count = sparkle_spawn_count(spawn_accum, dt, PENTAGRAM_SPARKLE_SPAWN_RATE);
    for _ in 0..count {
        let sparkle = create_pentagram_sparkle(*spawn_counter, origin, time, max_radius);
        *spawn_counter = spawn_counter.wrapping_add(1);
        sparkles.push(sparkle);
    }
}

fn sparkle_spawn_count(accum: &mut f32, dt: f32, rate: f32) -> usize {
    *accum += dt * rate;
    let count = accum.floor() as usize;
    *accum -= count as f32;
    count
}

fn create_pentagram_sparkle(
    seed: u32,
    origin: Vec2,
    time: f32,
    max_radius: f32,
) -> PentagramSparkle {
    let s = seed as f32;
    let angle = hash11(s + 3.7) * std::f32::consts::TAU;
    let dir = vec2(angle.cos(), angle.sin());
    let speed = lerp(
        PENTAGRAM_SPARKLE_MIN_SPEED,
        PENTAGRAM_SPARKLE_MAX_SPEED,
        hash11(s + 9.1),
    );
    let size = (1.0 + 2.0 * hash11(s + 11.2)) * scale::MODEL_SCALE;
    let base_alpha = lerp(
        PENTAGRAM_SPARKLE_MIN_ALPHA,
        PENTAGRAM_SPARKLE_MAX_ALPHA,
        hash11(s + 5.7),
    );
    let hue_seed = hash11(s * 7.13);
    let max_radius = max_radius.max(PENTAGRAM_SPARKLE_MIN_RADIUS);

    PentagramSparkle {
        origin,
        dir,
        speed,
        size,
        hue_seed,
        base_alpha,
        spawn_time: time,
        max_radius,
    }
}

fn sparkle_radius(sparkle: &PentagramSparkle, time: f32) -> f32 {
    ((time - sparkle.spawn_time) * sparkle.speed).max(0.0)
}

fn sparkle_alpha(base_alpha: f32, radius: f32, max_radius: f32) -> f32 {
    let fade = (1.0 - radius / max_radius).clamp(0.0, 1.0);
    base_alpha * fade
}

fn sparkle_color(sparkle: &PentagramSparkle, time: f32, radius: f32) -> Color {
    let hue = (sparkle.hue_seed
        + time * PENTAGRAM_SPARKLE_HUE_SPEED
        + radius / sparkle.max_radius * 0.5)
        % 1.0;
    hsv_to_rgb(hue, 0.9, 1.0)
}

fn sparkle_max_radius(view: Rect) -> f32 {
    let diag = (view.w * view.w + view.h * view.h).sqrt();
    (diag * PENTAGRAM_SPARKLE_MAX_RADIUS_FACTOR).max(PENTAGRAM_SPARKLE_MIN_MAX_RADIUS)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn hash11(mut x: f32) -> f32 {
    x = (x * 12.9898).sin() * 43758.5453;
    x.fract().abs()
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let h = (h % 1.0 + 1.0) % 1.0;
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i as i32 % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    Color::new(r, g, b, 1.0)
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

fn saturate_color(color: Color, amount: f32) -> Color {
    let gray = (color.r + color.g + color.b) / 3.0;
    let r = (gray + (color.r - gray) * amount).clamp(0.0, 1.0);
    let g = (gray + (color.g - gray) * amount).clamp(0.0, 1.0);
    let b = (gray + (color.b - gray) * amount).clamp(0.0, 1.0);
    Color { r, g, b, a: color.a }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_flag_inventory_is_ten() {
        assert_eq!(STARTING_FLAG_INVENTORY, 10);
    }

    #[test]
    fn ley_line_max_distance_is_150() {
        assert_eq!(LEY_MAX_DISTANCE, 150.0);
    }

    #[test]
    fn lerp_color_midpoint_is_halfway() {
        let a = Color::new(0.0, 0.2, 0.4, 0.6);
        let b = Color::new(1.0, 0.6, 0.2, 1.0);
        let mid = lerp_color(a, b, 0.5);
        assert!((mid.r - 0.5).abs() < 1e-6);
        assert!((mid.g - 0.4).abs() < 1e-6);
        assert!((mid.b - 0.3).abs() < 1e-6);
        assert!((mid.a - 0.8).abs() < 1e-6);
    }

    #[test]
    fn player_in_pentagram_center_respects_radius() {
        let centers = vec![vec2(0.0, 0.0)];
        assert!(player_in_pentagram(vec2(PENTAGRAM_CENTER_RADIUS * 0.5, 0.0), &centers));
        assert!(!player_in_pentagram(
            vec2(PENTAGRAM_CENTER_RADIUS * 1.1, 0.0),
            &centers
        ));
    }

    #[test]
    fn sparkle_spawn_count_accumulates() {
        let mut accum = 0.0;
        let count = sparkle_spawn_count(&mut accum, 0.5, 10.0);
        assert_eq!(count, 5);
        assert!((accum - 0.0).abs() < 1e-6);
    }

    #[test]
    fn sparkle_alpha_fades_to_zero_at_max_radius() {
        let alpha = sparkle_alpha(0.8, 100.0, 100.0);
        assert!(alpha <= 1e-6);
    }

    #[test]
    fn sparkle_color_changes_over_time() {
        let sparkle = create_pentagram_sparkle(1, vec2(0.0, 0.0), 0.0, 200.0);
        let radius_now = sparkle_radius(&sparkle, 0.2);
        let radius_later = sparkle_radius(&sparkle, 0.8);
        let now = sparkle_color(&sparkle, 0.2, radius_now);
        let later = sparkle_color(&sparkle, 0.8, radius_later);
        let delta = (now.r - later.r).abs() + (now.g - later.g).abs() + (now.b - later.b).abs();
        assert!(delta > 1e-3);
    }

    #[test]
    fn region_notice_alpha_fades_in_and_out() {
        assert!(region_notice_alpha(0.0) <= 1e-6);
        assert!((region_notice_alpha(0.25) - 0.5).abs() < 1e-4);
        assert!((region_notice_alpha(0.5) - 1.0).abs() < 1e-6);
        assert!((region_notice_alpha(3.5) - 1.0).abs() < 1e-6);
        assert!((region_notice_alpha(3.75) - 0.5).abs() < 1e-4);
        assert!(region_notice_alpha(4.0) <= 1e-6);
    }

    #[test]
    fn region_notice_switches_on_new_entry() {
        let mut notices = vec![
            RegionNotice::new("a", "A"),
            RegionNotice::new("b", "B"),
        ];

        update_region_notice_states(&mut notices, &[true, false], 0.1);
        assert_eq!(notices[0].timer, 0.0);
        assert!(notices[1].timer < 0.0);

        update_region_notice_states(&mut notices, &[false, true], 0.1);
        assert!(notices[0].timer < 0.0);
        assert_eq!(notices[1].timer, 0.0);
    }

    #[test]
    fn flagic_increases_while_in_pentagram() {
        let mut flagic = 0u8;
        let mut accum = 0.0;
        update_flagic(&mut flagic, &mut accum, true, 0.2);
        assert_eq!(flagic, 1);
        assert!(accum.abs() < 1e-6);
    }

    #[test]
    fn flagic_does_not_increase_outside_pentagram() {
        let mut flagic = 0u8;
        let mut accum = 0.0;
        update_flagic(&mut flagic, &mut accum, false, 1.0);
        assert_eq!(flagic, 0);
        assert!(accum.abs() < 1e-6);
    }

    #[test]
    fn flagic_clamps_to_max() {
        let mut flagic = 99u8;
        let mut accum = 0.0;
        update_flagic(&mut flagic, &mut accum, true, 1.0);
        assert_eq!(flagic, FLAGIC_MAX);
        assert!(accum.abs() < 1e-6);
    }

    #[test]
    fn total_hippie_flags_sums_carried_flags() {
        let region = [
            vec2(0.0, 0.0),
            vec2(10.0, 0.0),
            vec2(0.0, 10.0),
        ];
        let hippies = npc::spawn_hippies_with_flags(
            &[(vec2(1.0, 1.0), 1), (vec2(2.0, 2.0), 2)],
            0,
            &region,
        );
        assert_eq!(total_hippie_flags(&hippies), 3);
    }
}
