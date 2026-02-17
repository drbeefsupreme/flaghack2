use macroquad::prelude::*;
use std::collections::HashMap;

use crate::constants;

const PLAYA_COLOR: Color = Color::new(0.85, 0.78, 0.65, 1.0);
const DEEP_PLAYA_COLOR: Color = Color::new(0.78, 0.71, 0.58, 1.0);
const WILDERNESS_COLOR: Color = Color::new(0.72, 0.65, 0.52, 1.0);
const SCRUB_COLOR: Color = Color::new(0.58, 0.62, 0.42, 1.0);
const CRACK_COLOR: Color = Color::new(0.70, 0.63, 0.52, 1.0);

const NOISE_STRENGTH: f32 = 0.15;

pub struct ProceduralMap {
    pub width: f32,
    pub height: f32,
    pub core_radius: f32,
    pub effigy_pos: Vec2,
    chunks: HashMap<(i32, i32), Chunk>,
    rng_state: u32,
}

struct Chunk {
    features: Vec<TerrainFeature>,
    ground_texture: Option<Texture2D>,
}

enum TerrainFeature {
    Rock { pos: Vec2, size: f32, color: Color },
    ScrubBrush { pos: Vec2, size: f32 },
    Crack { start: Vec2, end: Vec2, width: f32 },
}

pub fn core_radius(camp_count: usize) -> f32 {
    constants::BURN_BASE_CORE_RADIUS + camp_count as f32 * constants::BURN_RADIUS_PER_CAMP
}

pub fn initial_map_size(core_radius: f32) -> f32 {
    core_radius * 3.0
}

pub fn chunk_grid(pos: f32) -> i32 {
    (pos / constants::BURN_CHUNK_SIZE).floor() as i32
}

impl ProceduralMap {
    pub fn new(effigy_pos: Vec2, core_radius: f32, seed: u32) -> Self {
        let size = initial_map_size(core_radius);
        Self {
            width: size,
            height: size,
            core_radius,
            effigy_pos,
            chunks: HashMap::new(),
            rng_state: seed,
        }
    }

    pub fn field_rect(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width, self.height)
    }

    pub fn ensure_chunks_for_view(&mut self, view: Rect) {
        let margin = constants::BURN_CHUNK_SIZE;
        let x0 = chunk_grid(view.x - margin);
        let x1 = chunk_grid(view.x + view.w + margin);
        let y0 = chunk_grid(view.y - margin);
        let y1 = chunk_grid(view.y + view.h + margin);

        for gx in x0..=x1 {
            for gy in y0..=y1 {
                if !self.chunks.contains_key(&(gx, gy)) {
                    let chunk = generate_chunk(
                        gx,
                        gy,
                        &mut self.rng_state,
                        self.effigy_pos,
                        self.core_radius,
                    );
                    self.chunks.insert((gx, gy), chunk);
                    let edge_x = (gx + 1) as f32 * constants::BURN_CHUNK_SIZE;
                    let edge_y = (gy + 1) as f32 * constants::BURN_CHUNK_SIZE;
                    if edge_x > self.width {
                        self.width = edge_x;
                    }
                    if edge_y > self.height {
                        self.height = edge_y;
                    }
                }
            }
        }
    }

    pub fn draw(&mut self, view: Rect) {
        self.ensure_chunks_for_view(view);

        let effigy_pos = self.effigy_pos;
        let core_radius = self.core_radius;

        for (&(gx, gy), chunk) in self.chunks.iter_mut() {
            let cx = gx as f32 * constants::BURN_CHUNK_SIZE;
            let cy = gy as f32 * constants::BURN_CHUNK_SIZE;
            let chunk_rect =
                Rect::new(cx, cy, constants::BURN_CHUNK_SIZE, constants::BURN_CHUNK_SIZE);

            if !rects_overlap(view, chunk_rect) {
                continue;
            }

            if chunk.ground_texture.is_none() {
                chunk.ground_texture =
                    Some(generate_chunk_texture(gx, gy, effigy_pos, core_radius));
            }
            if let Some(ref tex) = chunk.ground_texture {
                draw_texture(tex, cx, cy, WHITE);
            }

            for feature in &chunk.features {
                draw_terrain_feature(feature);
            }
        }
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
}

