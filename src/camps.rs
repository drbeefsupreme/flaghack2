use macroquad::prelude::*;

use crate::geom;
use crate::scenery::{ScenerySpawn, TENT_VARIANT_COUNT};

#[derive(Clone, Debug)]
pub struct CampConfig {
    pub name: &'static str,
    pub vertices: Vec<Vec2>,
    pub color: Color,
    pub notice_text: &'static str,
    pub spawns: CampSpawns,
}

#[derive(Clone, Debug, Default)]
pub struct CampSpawns {
    pub scenery: Vec<ScenerySpawn>,
    pub flags: Vec<Vec2>,
    pub hippies: Vec<Vec2>,
}

const T3MPCAMP_NAME: &str = "t3mpcamp";
const T3MPCAMP_NOTICE: &str = "t3mpcamp.com";
const T3MPCAMP_COLOR: Color = Color::new(0.1, 0.6, 0.2, 1.0);
const T3MPCAMP_VERTICES: [Vec2; 5] = [
    Vec2::new(4858.0, 3168.0),
    Vec2::new(5042.0, 3107.0),
    Vec2::new(5123.0, 3345.0),
    Vec2::new(5054.0, 3367.0),
    Vec2::new(4911.0, 3322.0),
];
const T3MPCAMP_CAMPFIRE_POS: Vec2 = Vec2::new(4982.0, 3233.0);
const T3MPCAMP_TENT_SPACING: f32 = 14.0;
const T3MPCAMP_ROW1_START: Vec2 = Vec2::new(4926.0, 3300.0);
const T3MPCAMP_ROW1_END: Vec2 = Vec2::new(5000.0, 3300.0);
const T3MPCAMP_ROW2_START: Vec2 = Vec2::new(4926.0, 3317.0);
const T3MPCAMP_ROW2_END: Vec2 = Vec2::new(5000.0, 3317.0);
const T3MPCAMP_CAMPFIRE_SCALE: f32 = 1.5;

const GEORGIA_PEANUTS_NAME: &str = "Georgia Peanuts";
const GEORGIA_PEANUTS_NOTICE: &str = "Georgia Peanuts";
const GEORGIA_PEANUTS_COLOR: Color = Color::new(0.12, 0.55, 0.24, 1.0);
const GEORGIA_PEANUTS_VERTICES: [Vec2; 4] = [
    Vec2::new(5123.0, 3345.0),
    Vec2::new(5042.0, 3107.0),
    Vec2::new(5255.0, 3037.0),
    Vec2::new(5329.0, 3274.0),
];

const DEBUSSY_BUS_NAME: &str = "DeBussy Bus Station";
const DEBUSSY_BUS_NOTICE: &str = "DeBussy Bus Station";
const DEBUSSY_BUS_COLOR: Color = Color::new(0.11, 0.58, 0.23, 1.0);
const DEBUSSY_BUS_VERTICES: [Vec2; 4] = [
    Vec2::new(4850.0, 3134.0),
    Vec2::new(4784.0, 2933.0),
    Vec2::new(4913.0, 2894.0),
    Vec2::new(4975.0, 3092.0),
];

pub fn camp_configs() -> Vec<CampConfig> {
    let mut camps = Vec::new();

    camps.push(CampConfig {
        name: T3MPCAMP_NAME,
        vertices: T3MPCAMP_VERTICES.to_vec(),
        color: T3MPCAMP_COLOR,
        notice_text: T3MPCAMP_NOTICE,
        spawns: t3mpcamp_spawns(),
    });

    camps.push(CampConfig {
        name: GEORGIA_PEANUTS_NAME,
        vertices: GEORGIA_PEANUTS_VERTICES.to_vec(),
        color: GEORGIA_PEANUTS_COLOR,
        notice_text: GEORGIA_PEANUTS_NOTICE,
        spawns: georgia_peanuts_spawns(),
    });

    camps.push(CampConfig {
        name: DEBUSSY_BUS_NAME,
        vertices: DEBUSSY_BUS_VERTICES.to_vec(),
        color: DEBUSSY_BUS_COLOR,
        notice_text: DEBUSSY_BUS_NOTICE,
        spawns: debussy_bus_spawns(),
    });

    camps
}

pub fn collect_scenery_spawns(camps: &[CampConfig]) -> Vec<ScenerySpawn> {
    let mut spawns = Vec::new();
    for camp in camps {
        spawns.extend(camp.spawns.scenery.iter().cloned());
    }
    spawns
}

pub fn collect_flag_spawns(camps: &[CampConfig]) -> Vec<Vec2> {
    let mut flags = Vec::new();
    for camp in camps {
        flags.extend(camp.spawns.flags.iter().copied());
    }
    flags
}

pub fn collect_camp_vertices(camps: &[CampConfig]) -> Vec<Vec<Vec2>> {
    camps.iter().map(|camp| camp.vertices.clone()).collect()
}

