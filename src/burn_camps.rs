use macroquad::prelude::*;

use crate::camps::{CampConfig, CampSpawns};
use crate::constants;
use crate::geom;
use crate::procmap;
use crate::scenery::{DomeDecoration, SceneryKind, ScenerySpawn, TENT_VARIANT_COUNT};

const ADJECTIVES: [&str; 20] = [
    "Cosmic",
    "Radical",
    "Sacred",
    "Electric",
    "Dusty",
    "Primal",
    "Ethereal",
    "Blazing",
    "Mystical",
    "Golden",
    "Lucid",
    "Astral",
    "Feral",
    "Neon",
    "Ancient",
    "Chromatic",
    "Sublime",
    "Molten",
    "Lunar",
    "Solar",
];

const NOUNS: [&str; 20] = [
    "Phoenix",
    "Lotus",
    "Serpent",
    "Thunder",
    "Mirage",
    "Oasis",
    "Temple",
    "Mandala",
    "Spirit",
    "Nebula",
    "Coyote",
    "Nomad",
    "Crystal",
    "Ember",
    "Horizon",
    "Totem",
    "Echo",
    "Pulse",
    "Zenith",
    "Vortex",
];

const CAMP_TYPES: [&str; 10] = [
    "Camp",
    "Village",
    "Sanctuary",
    "Collective",
    "Lodge",
    "Haven",
    "Circle",
    "Tribe",
    "Domain",
    "Station",
];

pub fn generate_camp_name(rng: &mut u32) -> String {
    let adj = ADJECTIVES[next_usize(rng, ADJECTIVES.len())];
    let noun = NOUNS[next_usize(rng, NOUNS.len())];
    let camp_type = CAMP_TYPES[next_usize(rng, CAMP_TYPES.len())];
    format!("{} {} {}", adj, noun, camp_type)
}

pub fn generate_camp_polygon(
    center: Vec2,
    radius: f32,
    vertex_count: usize,
    rng: &mut u32,
) -> Vec<Vec2> {
    let mut vertices = Vec::with_capacity(vertex_count);
    for i in 0..vertex_count {
        let base_angle = (i as f32 / vertex_count as f32) * std::f32::consts::TAU;
        let angle_jitter = (next_f32(rng) - 0.5) * 0.4;
        let radius_jitter = 0.7 + next_f32(rng) * 0.6;
        let r = radius * radius_jitter;
        let angle = base_angle + angle_jitter;
        vertices.push(center + vec2(angle.cos() * r, angle.sin() * r));
    }
    vertices
}

pub fn generate_camp_spawns(
    vertices: &[Vec2],
    hippie_count: usize,
    rng: &mut u32,
) -> CampSpawns {
    let mut spawns = CampSpawns::default();

    // 1-3 campfires
    let campfire_count = 1 + next_usize(rng, 3);
    for _ in 0..campfire_count {
        let pos = random_point_in_polygon(vertices, rng);
        let scale = 0.8 + next_f32(rng) * 0.8;
        spawns.scenery.push(ScenerySpawn::campfire(pos, scale));
    }

    // 2-6 tents
    let tent_count = 2 + next_usize(rng, 5);
    for _ in 0..tent_count {
        let pos = random_point_in_polygon(vertices, rng);
        let variant = next_usize(rng, TENT_VARIANT_COUNT as usize) as u8;
        spawns.scenery.push(ScenerySpawn::tent(pos, variant));
    }

    // 1-4 chairs
    let chair_count = 1 + next_usize(rng, 4);
    for _ in 0..chair_count {
        let pos = random_point_in_polygon(vertices, rng);
        let rotation = (next_f32(rng) - 0.5) * 2.0;
        spawns.scenery.push(ScenerySpawn::chair(pos, rotation));
    }

    // 0-1 dome (20% chance)
    if next_f32(rng) < 0.2 {
        let pos = random_point_in_polygon(vertices, rng);
        let decorations = if next_f32(rng) < 0.5 {
            vec![DomeDecoration::Crystal]
        } else {
            Vec::new()
        };
        spawns.scenery.push(ScenerySpawn {
            kind: SceneryKind::Dome,
            pos,
            scale: 1.0,
            rotation: 0.0,
            variant: 0,
            decorations,
        });
    }

    // 0-3 trees
    let tree_count = next_usize(rng, 4);
    for _ in 0..tree_count {
        let pos = random_point_in_polygon(vertices, rng);
        let scale = 0.8 + next_f32(rng) * 0.4;
        spawns.scenery.push(ScenerySpawn {
            kind: SceneryKind::Tree,
            pos,
            scale,
            rotation: 0.0,
            variant: 0,
            decorations: Vec::new(),
        });
    }

    // 2-5 flags per camp
    let flag_count = 2 + next_usize(rng, 4);
    for _ in 0..flag_count {
        let pos = random_point_in_polygon(vertices, rng);
        spawns.flags.push(pos);
    }

    // Pentagram: 5 flags in pentagon pattern near camp center
    let pentagram_center = polygon_centroid(vertices);
    let pentagram_radius = constants::BURN_PENTAGRAM_RADIUS;
    let angle_offset = next_f32(rng) * std::f32::consts::TAU;
    for i in 0..5 {
        let angle = angle_offset + (i as f32 / 5.0) * std::f32::consts::TAU;
        let jitter = 1.0 + (next_f32(rng) - 0.5) * 0.15;
        let r = pentagram_radius * jitter;
        let pos = pentagram_center + vec2(angle.cos() * r, angle.sin() * r);
        spawns.flags.push(pos);
    }

    // Hippie spawn positions
    for _ in 0..hippie_count {
        spawns.hippies.push(random_point_in_polygon(vertices, rng));
    }

    spawns
}

