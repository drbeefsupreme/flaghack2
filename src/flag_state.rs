use macroquad::prelude::{Rect, Vec2};

use crate::flags;

#[derive(Clone, Debug)]
pub struct FlagState {
    ground: Vec<flags::Flag>,
    player: u32,
    total: u32,
}

impl FlagState {
    pub fn new(ground: Vec<flags::Flag>, player: u32, total: u32) -> Self {
        debug_assert!(total >= ground.len() as u32 + player);
        Self {
            ground,
            player,
            total,
        }
    }

    pub fn ground_flags(&self) -> &[flags::Flag] {
        &self.ground
    }

    pub fn player_inventory(&self) -> u32 {
        self.player
    }

    pub fn current_total(&self, hippie_flags: u32) -> u32 {
        self.ground.len() as u32 + self.player + hippie_flags
    }

    pub fn debug_assert_invariant(&self, hippie_flags: u32) {
        debug_assert_eq!(self.total, self.current_total(hippie_flags));
    }

    pub fn try_place_from_player(&mut self, origin: Vec2, offset: Vec2, field: Rect) -> bool {
        flags::try_place_flag(&mut self.ground, &mut self.player, origin, offset, field)
    }

    pub fn try_pickup_to_player(&mut self, origin: Vec2, radius: f32) -> bool {
        if flags::try_pickup_flag(&mut self.ground, origin, radius) {
            self.player = self.player.saturating_add(1);
            true
        } else {
            false
        }
    }

    pub fn steal_from_hippie(&mut self, hippie_flags: &mut u8) -> bool {
        if *hippie_flags == 0 {
            return false;
        }
        *hippie_flags = hippie_flags.saturating_sub(1);
        self.player = self.player.saturating_add(1);
        true
    }

    pub fn transfer_ground_to_hippie(
        &mut self,
        hippie_flags: &mut u8,
        capacity: u8,
        pos: Vec2,
        radius: f32,
    ) -> bool {
        let mut picked = false;
        let mut index = 0;
        while index < self.ground.len() && *hippie_flags < capacity {
            let flag_pos = self.ground[index].pos;
            if flag_pos.distance(pos) <= radius {
                self.ground.swap_remove(index);
                *hippie_flags = hippie_flags.saturating_add(1);
                picked = true;
                continue;
            }
            index += 1;
        }
        picked
    }

    pub fn drop_from_hippie(&mut self, hippie_flags: &mut u8, count: u8, pos: Vec2) -> u8 {
        let drop = count.min(*hippie_flags);
        if drop == 0 {
            return 0;
        }
        *hippie_flags -= drop;
        for _ in 0..drop {
            self.ground.push(flags::make_flag(pos));
        }
        drop
    }

    pub fn steal_from_player_to_hippie(
        &mut self,
        hippie_flags: &mut u8,
        capacity: u8,
        pos: Vec2,
        max_count: u32,
    ) -> u32 {
        let stolen = max_count.min(self.player);
        if stolen == 0 {
            return 0;
        }
        self.player -= stolen;

        let mut remaining = stolen;
        while remaining > 0 {
            if *hippie_flags < capacity {
                *hippie_flags = hippie_flags.saturating_add(1);
            } else {
                self.ground.push(flags::make_flag(pos));
            }
            remaining -= 1;
        }

        stolen
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::prelude::vec2;

    #[test]
    fn place_from_player_moves_flag_to_ground() {
        let mut state = FlagState::new(Vec::new(), 1, 1);
        let field = Rect::new(0.0, 0.0, 100.0, 100.0);
        let placed = state.try_place_from_player(vec2(10.0, 10.0), vec2(0.0, 0.0), field);
        assert!(placed);
        assert_eq!(state.player_inventory(), 0);
        assert_eq!(state.ground_flags().len(), 1);
    }

    #[test]
    fn pickup_to_player_moves_flag_from_ground() {
        let mut state = FlagState::new(vec![flags::make_flag(vec2(5.0, 5.0))], 0, 1);
        let picked = state.try_pickup_to_player(vec2(5.0, 5.0), 10.0);
        assert!(picked);
        assert_eq!(state.player_inventory(), 1);
        assert_eq!(state.ground_flags().len(), 0);
    }

    #[test]
    fn steal_from_hippie_transfers_to_player() {
        let mut state = FlagState::new(Vec::new(), 0, 1);
        let mut hippie_flags = 1;
        let stolen = state.steal_from_hippie(&mut hippie_flags);
        assert!(stolen);
        assert_eq!(state.player_inventory(), 1);
        assert_eq!(hippie_flags, 0);
    }

    #[test]
    fn steal_from_player_overflow_drops_to_ground() {
        let mut state = FlagState::new(Vec::new(), 2, 4);
        let mut hippie_flags = 2;
        let stolen = state.steal_from_player_to_hippie(&mut hippie_flags, 2, vec2(1.0, 1.0), 2);
        assert_eq!(stolen, 2);
        assert_eq!(state.player_inventory(), 0);
        assert_eq!(hippie_flags, 2);
        assert_eq!(state.ground_flags().len(), 2);
    }

    #[test]
    fn transfer_ground_to_hippie_picks_multiple() {
        let mut state = FlagState::new(
            vec![
                flags::make_flag(vec2(0.0, 0.0)),
                flags::make_flag(vec2(1.0, 0.0)),
                flags::make_flag(vec2(100.0, 100.0)),
            ],
            0,
            3,
        );
        let mut hippie_flags = 0;
        let picked = state.transfer_ground_to_hippie(&mut hippie_flags, 2, vec2(0.0, 0.0), 5.0);
        assert!(picked);
        assert_eq!(hippie_flags, 2);
        assert_eq!(state.ground_flags().len(), 1);
    }

    #[test]
    fn drop_from_hippie_spawns_ground_flags() {
        let mut state = FlagState::new(Vec::new(), 0, 2);
        let mut hippie_flags = 2;
        let dropped = state.drop_from_hippie(&mut hippie_flags, 1, vec2(2.0, 2.0));
        assert_eq!(dropped, 1);
        assert_eq!(hippie_flags, 1);
        assert_eq!(state.ground_flags().len(), 1);
    }
}
