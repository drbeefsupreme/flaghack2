use crate::assets;
use macroquad::prelude::*;
use std::path::Path;

const POLYGON_EPSILON: f32 = 1.0e-5;

#[derive(Debug)]
pub struct TileMap {
    pub tile_size: f32,
    pub columns: usize,
    pub rows: usize,
    pub width: f32,
    pub height: f32,
    tiles: Vec<Option<Texture2D>>,
}

#[derive(Debug)]
pub struct MapRegion {
    pub name: &'static str,
    vertices: Vec<Vec2>,
    triangles: Vec<[Vec2; 3]>,
    color: Color,
}

impl MapRegion {
    pub fn new(name: &'static str, vertices: Vec<Vec2>, color: Color) -> Self {
        let triangles = triangulate_polygon(&vertices);
        Self {
            name,
            vertices,
            triangles,
            color,
        }
    }

    pub fn set_vertices(&mut self, vertices: Vec<Vec2>) {
        self.vertices = vertices;
        self.triangles = triangulate_polygon(&self.vertices);
    }

    pub fn draw(&self) {
        for tri in &self.triangles {
            draw_triangle(tri[0], tri[1], tri[2], self.color);
        }
    }
}

impl TileMap {
    pub fn load_from_dir<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        let mut entries = Vec::new();

        for entry in std::fs::read_dir(path).expect("Failed to read map directory") {
            let entry = entry.expect("Failed to read map entry");
            let file_path = entry.path();
            if file_path.extension().and_then(|e| e.to_str()) != Some("png") {
                continue;
            }
            let file_name = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .expect("Invalid filename");
            if let Some((x, y)) = parse_tile_filename(file_name) {
                entries.push((x, y, file_path));
            }
        }

        entries.sort_by(|a, b| (a.1, a.0).cmp(&(b.1, b.0)));
        let max_x = entries.iter().map(|(x, _, _)| *x).max().unwrap_or(0);
        let max_y = entries.iter().map(|(_, y, _)| *y).max().unwrap_or(0);

        let columns = max_x + 1;
        let rows = max_y + 1;

        let mut tiles: Vec<Option<Texture2D>> = vec![None; columns * rows];
        let mut tile_size = None;

        for (x, y, file_path) in entries {
            let raster = assets::load_png_rgba(
                file_path
                    .to_str()
                    .expect("Invalid tile path string"),
            );
            tile_size.get_or_insert(raster.width as f32);
            if (raster.width as f32 - tile_size.unwrap()).abs() > f32::EPSILON {
                panic!("Tile width mismatch for {:?}", file_path);
            }
            if (raster.height as f32 - tile_size.unwrap()).abs() > f32::EPSILON {
                panic!("Tile height mismatch for {:?}", file_path);
            }

            let texture = Texture2D::from_rgba8(raster.width, raster.height, &raster.pixels);
            texture.set_filter(FilterMode::Linear);

            let index = y * columns + x;
            tiles[index] = Some(texture);
        }

        let tile_size = tile_size.unwrap_or(1.0);
        let width = tile_size * columns as f32;
        let height = tile_size * rows as f32;