fn leak_string(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub fn generate_burn_camps(
    effigy_pos: Vec2,
    camp_count: usize,
    hippies_per_camp: usize,
    seed: u32,
) -> Vec<CampConfig> {
    let mut rng = seed;
    let ring_radius = procmap::core_radius(camp_count) * 0.6;

    // Generate camp centers on a ring around the effigy
    let mut centers = Vec::with_capacity(camp_count);
    for i in 0..camp_count {
        let angle = (i as f32 / camp_count as f32) * std::f32::consts::TAU;
        let jitter = (next_f32(&mut rng) - 0.5) * 0.3 / (camp_count as f32).max(1.0);
        let actual_angle = angle + jitter;

        let radial_jitter = 1.0 + (next_f32(&mut rng) - 0.5) * 0.2;
        let r = ring_radius * radial_jitter;

        centers.push(effigy_pos + vec2(actual_angle.cos() * r, actual_angle.sin() * r));
    }

    // Compute Voronoi cells with effigy as a virtual site (index 0)
    let mut all_sites = Vec::with_capacity(camp_count + 1);
    all_sites.push(effigy_pos); // virtual site for central clearing
    all_sites.extend_from_slice(&centers);

    // Bounding polygon: large circle around effigy
    let bound_radius = ring_radius * 2.5;
    let bounds = geom::circle_polygon(effigy_pos, bound_radius, 32);

    let mut camps = Vec::with_capacity(camp_count);
    for i in 0..camp_count {
        let site_index = i + 1; // offset by 1 because effigy is site 0
        let voronoi_verts = geom::voronoi_cell(&all_sites, site_index, &bounds);
        let vertices = shrink_polygon(&voronoi_verts, constants::BURN_CAMP_SHRINK);

        // Advance RNG to stay deterministic (consume same number of values as before)
        let _ = next_f32(&mut rng); // was camp_radius
        let _ = next_usize(&mut rng, 4); // was vertex_count
        // consume what generate_camp_polygon would have used
        for _ in 0..8 {
            let _ = next_f32(&mut rng);
        }

        let hue = next_f32(&mut rng) * 0.15 + 0.05;
        let sat = 0.3 + next_f32(&mut rng) * 0.3;
        let val = 0.4 + next_f32(&mut rng) * 0.2;
        let color = hsv_to_color(hue, sat, val);

        let name = generate_camp_name(&mut rng);
        let notice = name.clone();
        let spawns = generate_camp_spawns(&vertices, hippies_per_camp, &mut rng);

        camps.push(CampConfig {
            name: leak_string(name),
            vertices,
            color,
            notice_text: leak_string(notice),
            spawns,
        });
    }

    camps
}

pub fn pentagram_blueprint(centroid: Vec2, seed: u32) -> Vec<Vec2> {
    let mut rng = seed;
    let angle_offset = next_f32(&mut rng) * std::f32::consts::TAU;
    let radius = constants::BURN_PENTAGRAM_RADIUS;
    (0..5)
        .map(|i| {
            let angle = angle_offset + (i as f32 / 5.0) * std::f32::consts::TAU;
            centroid + vec2(angle.cos() * radius, angle.sin() * radius)
        })
        .collect()
}

pub fn polygon_centroid(vertices: &[Vec2]) -> Vec2 {
    if vertices.is_empty() {
        return Vec2::ZERO;
    }
    let sum: Vec2 = vertices.iter().copied().sum();
    sum / vertices.len() as f32
}

fn shrink_polygon(vertices: &[Vec2], factor: f32) -> Vec<Vec2> {
    if vertices.is_empty() {
        return Vec::new();
    }
    let sum: Vec2 = vertices.iter().copied().sum();
    let centroid = sum / vertices.len() as f32;
    vertices
        .iter()
        .map(|v| centroid + (*v - centroid) * factor)
        .collect()
}

fn hsv_to_color(h: f32, s: f32, v: f32) -> Color {
    let h = (h % 1.0 + 1.0) % 1.0;
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

fn random_point_in_polygon(vertices: &[Vec2], rng: &mut u32) -> Vec2 {
    let Some((min, max)) = geom::polygon_bounds(vertices) else {
        return Vec2::ZERO;
    };
    for _ in 0..32 {
        let x = min.x + next_f32(rng) * (max.x - min.x);
        let y = min.y + next_f32(rng) * (max.y - min.y);
        let candidate = vec2(x, y);
        if geom::point_in_polygon(candidate, vertices) {
            return candidate;
        }
    }
    vertices[0]
}

fn next_f32(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    let v = (*state >> 8) as f32;
    v / ((u32::MAX >> 8) as f32 + 1.0)
}

fn next_usize(state: &mut u32, max: usize) -> usize {
    if max == 0 {
        return 0;
    }
    (next_f32(state) * max as f32) as usize % max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camp_name_is_three_words() {
        let mut rng = 42u32;
        for _ in 0..20 {
            let name = generate_camp_name(&mut rng);
            let words: Vec<&str> = name.split_whitespace().collect();
            assert_eq!(words.len(), 3, "Name '{}' is not 3 words", name);
        }
    }

    #[test]
    fn camp_polygon_has_correct_vertex_count() {
        let mut rng = 42u32;
        for count in 5..=8 {
            let verts = generate_camp_polygon(vec2(100.0, 100.0), 80.0, count, &mut rng);
            assert_eq!(verts.len(), count);
        }
    }

    #[test]
    fn camp_polygon_vertices_near_center() {
        let mut rng = 42u32;
        let center = vec2(500.0, 500.0);
        let radius = 100.0;
        let verts = generate_camp_polygon(center, radius, 6, &mut rng);
        for v in &verts {
            let dist = v.distance(center);
            assert!(
                dist < radius * 2.0,
                "Vertex too far from center: {} > {}",
                dist,
                radius * 2.0
            );
        }
    }

    #[test]
    fn camp_placement_ring_has_correct_count() {
        let camps = generate_burn_camps(vec2(1000.0, 1000.0), 5, 3, 42);
        assert_eq!(camps.len(), 5);
    }

    #[test]
    fn camp_placement_ring_evenly_spaced() {
        let effigy = vec2(1000.0, 1000.0);
        let camps = generate_burn_camps(effigy, 6, 2, 42);
        let mut angles: Vec<f32> = camps
            .iter()
            .map(|c| {
                let center = camp_center(&c.vertices);
                (center.y - effigy.y).atan2(center.x - effigy.x)
            })
            .collect();
        angles.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let expected_gap = std::f32::consts::TAU / 6.0;
        for i in 0..angles.len() {
            let next = (i + 1) % angles.len();
            let mut gap = angles[next] - angles[i];
            if gap < 0.0 {
                gap += std::f32::consts::TAU;
            }
            assert!(
                (gap - expected_gap).abs() < expected_gap * 0.6,
                "Angular gap {} too far from expected {}",
                gap,
                expected_gap
            );
        }
    }

    #[test]
    fn camp_spawns_have_minimum_items() {
        let camps = generate_burn_camps(vec2(1000.0, 1000.0), 3, 5, 42);
        for camp in &camps {
            let campfires = camp
                .spawns
                .scenery
                .iter()
                .filter(|s| s.kind == SceneryKind::Campfire)
                .count();
            let tents = camp
                .spawns
                .scenery
                .iter()
                .filter(|s| s.kind == SceneryKind::Tent)
                .count();
            assert!(campfires >= 1, "Camp '{}' has no campfires", camp.name);
            assert!(tents >= 2, "Camp '{}' has fewer than 2 tents", camp.name);
            assert!(
                !camp.spawns.hippies.is_empty(),
                "Camp '{}' has no hippies",
                camp.name
            );
            assert!(
                camp.spawns.hippies.len() == 5,
                "Camp '{}' has {} hippies, expected 5",
                camp.name,
                camp.spawns.hippies.len()
            );
        }
    }

    #[test]
    fn hippie_positions_inside_camp_polygon() {
        let camps = generate_burn_camps(vec2(1000.0, 1000.0), 3, 5, 42);
        for camp in &camps {
            for pos in &camp.spawns.hippies {
                assert!(
                    geom::point_in_polygon(*pos, &camp.vertices),
                    "Hippie at {:?} not inside camp '{}'",
                    pos,
                    camp.name
                );
            }
        }
    }

    #[test]
    fn flag_positions_near_camp() {
        let effigy = vec2(1000.0, 1000.0);
        let camps = generate_burn_camps(effigy, 3, 5, 42);
        for camp in &camps {
            // Each camp has 2-5 random flags (inside) + 5 pentagram flags (may be outside)
            assert!(camp.spawns.flags.len() >= 7, "Camp '{}' has too few flags", camp.name);
            let centroid = camp_center(&camp.vertices);
            for pos in &camp.spawns.flags {
                // All flags should be reasonably near the camp centroid
                let dist = pos.distance(centroid);
                assert!(
                    dist < 300.0,
                    "Flag at {:?} too far from camp '{}' centroid ({:?}), dist={}",
                    pos, camp.name, centroid, dist
                );
            }
            // At least some flags should be inside the camp polygon
            let inside_count = camp.spawns.flags.iter()
                .filter(|pos| geom::point_in_polygon(**pos, &camp.vertices))
                .count();
            assert!(inside_count >= 2, "Camp '{}' has too few flags inside polygon", camp.name);
        }
    }

    #[test]
    fn single_camp_generates_successfully() {
        let camps = generate_burn_camps(vec2(500.0, 500.0), 1, 1, 99);
        assert_eq!(camps.len(), 1);
        assert!(!camps[0].name.is_empty());
    }

    #[test]
    fn max_camps_generates_successfully() {
        let camps = generate_burn_camps(vec2(2000.0, 2000.0), 20, 2, 1234);
        assert_eq!(camps.len(), 20);
        // All names should be unique
        let names: Vec<&str> = camps.iter().map(|c| c.name).collect();
        for i in 0..names.len() {
            for j in (i + 1)..names.len() {
                assert_ne!(names[i], names[j], "Duplicate camp name: {}", names[i]);
            }
        }
    }

    #[test]
    fn pentagram_blueprint_generates_five_positions() {
        let centroid = vec2(500.0, 500.0);
        let slots = pentagram_blueprint(centroid, 42);
        assert_eq!(slots.len(), 5);
        for slot in &slots {
            let dist = slot.distance(centroid);
            assert!(
                (dist - constants::BURN_PENTAGRAM_RADIUS).abs() < 1.0,
                "Slot {:?} not at expected radius: {}",
                slot,
                dist
            );
        }
        // Slots should be roughly evenly spaced angularly
        let mut angles: Vec<f32> = slots
            .iter()
            .map(|s| (s.y - centroid.y).atan2(s.x - centroid.x))
            .collect();
        angles.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let expected_gap = std::f32::consts::TAU / 5.0;
        for i in 0..5 {
            let next = (i + 1) % 5;
            let mut gap = angles[next] - angles[i];
            if gap < 0.0 {
                gap += std::f32::consts::TAU;
            }
            assert!(
                (gap - expected_gap).abs() < 0.1,
                "Angular gap {} too far from expected {}",
                gap,
                expected_gap
            );
        }
    }

    fn camp_center(vertices: &[Vec2]) -> Vec2 {
        let sum: Vec2 = vertices.iter().copied().sum();
        sum / vertices.len() as f32
    }
}