fn t3mpcamp_spawns() -> CampSpawns {
    let mut spawns = CampSpawns::default();

    spawns.scenery.push(ScenerySpawn::campfire(
        T3MPCAMP_CAMPFIRE_POS,
        T3MPCAMP_CAMPFIRE_SCALE,
    ));

    let rows = [
        (T3MPCAMP_ROW1_START, T3MPCAMP_ROW1_END),
        (T3MPCAMP_ROW2_START, T3MPCAMP_ROW2_END),
    ];
    let mut tent_index = 0;
    for (_row_index, (start, end)) in rows.iter().enumerate() {
        let positions = geom::line_points(*start, *end, T3MPCAMP_TENT_SPACING);
        for pos in positions {
            let variant = (tent_index % TENT_VARIANT_COUNT as usize) as u8;
            spawns.scenery.push(ScenerySpawn::tent(pos, variant));
            tent_index += 1;
        }
    }

    spawns.hippies = vec![
        T3MPCAMP_CAMPFIRE_POS + vec2(-28.0, -16.0),
        T3MPCAMP_CAMPFIRE_POS + vec2(26.0, 14.0),
        T3MPCAMP_CAMPFIRE_POS + vec2(-12.0, 22.0),
        T3MPCAMP_CAMPFIRE_POS + vec2(18.0, -24.0),
    ];

    spawns
}

fn georgia_peanuts_spawns() -> CampSpawns {
    let mut spawns = CampSpawns::default();

    spawns.scenery.extend([
        ScenerySpawn::campfire(vec2(5200.0, 3180.0), 1.0),
        ScenerySpawn::campfire(vec2(5280.0, 3230.0), 1.0),
        ScenerySpawn::chair(vec2(5170.0, 3210.0), 0.4),
        ScenerySpawn::chair(vec2(5190.0, 3160.0), -0.3),
        ScenerySpawn::chair(vec2(5255.0, 3220.0), 0.6),
        ScenerySpawn::chair(vec2(5295.0, 3250.0), -0.2),
        ScenerySpawn::tent(vec2(5150.0, 3300.0), 0),
        ScenerySpawn::tent(vec2(5180.0, 3320.0), 1),
        ScenerySpawn::tent(vec2(5210.0, 3290.0), 2),
        ScenerySpawn::tent(vec2(5230.0, 3300.0), 3),
        ScenerySpawn::tent(vec2(5190.0, 3265.0), 4),
    ]);

    spawns.flags = vec![
        vec2(5090.0, 3200.0),
        vec2(5160.0, 3220.0),
        vec2(5200.0, 3180.0),
        vec2(5220.0, 3100.0),
        vec2(5270.0, 3150.0),
    ];

    spawns.hippies = vec![
        vec2(5175.0, 3190.0),
        vec2(5215.0, 3205.0),
        vec2(5265.0, 3235.0),
    ];

    spawns
}

fn debussy_bus_spawns() -> CampSpawns {
    let mut spawns = CampSpawns::default();

    spawns.scenery.extend([
        ScenerySpawn::campfire(vec2(4860.0, 3050.0), 1.0),
        ScenerySpawn::campfire(vec2(4935.0, 3055.0), 1.0),
        ScenerySpawn::chair(vec2(4880.0, 2990.0), 0.3),
        ScenerySpawn::chair(vec2(4825.0, 3000.0), -0.4),
        ScenerySpawn::chair(vec2(4910.0, 3005.0), 0.5),
        ScenerySpawn::tent(vec2(4870.0, 3080.0), 0),
        ScenerySpawn::tent(vec2(4895.0, 3020.0), 1),
        ScenerySpawn::tent(vec2(4925.0, 2965.0), 2),
        ScenerySpawn::tent(vec2(4950.0, 3060.0), 3),
    ]);

    spawns.flags = vec![
        vec2(4840.0, 3040.0),
        vec2(4860.0, 3050.0),
        vec2(4890.0, 2960.0),
        vec2(4920.0, 3040.0),
        vec2(4940.0, 3070.0),
    ];

    spawns.hippies = vec![
        vec2(4860.0, 3050.0),
        vec2(4895.0, 3020.0),
        vec2(4935.0, 3055.0),
    ];

    spawns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn georgia_peanuts_has_expected_spawns() {
        let camps = camp_configs();
        let georgia = camps
            .iter()
            .find(|camp| camp.name == GEORGIA_PEANUTS_NAME)
            .expect("Georgia Peanuts camp");

        let campfires = georgia
            .spawns
            .scenery
            .iter()
            .filter(|spawn| spawn.kind == crate::scenery::SceneryKind::Campfire)
            .count();
        let chairs = georgia
            .spawns
            .scenery
            .iter()
            .filter(|spawn| spawn.kind == crate::scenery::SceneryKind::Chair)
            .count();
        let tents = georgia
            .spawns
            .scenery
            .iter()
            .filter(|spawn| spawn.kind == crate::scenery::SceneryKind::Tent)
            .count();

        assert!(campfires >= 2);
        assert!(chairs >= 3);
        assert!(tents >= 3);
        assert!(!georgia.spawns.flags.is_empty());
        assert!(!georgia.spawns.hippies.is_empty());
    }

    #[test]
    fn debussy_bus_has_expected_spawns() {
        let camps = camp_configs();
        let bus = camps
            .iter()
            .find(|camp| camp.name == DEBUSSY_BUS_NAME)
            .expect("DeBussy Bus Station camp");

        let campfires = bus
            .spawns
            .scenery
            .iter()
            .filter(|spawn| spawn.kind == crate::scenery::SceneryKind::Campfire)
            .count();
        let chairs = bus
            .spawns
            .scenery
            .iter()
            .filter(|spawn| spawn.kind == crate::scenery::SceneryKind::Chair)
            .count();
        let tents = bus
            .spawns
            .scenery
            .iter()
            .filter(|spawn| spawn.kind == crate::scenery::SceneryKind::Tent)
            .count();

        assert!(campfires >= 2);
        assert!(chairs >= 2);
        assert!(tents >= 3);
        assert!(!bus.spawns.flags.is_empty());
        assert!(!bus.spawns.hippies.is_empty());
    }
}