fn generate_chunk_texture(
    gx: i32,
    gy: i32,
    effigy_pos: Vec2,
    core_radius: f32,
) -> Texture2D {
    let size = constants::BURN_CHUNK_SIZE as u16;
    let cx = gx as f32 * constants::BURN_CHUNK_SIZE;
    let cy = gy as f32 * constants::BURN_CHUNK_SIZE;

    let threshold_wild = core_radius * 2.0;
    let threshold_deep = core_radius * 1.2;
    let tw_sq = threshold_wild * threshold_wild;
    let td_sq = threshold_deep * threshold_deep;

    let mut pixels = vec![0u8; (size as usize) * (size as usize) * 4];

    for py in 0..size as usize {
        let wy = cy + py as f32;
        let dy = wy - effigy_pos.y;
        for px in 0..size as usize {
            let wx = cx + px as f32;
            let dx = wx - effigy_pos.x;
            let dist_sq = dx * dx + dy * dy;

            let base = if dist_sq > tw_sq {
                WILDERNESS_COLOR
            } else if dist_sq > td_sq {
                DEEP_PLAYA_COLOR
            } else {
                PLAYA_COLOR
            };

            let n = terrain_noise(wx, wy);
            let variation = (n - 0.5) * NOISE_STRENGTH * 2.0;

            let idx = (py * size as usize + px) * 4;
            pixels[idx] = ((base.r + variation).clamp(0.0, 1.0) * 255.0) as u8;
            pixels[idx + 1] = ((base.g + variation).clamp(0.0, 1.0) * 255.0) as u8;
            pixels[idx + 2] = ((base.b + variation).clamp(0.0, 1.0) * 255.0) as u8;
            pixels[idx + 3] = 255;
        }
    }

    let tex = Texture2D::from_rgba8(size, size, &pixels);
    tex.set_filter(FilterMode::Nearest);
    tex
}

fn terrain_noise(x: f32, y: f32) -> f32 {
    let n1 = noise2d(x * 0.02, y * 0.02);
    let n2 = noise2d(x * 0.07, y * 0.07);
    let n3 = noise2d(x * 0.2, y * 0.2);
    n1 * 0.45 + n2 * 0.35 + n3 * 0.20
}

fn noise2d(x: f32, y: f32) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let fx = x - x.floor();
    let fy = y - y.floor();

    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);

    let n00 = hash_grid(ix, iy);
    let n10 = hash_grid(ix + 1, iy);
    let n01 = hash_grid(ix, iy + 1);
    let n11 = hash_grid(ix + 1, iy + 1);

    let nx0 = n00 + (n10 - n00) * sx;
    let nx1 = n01 + (n11 - n01) * sx;
    nx0 + (nx1 - nx0) * sy
}

fn hash_grid(x: i32, y: i32) -> f32 {
    let mut h = (x as u32)
        .wrapping_mul(374761393)
        .wrapping_add((y as u32).wrapping_mul(668265263));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h = h ^ (h >> 16);
    (h & 0xFFFF) as f32 / 65535.0
}

fn rects_overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

fn generate_chunk(
    gx: i32,
    gy: i32,
    rng: &mut u32,
    effigy_pos: Vec2,
    core_radius: f32,
) -> Chunk {
    let cx = gx as f32 * constants::BURN_CHUNK_SIZE;
    let cy = gy as f32 * constants::BURN_CHUNK_SIZE;
    let chunk_center = vec2(
        cx + constants::BURN_CHUNK_SIZE * 0.5,
        cy + constants::BURN_CHUNK_SIZE * 0.5,
    );
    let dist = chunk_center.distance(effigy_pos);

    let mut features = Vec::new();

    let feature_density = if dist < core_radius * 0.5 {
        1
    } else if dist < core_radius {
        2
    } else {
        4
    };

    for _ in 0..feature_density {
        let fx = cx + next_f32(rng) * constants::BURN_CHUNK_SIZE;
        let fy = cy + next_f32(rng) * constants::BURN_CHUNK_SIZE;
        let roll = next_f32(rng);

        if roll < 0.4 {
            let brightness = 0.45 + next_f32(rng) * 0.15;
            features.push(TerrainFeature::Rock {
                pos: vec2(fx, fy),
                size: 2.0 + next_f32(rng) * 4.0,
                color: Color::new(brightness, brightness - 0.05, brightness - 0.1, 1.0),
            });
        } else if roll < 0.7 {
            features.push(TerrainFeature::ScrubBrush {
                pos: vec2(fx, fy),
                size: 3.0 + next_f32(rng) * 5.0,
            });
        } else {
            let angle = next_f32(rng) * std::f32::consts::TAU;
            let length = 10.0 + next_f32(rng) * 30.0;
            features.push(TerrainFeature::Crack {
                start: vec2(fx, fy),
                end: vec2(fx + angle.cos() * length, fy + angle.sin() * length),
                width: 0.5 + next_f32(rng) * 1.0,
            });
        }
    }

    Chunk {
        features,
        ground_texture: None,
    }
}

