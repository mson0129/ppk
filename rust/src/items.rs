//! Candy chains, egg hatching, support checks and spikes.
use crate::state::{FloatingScore, GameState, Hatchling};
const CANDY_POINTS: [u32; 12] = [1, 2, 3, 5, 10, 20, 30, 50, 100, 200, 300, 500];
#[derive(Clone, Copy, Debug)]
pub enum Pickup {
    Candy,
    Bonus,
    Egg,
}
#[derive(Clone, Copy, Debug)]
pub struct Collection {
    pub kind: Pickup,
    pub erase_x: i16,
    pub erase_y: i16,
}
impl GameState {
    pub fn collect_and_check_hazards(&mut self) -> Option<Collection> {
        let p = self.player;
        if p.falling != 0 || p.respawn != 0 || p.climbing != 0 || !(0..=1).contains(&p.facing) {
            return None;
        }
        let right = p.facing == 1;
        let column = if right {
            (p.x + 8) >> 3
        } else {
            (p.x - 8) >> 3
        };
        let row = (p.y - 16) >> 3;
        let cell = (row as i32 * 40 + column as i32) as usize;
        let tile = self.tiles.get(cell).copied().unwrap_or(0);
        let pickup = match (right, tile) {
            (false, 5) | (true, 8) => Some(Pickup::Candy),
            (false, 7) | (true, 10) => Some(Pickup::Bonus),
            (false, 6) | (true, 9) => Some(Pickup::Egg),
            _ => None,
        };
        if let Some(kind) = pickup {
            self.tiles[cell] = 0;
            self.tiles[if right { cell - 1 } else { cell + 1 }] = 0;
            match kind {
                Pickup::Candy | Pickup::Bonus => {
                    self.floating_cursor += 1;
                    if self.floating_cursor == 8 {
                        self.floating_cursor = 0;
                    }
                    self.floating_scores[self.floating_cursor as usize] = FloatingScore {
                        x: p.x + if right { 4 } else { 0 },
                        y: p.y + 8,
                        sprite: self.candy_chain + 36,
                        active: 1,
                    };
                    self.score = self
                        .score
                        .wrapping_add(CANDY_POINTS[self.candy_chain as usize]);
                    self.score_dirty = 1;
                    self.candy_chain += 1;
                    if self.candy_chain >= 12 {
                        self.candy_chain = 11;
                    }
                    if matches!(kind, Pickup::Candy) {
                        self.remaining_candy -= 1;
                    }
                }
                Pickup::Egg => {
                    self.hatchlings[self.hatchling_count as usize] = Hatchling {
                        x: p.x,
                        y: p.y,
                        right: if right { 0 } else { 1 },
                        sprite: if right { 4 } else { 0 },
                        active: 1,
                        age: 0,
                        reserved: self.hatchlings[self.hatchling_count as usize].reserved,
                    };
                    self.hatchling_count += 1;
                    if self.hatchling_count >= 10 {
                        self.hatchling_count = 0;
                    }
                }
            }
        }
        // These probes intentionally use facing-dependent offsets from the DOS game.
        let support_x = p.x + if right { 1 } else { -1 };
        if self.tile(support_x, p.y - 8) == 0 && self.player.jump == 0 && self.player.climbing == 0
        {
            self.player.falling = 1;
        }
        let spike_x = p.x + if right { -1 } else { -2 };
        if self.tile(spike_x, p.y - 16) == 3 && self.player.climbing == 0 {
            self.player.sprite = 32;
            self.player.respawn = 1;
        }
        pickup.map(|kind| Collection {
            kind,
            erase_x: p.x + if right { 8 } else { 0 },
            erase_y: ((p.y + 6) >> 3) << 3,
        })
    }
}
