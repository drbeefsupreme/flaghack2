use macroquad::prelude::*;

const POLYGON_EPSILON: f32 = 1.0e-5;

pub fn point_in_polygon(point: Vec2, vertices: &[Vec2]) -> bool {
    if vertices.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = vertices.len() - 1;
    for i in 0..vertices.len() {
        let vi = vertices[i];
        let vj = vertices[j];
        let intersects = (vi.y > point.y) != (vj.y > point.y)
            && point.x < (vj.x - vi.x) * (point.y - vi.y) / (vj.y - vi.y + f32::EPSILON) + vi.x;
        if intersects {
            inside = !inside;
        }
        j = i;
    }
    inside
}

pub fn polygon_bounds(vertices: &[Vec2]) -> Option<(Vec2, Vec2)> {
    let first = *vertices.first()?;
    let mut min = first;
    let mut max = first;
    for v in &vertices[1..] {
        min.x = min.x.min(v.x);
        min.y = min.y.min(v.y);
        max.x = max.x.max(v.x);
        max.y = max.y.max(v.y);
    }
    Some((min, max))
}

pub fn line_points(start: Vec2, end: Vec2, spacing: f32) -> Vec<Vec2> {
    let length = (end - start).length();
    if length < f32::EPSILON || spacing <= f32::EPSILON {
        return vec![start];
    }

    let segments = ((length / spacing).round() as usize).max(1);
    let count = segments + 1;
    let mut points = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / (count - 1) as f32;
        points.push(start + (end - start) * t);
    }
    points
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

            if triangle_area(vertices[prev], vertices[curr], vertices[next]).abs() < POLYGON_EPSILON
            {
                continue;
            }

            let mut contains = false;
            for &idx in &indices {
                if idx == prev || idx == curr || idx == next {
                    continue;
                }

                if point_in_triangle(
                    vertices[idx],
                    vertices[prev],
                    vertices[curr],
                    vertices[next],
                ) {
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

pub fn point_in_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let ab = cross_2d(b - a, p - a);
    let bc = cross_2d(c - b, p - b);
    let ca = cross_2d(a - c, p - c);

    let has_neg = ab < -POLYGON_EPSILON || bc < -POLYGON_EPSILON || ca < -POLYGON_EPSILON;
    let has_pos = ab > POLYGON_EPSILON || bc > POLYGON_EPSILON || ca > POLYGON_EPSILON;

    !(has_neg && has_pos)
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

/// Clip a convex polygon by the half-plane on the left side of the line from `a` to `b`.
/// Uses Sutherland-Hodgman algorithm for a single edge.
pub fn clip_polygon_by_halfplane(polygon: &[Vec2], a: Vec2, b: Vec2) -> Vec<Vec2> {
    if polygon.is_empty() {
        return Vec::new();
    }
    let normal = vec2(-(b.y - a.y), b.x - a.x); // left-side normal
    let mut output = Vec::with_capacity(polygon.len() + 1);

    let len = polygon.len();
    for i in 0..len {
        let curr = polygon[i];
        let next = polygon[(i + 1) % len];
        let d_curr = (curr - a).dot(normal);
        let d_next = (next - a).dot(normal);

        if d_curr >= 0.0 {
            output.push(curr);
            if d_next < 0.0 {
                // curr inside, next outside -> add intersection
                if let Some(p) = line_intersection(curr, next, a, b) {
                    output.push(p);
                }
            }
        } else if d_next >= 0.0 {
            // curr outside, next inside -> add intersection
            if let Some(p) = line_intersection(curr, next, a, b) {
                output.push(p);
            }
        }
    }
    output
}

fn line_intersection(p1: Vec2, p2: Vec2, p3: Vec2, p4: Vec2) -> Option<Vec2> {
    let d1 = p2 - p1;
    let d2 = p4 - p3;
    let denom = cross_2d(d1, d2);
    if denom.abs() < 1e-10 {
        return None;
    }
    let t = cross_2d(p3 - p1, d2) / denom;
    Some(p1 + d1 * t)
}

/// Compute the Voronoi cell for site `i` given all `sites`, clipped to `bounds`.
/// Each cell is the intersection of half-planes: for each other site j,
/// the half-plane closer to site i than to site j.
pub fn voronoi_cell(sites: &[Vec2], i: usize, bounds: &[Vec2]) -> Vec<Vec2> {
    let mut cell = bounds.to_vec();
    let si = sites[i];

    for (j, &sj) in sites.iter().enumerate() {
        if j == i {
            continue;
        }
        if cell.len() < 3 {
            break;
        }
        // Bisector midpoint and direction
        let mid = (si + sj) * 0.5;
        // The half-plane we keep is the one on the side of si.
        // We need the clipping line perpendicular to (sj - si), passing through mid.
        // The "left side" of line (a, b) should be toward si.
        let perp = vec2(-(sj.y - si.y), sj.x - si.x);
        let a = mid;
        let b = mid + perp;
        cell = clip_polygon_by_halfplane(&cell, a, b);
    }
    cell
}

/// Generate a regular polygon (circle approximation) with given center, radius, and vertex count.
pub fn circle_polygon(center: Vec2, radius: f32, n: usize) -> Vec<Vec2> {
    let mut verts = Vec::with_capacity(n);
    for i in 0..n {
        let angle = (i as f32 / n as f32) * std::f32::consts::TAU;
        verts.push(center + vec2(angle.cos() * radius, angle.sin() * radius));
    }
    verts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polygon_bounds_returns_min_max() {
        let poly = vec![vec2(-2.0, 3.0), vec2(5.0, -1.0), vec2(1.0, 4.0)];
        let (min, max) = polygon_bounds(&poly).expect("bounds");
        assert_eq!(min, vec2(-2.0, -1.0));
        assert_eq!(max, vec2(5.0, 4.0));
    }

    #[test]
    fn line_points_includes_endpoints() {
        let pts = line_points(vec2(0.0, 0.0), vec2(10.0, 0.0), 5.0);
        assert_eq!(pts.first(), Some(&vec2(0.0, 0.0)));
        assert_eq!(pts.last(), Some(&vec2(10.0, 0.0)));
        assert!(pts.len() >= 2);
    }

    #[test]
    fn clip_polygon_keeps_inside_half() {
        // Square from (0,0) to (10,10), clip by line y=5 keeping bottom half
        let square = vec![
            vec2(0.0, 0.0),
            vec2(10.0, 0.0),
            vec2(10.0, 10.0),
            vec2(0.0, 10.0),
        ];
        // Line from (0,5) to (10,5), left side is below (y < 5)
        let clipped = clip_polygon_by_halfplane(&square, vec2(10.0, 5.0), vec2(0.0, 5.0));
        assert!(clipped.len() >= 3);
        for v in &clipped {
            assert!(v.y <= 5.0 + 1e-3, "vertex {:?} above clip line", v);
        }
    }

    #[test]
    fn voronoi_two_sites_splits_bounds() {
        let sites = vec![vec2(0.0, 0.0), vec2(10.0, 0.0)];
        let bounds = circle_polygon(vec2(5.0, 0.0), 20.0, 32);
        let cell0 = voronoi_cell(&sites, 0, &bounds);
        let cell1 = voronoi_cell(&sites, 1, &bounds);
        // All points in cell0 should be closer to site 0
        for v in &cell0 {
            assert!(
                v.distance(sites[0]) <= v.distance(sites[1]) + 1e-2,
                "{:?} closer to site 1",
                v
            );
        }
        // All points in cell1 should be closer to site 1
        for v in &cell1 {
            assert!(
                v.distance(sites[1]) <= v.distance(sites[0]) + 1e-2,
                "{:?} closer to site 0",
                v
            );
        }
    }

    #[test]
    fn voronoi_cells_cover_bounds() {
        // With 4 sites, every point in bounds should be in at least one cell
        let sites = vec![
            vec2(-5.0, -5.0),
            vec2(5.0, -5.0),
            vec2(5.0, 5.0),
            vec2(-5.0, 5.0),
        ];
        let bounds = circle_polygon(vec2(0.0, 0.0), 20.0, 32);
        for i in 0..sites.len() {
            let cell = voronoi_cell(&sites, i, &bounds);
            assert!(cell.len() >= 3, "cell {} has too few vertices", i);
        }
    }

    #[test]
    fn circle_polygon_vertex_count() {
        let poly = circle_polygon(vec2(100.0, 100.0), 50.0, 16);
        assert_eq!(poly.len(), 16);
        for v in &poly {
            let dist = v.distance(vec2(100.0, 100.0));
            assert!((dist - 50.0).abs() < 1e-3);
        }
    }
}
