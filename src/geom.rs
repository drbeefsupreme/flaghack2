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
}
