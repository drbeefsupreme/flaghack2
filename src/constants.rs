use macroquad::prelude::*;

use crate::{camera, scale};

pub const SCREEN_W: i32 = 960;
pub const SCREEN_H: i32 = 540;
pub const ACCENT: Color = Color::new(1.0, 0.9, 0.0, 1.0);
pub const HUD_HEIGHT: f32 = 48.0;
pub const FLAG_INTERACT_RADIUS: f32 = 48.0 * scale::MODEL_SCALE;
pub const FLAG_POLE_HEIGHT: f32 = 36.0 * scale::MODEL_SCALE;
pub const FLAG_POLE_WIDTH: f32 = 3.0 * scale::MODEL_SCALE;
pub const FLAG_CLOTH_SIZE: Vec2 =
    Vec2::new(22.0 * scale::MODEL_SCALE, 14.0 * scale::MODEL_SCALE);
pub const FLAG_PLACE_OFFSET: Vec2 =
    Vec2::new(28.0 * scale::MODEL_SCALE, 0.0);
pub const FLAG_COUNT_START: usize = 10;
pub const STARTING_FLAG_INVENTORY: u32 = 10;
pub const LEY_MAX_DISTANCE: f32 = 150.0;
pub const LEY_COLOR_PURPLE: Color = Color::new(0.55, 0.25, 0.95, 1.0);
pub const LEY_COLOR_PINK: Color = Color::new(1.0, 0.35, 0.75, 1.0);
pub const LEY_COLOR_CYCLE_SPEED: f32 = 0.9;
pub const PENTAGRAM_COLOR_RED: Color = Color::new(1.0, 0.15, 0.05, 1.0);
pub const PENTAGRAM_COLOR_ORANGE: Color = Color::new(1.0, 0.55, 0.0, 1.0);
pub const PENTAGRAM_COLOR_CYCLE_SPEED: f32 = 1.2;
pub const LEY_SPARKLE_SPEED: f32 = 3.5;
pub const LEY_SPARKLE_STRENGTH: f32 = 0.35;
pub const LEY_SPARKLE_SPATIAL: f32 = 0.02;
pub const LEY_MIN_ALPHA: f32 = 0.05;
pub const PENTAGRAM_MIN_ALPHA: f32 = 0.12;
pub const PENTAGRAM_CENTER_RADIUS: f32 = 32.0 * scale::MODEL_SCALE;
pub const PENTAGRAM_SPARKLE_SPAWN_RATE: f32 = 60.0;
pub const PENTAGRAM_SPARKLE_MIN_ALPHA: f32 = 0.5;
pub const PENTAGRAM_SPARKLE_MAX_ALPHA: f32 = 0.95;
pub const PENTAGRAM_SPARKLE_MIN_SPEED: f32 = 40.0 * scale::MODEL_SCALE;
pub const PENTAGRAM_SPARKLE_MAX_SPEED: f32 = 160.0 * scale::MODEL_SCALE;
pub const PENTAGRAM_SPARKLE_MIN_RADIUS: f32 = 2.0 * scale::MODEL_SCALE;
pub const PENTAGRAM_SPARKLE_MIN_MAX_RADIUS: f32 = 60.0 * scale::MODEL_SCALE;
pub const PENTAGRAM_SPARKLE_MAX_RADIUS_FACTOR: f32 = 0.9;
pub const PENTAGRAM_SPARKLE_HUE_SPEED: f32 = 0.35;
pub const T3MPCAMP_NOTICE_DURATION: f32 = 4.0;
pub const T3MPCAMP_NOTICE_FADE: f32 = 0.5;
pub const T3MPCAMP_NOTICE_SIZE: f32 = 54.0;
pub const T3MPCAMP_CAMPFIRE_POS: Vec2 = Vec2::new(4982.0, 3233.0);
pub const HIPPIE_STEAL_RADIUS: f32 = 36.0 * scale::MODEL_SCALE;
pub const CAMERA_ZOOM_MIN: f32 = camera::DEFAULT_ZOOM * 0.25;
pub const CAMERA_ZOOM_MAX: f32 = camera::DEFAULT_ZOOM * 2.0;
pub const CAMERA_ZOOM_STEP: f32 = 0.1;
pub const MAP_TILE_DIR: &str = "assets/map/tiles";
pub const MAP_TRAVEL_MINUTES: f32 = 10.0;
pub const SPEED_MULTIPLIER: f32 = 4.0;
pub const MAP_REGION_COLOR: Color = Color::new(0.1, 0.6, 0.2, 1.0);
pub const PLAYER_SPAWN_POS: Vec2 = Vec2::new(5015.0, 3292.0);
pub const T3MPCAMP_NAME: &str = "t3mpcamp";
pub const T3MPCAMP_VERTICES: [Vec2; 5] = [
    Vec2::new(4858.0, 3168.0),
    Vec2::new(5042.0, 3107.0),
    Vec2::new(5123.0, 3345.0),
    Vec2::new(5054.0, 3367.0),
    Vec2::new(4911.0, 3322.0),
];
