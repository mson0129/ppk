//! Walking, ladder alignment and the original discrete jump trajectories.
use crate::{
    audio::Tune,
    state::{GameState, Player},
};
#[derive(Clone, Copy, Default)]
pub struct Input {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub enter: bool,
}
const SHORT_X: [i16; 9] = [4, 3, 3, 3, 2, 2, 2, 2, 3];
const SHORT_Y: [i16; 9] = [3, 2, 1, 1, 0, 0, -2, -2, -3];
const SHORT_RIGHT: [i16; 9] = [16, 16, 17, 17, 18, 18, 19, 19, 8];
const SHORT_LEFT: [i16; 9] = [20, 20, 21, 21, 22, 22, 23, 23, 12];
const VERTICAL_Y: [i16; 9] = [2, 2, 1, 1, 1, 0, -2, -2, -3];
const VERTICAL_SPRITE: [i16; 9] = [28, 28, 29, 29, 28, 29, 29, 28, 28];
const LONG_X: [i16; 14] = [4, 4, 3, 3, 3, 2, 1, 1, 1, 2, 2, 2, 3, 4];
const LONG_Y: [i16; 14] = [3, 3, 3, 2, 2, 0, 0, 0, 0, -1, -2, -3, -3, -4];
const LONG_RIGHT: [i16; 14] = [16, 16, 16, 17, 17, 17, 17, 17, 18, 18, 18, 19, 19, 9];
const LONG_LEFT: [i16; 14] = [20, 20, 20, 21, 21, 21, 21, 21, 22, 22, 22, 23, 23, 12];
fn cycle(sprite: i16, first: i16, end: i16) -> i16 {
    if sprite >= end || sprite < first {
        first
    } else {
        sprite
    }
}
fn tile(tiles: &[u8], x: i16, y: i16) -> u8 {
    let i = (y as i32 >> 3) * 40 + (x as i32 >> 3);
    tiles.get(i as usize).copied().unwrap_or(0)
}
impl GameState {
    /// Returns the opening note sequence of a newly initiated jump, if any.
    pub fn move_player(&mut self, input: Input) -> Option<Tune> {
        let mut p = self.player;
        let song = move_player(&mut p, &self.tiles, input, self.music_playing != 0);
        self.player = p;
        song
    }
}
fn move_player(p: &mut Player, tiles: &[u8], key: Input, music_playing: bool) -> Option<Tune> {
    if p.falling != 0 {
        p.y = p.y.wrapping_add(2);
        p.death_animation_tick += 1;
        if p.death_animation_tick == 2 {
            p.sprite += 1;
            p.death_animation_tick = 0;
        }
        p.sprite = cycle(p.sprite, 32, 36);
        if p.y >= 168 {
            p.respawn = 1;
            p.falling = 0;
            p.y = 168;
        }
        return None;
    }
    if p.jump == 0 {
        if key.up {
            if tile(tiles, p.x, p.y - 10) == 2 {
                p.y -= p.climb_step;
                p.sprite = cycle(p.sprite + 1, 30, 32);
                p.climbing = 1;
            }
            if (p.y - 24) % 32 == 16 && p.climbing != 0 {
                p.climbing = 0;
                p.sprite = 28;
                p.facing = 2;
            }
        }
        if key.down {
            if tile(tiles, p.x, p.y - 8) == 2 {
                p.y += p.climb_step;
                p.sprite = cycle(p.sprite + 1, 30, 32);
                p.climbing = 1;
            }
            if (p.y - 24) % 32 == 16 && p.climbing != 0 {
                p.climbing = 0;
                p.sprite = 28;
                p.facing = 2;
            }
        }
        if p.climbing != 0 {
            return None;
        }
        if key.left {
            if p.facing == 0 && tile(tiles, p.x - 8, p.y - 16) != 1 {
                p.x -= p.walk_step;
            }
            p.facing = 0;
            p.sprite = cycle(p.sprite + 1, 12, 16);
        }
        if key.right {
            if p.facing == 1 && tile(tiles, p.x + 8, p.y - 16) != 1 {
                p.x += p.walk_step;
            }
            p.facing = 1;
            p.sprite = cycle(p.sprite + 1, 8, 12);
        }
        if key.jump {
            p.jump_tick = 0;
            if key.left || key.right {
                p.jump = 2;
                return if music_playing {
                    None
                } else {
                    Some(Tune::RunningJump)
                };
            }
            p.jump = if p.facing == 2 { 3 } else { 1 };
            return if music_playing {
                None
            } else {
                Some(Tune::StandingJump)
            };
        }
        return None;
    }
    if p.jump == 2 {
        let i = p.jump_tick as usize;
        p.sprite = if p.facing == 0 {
            LONG_LEFT[i]
        } else {
            LONG_RIGHT[i]
        };
        let dx = if p.facing == 0 { -8 } else { 8 };
        if tile(tiles, p.x + dx, p.y - 16) != 1 {
            p.x += if p.facing == 0 { -LONG_X[i] } else { LONG_X[i] };
        }
        p.y -= LONG_Y[i];
        p.jump_tick += 1;
        if p.jump_tick >= 14 {
            p.jump = 0;
        }
    }
    if p.jump == 1 {
        let i = p.jump_tick as usize;
        p.sprite = if p.facing == 0 {
            SHORT_LEFT[i]
        } else {
            SHORT_RIGHT[i]
        };
        let dx = if p.facing == 0 { -8 } else { 8 };
        if tile(tiles, p.x + dx, p.y - 16) != 1 {
            p.x += if p.facing == 0 {
                -SHORT_X[i]
            } else {
                SHORT_X[i]
            };
        }
        p.y -= SHORT_Y[i];
        p.jump_tick += 1;
        if p.jump_tick >= 9 {
            p.jump = 0;
        }
    }
    if p.jump == 3 {
        let i = p.jump_tick as usize;
        p.sprite = VERTICAL_SPRITE[i];
        p.y -= VERTICAL_Y[i];
        p.jump_tick += 1;
        if p.jump_tick >= 9 {
            p.jump = 0;
        }
    }
    None
}