        Self {
            tile_size,
            columns,
            rows,
            width,
            height,
            tiles,
        }
    }

    pub fn draw(&self, view: Rect) {
        let range = tile_range(view, self.tile_size, self.columns, self.rows);
        for y in range.y0..=range.y1 {
            for x in range.x0..=range.x1 {
                if let Some(texture) = &self.tiles[y * self.columns + x] {
                    draw_texture_ex(
                        texture,
                        x as f32 * self.tile_size,
                        y as f32 * self.tile_size,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(self.tile_size, self.tile_size)),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    pub fn field_rect(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width, self.height)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileRange {
    pub x0: usize,
    pub x1: usize,
    pub y0: usize,
    pub y1: usize,
}

pub fn tile_range(view: Rect, tile_size: f32, columns: usize, rows: usize) -> TileRange {
    let x0 = (view.x / tile_size).floor().max(0.0) as usize;
    let y0 = (view.y / tile_size).floor().max(0.0) as usize;
    let x1 = ((view.x + view.w) / tile_size).floor().max(0.0) as usize;
    let y1 = ((view.y + view.h) / tile_size).floor().max(0.0) as usize;

    TileRange {
        x0: x0.min(columns.saturating_sub(1)),
        x1: x1.min(columns.saturating_sub(1)),
        y0: y0.min(rows.saturating_sub(1)),
        y1: y1.min(rows.saturating_sub(1)),
    }
}

pub fn parse_tile_filename(name: &str) -> Option<(usize, usize)> {
    let name = name.strip_suffix(".png")?;
    let mut parts = name.split('_');
    if parts.next()? != "tile" {
        return None;
    }
    let x = parts.next()?.parse::<usize>().ok()?;
    let y = parts.next()?.parse::<usize>().ok()?;
    Some((x, y))
}

pub fn travel_speed(map_width: f32, map_height: f32, minutes: f32) -> f32 {
    let max_dim = map_width.max(map_height).max(1.0);
    max_dim / (minutes * 60.0)
}

pub fn adjusted_travel_speed(map_width: f32, map_height: f32, minutes: f32, multiplier: f32) -> f32 {
    travel_speed(map_width, map_height, minutes) * multiplier
}

pub fn triangulate_polygon(vertices: &[Vec2]) -> Vec<[Vec2; 3]> {
    let count = vertices.len();
    if count < 3 {
        return Vec::new();
    }

    let area = polygon_area(vertices);
    if area.abs() < POLYGON_EPSILON {
        return Vec::new();
    }
    let orientation = if area > 0.0 { 1.0 } else { -1.0 };

    let mut indices: Vec<usize> = (0..count).collect();
    let mut triangles: Vec<[Vec2; 3]> = Vec::with_capacity(count.saturating_sub(2));
    let mut failed = false;
    let mut guard = 0;

    while indices.len() > 3 && guard < count * count {
        let mut ear_found = false;
        let len = indices.len();

        for i in 0..len {
            let prev = indices[(i + len - 1) % len];
            let curr = indices[i];
            let next = indices[(i + 1) % len];

            if !is_convex(vertices[prev], vertices[curr], vertices[next], orientation) {
                continue;
            }

            if triangle_area(vertices[prev], vertices[curr], vertices[next]).abs() < POLYGON_EPSILON {
                continue;
            }

            let mut contains = false;
            for &idx in &indices {
                if idx == prev || idx == curr || idx == next {
                    continue;
                }

                if point_in_triangle(vertices[idx], vertices[prev], vertices[curr], vertices[next]) {
                    contains = true;
                    break;
                }
            }

            if contains {
                continue;
            }

            triangles.push([vertices[prev], vertices[curr], vertices[next]]);
            indices.remove(i);
            ear_found = true;
            break;
        }

        if !ear_found {
            failed = true;
            break;
        }

        guard += 1;
    }

    if !failed && indices.len() == 3 {
        triangles.push([
            vertices[indices[0]],
            vertices[indices[1]],
            vertices[indices[2]],
        ]);
    }

    if failed || triangles.len() != count.saturating_sub(2) {
        if is_polygon_convex(vertices) {
            triangles.clear();
            for i in 1..count.saturating_sub(1) {
                triangles.push([vertices[0], vertices[i], vertices[i + 1]]);
            }
        }
    }

    triangles
}

fn is_polygon_convex(vertices: &[Vec2]) -> bool {
    let count = vertices.len();
    if count < 4 {
        return true;
    }

    let area = polygon_area(vertices);
    if area.abs() < POLYGON_EPSILON {
        return false;
    }
    let orientation = if area > 0.0 { 1.0 } else { -1.0 };

    for i in 0..count {
        let prev = vertices[(i + count - 1) % count];
        let curr = vertices[i];
        let next = vertices[(i + 1) % count];
        let cross = cross_2d(next - curr, prev - curr);
        if cross.abs() < POLYGON_EPSILON {
            continue;
        }
        if cross * orientation < 0.0 {
            return false;
        }
    }

    true
}

fn is_convex(prev: Vec2, curr: Vec2, next: Vec2, orientation: f32) -> bool {
    let cross = cross_2d(next - curr, prev - curr);
    cross * orientation > POLYGON_EPSILON
}

fn point_in_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let ab = cross_2d(b - a, p - a);
    let bc = cross_2d(c - b, p - b);
    let ca = cross_2d(a - c, p - c);

    let has_neg = ab < -POLYGON_EPSILON || bc < -POLYGON_EPSILON || ca < -POLYGON_EPSILON;
    let has_pos = ab > POLYGON_EPSILON || bc > POLYGON_EPSILON || ca > POLYGON_EPSILON;

    !(has_neg && has_pos)
}

fn triangle_area(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    cross_2d(b - a, c - a) * 0.5
}

fn polygon_area(vertices: &[Vec2]) -> f32 {
    let count = vertices.len();
    let mut area = 0.0;
    for i in 0..count {
        let j = (i + 1) % count;
        area += vertices[i].x * vertices[j].y - vertices[j].x * vertices[i].y;
    }
    area * 0.5
}

fn cross_2d(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tile_filename_accepts_valid() {
        assert_eq!(parse_tile_filename("tile_3_5.png"), Some((3, 5)));
    }

    #[test]
    fn parse_tile_filename_rejects_invalid() {
        assert_eq!(parse_tile_filename("tile_x_5.png"), None);
        assert_eq!(parse_tile_filename("tile_1.png"), None);
        assert_eq!(parse_tile_filename("map_1_2.png"), None);
    }

    #[test]
    fn tile_range_clamps_to_bounds() {
        let view = Rect::new(-10.0, -5.0, 120.0, 80.0);
        let range = tile_range(view, 64.0, 3, 2);
        assert_eq!(range.x0, 0);
        assert_eq!(range.y0, 0);
        assert_eq!(range.x1, 1);
        assert_eq!(range.y1, 1);
    }

    #[test]
    fn travel_speed_uses_max_dimension() {
        let speed = travel_speed(1200.0, 600.0, 10.0);
        assert!((speed - 2.0).abs() < 0.001);
    }

    #[test]
    fn adjusted_travel_speed_scales_base() {
        let speed = adjusted_travel_speed(1200.0, 600.0, 10.0, 4.0);
        assert!((speed - 8.0).abs() < 0.001);
    }
}