fn draw_terrain_feature(feature: &TerrainFeature) {
    match feature {
        TerrainFeature::Rock { pos, size, color } => {
            draw_circle(pos.x, pos.y, *size, *color);
        }
        TerrainFeature::ScrubBrush { pos, size } => {
            draw_circle(pos.x, pos.y, *size, SCRUB_COLOR);
        }
        TerrainFeature::Crack {
            start,
            end,
            width,
        } => {
            draw_line(start.x, start.y, end.x, end.y, *width, CRACK_COLOR);
        }
    }
}

fn next_f32(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    let v = (*state >> 8) as f32;
    v / ((u32::MAX >> 8) as f32 + 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_grid_from_position() {
        assert_eq!(chunk_grid(0.0), 0);
        assert_eq!(chunk_grid(511.0), 0);
        assert_eq!(chunk_grid(512.0), 1);
        assert_eq!(chunk_grid(-1.0), -1);
    }

    #[test]
    fn core_radius_scales_with_camps() {
        let r5 = core_radius(5);
        let r10 = core_radius(10);
        assert!(r10 > r5);
        let expected = constants::BURN_BASE_CORE_RADIUS + 5.0 * constants::BURN_RADIUS_PER_CAMP;
        assert!((r5 - expected).abs() < 1e-3);
    }

    #[test]
    fn initial_map_size_is_triple_core() {
        let cr = core_radius(5);
        assert!((initial_map_size(cr) - cr * 3.0).abs() < 1e-3);
    }

    #[test]
    fn ensure_chunks_generates_missing() {
        let effigy = vec2(600.0, 600.0);
        let mut map = ProceduralMap::new(effigy, 400.0, 42);
        assert_eq!(map.chunk_count(), 0);
        map.ensure_chunks_for_view(Rect::new(500.0, 500.0, 200.0, 200.0));
        assert!(map.chunk_count() > 0);
    }

    #[test]
    fn map_dimensions_grow_with_chunks() {
        let effigy = vec2(600.0, 600.0);
        let mut map = ProceduralMap::new(effigy, 400.0, 42);
        let initial_w = map.width;
        map.ensure_chunks_for_view(Rect::new(10000.0, 10000.0, 200.0, 200.0));
        assert!(map.width > initial_w);
        assert!(map.height > initial_w);
    }

    #[test]
    fn noise2d_is_seamless_and_bounded() {
        for i in 0..100 {
            let x = i as f32 * 0.37 - 20.0;
            let y = i as f32 * 0.53 - 15.0;
            let n = noise2d(x, y);
            assert!(n >= 0.0 && n <= 1.0, "noise2d({}, {}) = {} out of [0,1]", x, y, n);
        }
        // Check continuity: nearby points should have similar values
        let a = noise2d(5.0, 5.0);
        let b = noise2d(5.01, 5.0);
        assert!((a - b).abs() < 0.1, "noise not continuous: {} vs {}", a, b);
    }

    #[test]
    fn terrain_noise_has_variation() {
        let mut min = 1.0f32;
        let mut max = 0.0f32;
        for i in 0..50 {
            let n = terrain_noise(i as f32 * 30.0, i as f32 * 17.0);
            min = min.min(n);
            max = max.max(n);
        }
        assert!(max - min > 0.1, "terrain noise has too little variation: {} to {}", min, max);
    }

    #[test]
    fn terrain_features_within_chunk_bounds() {
        let mut rng = 123u32;
        let effigy = vec2(1000.0, 1000.0);
        let chunk = generate_chunk(2, 3, &mut rng, effigy, 500.0);
        let cx = 2.0 * constants::BURN_CHUNK_SIZE;
        let cy = 3.0 * constants::BURN_CHUNK_SIZE;
        for feature in &chunk.features {
            let pos = match feature {
                TerrainFeature::Rock { pos, .. } => *pos,
                TerrainFeature::ScrubBrush { pos, .. } => *pos,
                TerrainFeature::Crack { start, .. } => *start,
            };
            assert!(pos.x >= cx && pos.x <= cx + constants::BURN_CHUNK_SIZE);
            assert!(pos.y >= cy && pos.y <= cy + constants::BURN_CHUNK_SIZE);
        }
    }
}
