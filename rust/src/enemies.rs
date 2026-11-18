//! Enemy motion and collision, including the original order of edge checks.
use crate::state::{GameState, Walker};
fn touches(x: i16, y: i16, p: &crate::state::Player) -> bool {
    (p.x.wrapping_sub(x)).wrapping_abs() < 14 && (p.y.wrapping_sub(y)).wrapping_abs() < 13
}
fn move_walker(e: &mut Walker, step: i16) {
    if e.right == 1 {
        if e.x >= 224 {
            e.right = 0;
            e.sprite = 26;
        }
        e.x = e.x.wrapping_add(step);
        e.sprite = e.sprite.wrapping_add(1);
        if e.sprite == 26 {
            e.sprite = 24;
        }
    } else {
        if e.x <= 16 {
            e.right = 1;
            e.sprite = 24;
        }
        e.x = e.x.wrapping_sub(step);
        e.sprite = e.sprite.wrapping_add(1);
        if e.sprite == 28 {
            e.sprite = 26;
        }
    }
}
impl GameState {
    pub fn move_walkers(&mut self, fast: bool) {
        let count = if fast {
            self.fast_count
        } else {
            self.walker_count
        };
        for i in 0..count.max(0) as usize {
            let e = if fast {
                self.fast_walkers[i]
            } else {
                self.walkers[i]
            };
            if touches(e.x, e.y, &self.player) {
                self.hit_player();
            }
            move_walker(
                if fast {
                    &mut self.fast_walkers[i]
                } else {
                    &mut self.walkers[i]
                },
                if fast { 4 } else { 2 },
            );
        }
    }
    pub fn move_hatchlings(&mut self) {
        for i in 0..self.hatchling_count.max(0) as usize {
            let e = &mut self.hatchlings[i];
            if e.active == 0 {
                continue;
            }
            let hit = if e.age == 23 {
                touches(e.x, e.y, &self.player)
            } else {
                e.age = e.age.wrapping_add(1);
                false
            };
            if e.right == 1 {
                e.sprite = e.sprite.wrapping_add(1);
                if e.sprite == 4 {
                    e.sprite = 0;
                }
                e.x = e.x.wrapping_add(1);
                if e.x >= 224 {
                    e.right = 0;
                    e.sprite = 4;
                }
            } else {
                e.sprite = e.sprite.wrapping_add(1);
                if e.sprite == 8 {
                    e.sprite = 4;
                }
                e.x = e.x.wrapping_sub(1);
                if e.x <= 16 {
                    e.right = 1;
                    e.sprite = 0;
                }
            }
            if hit {
                self.hit_player();
            }
        }
    }
    pub fn move_floating_scores(&mut self) {
        for e in &mut self.floating_scores {
            if e.active != 0 {
                e.y = e.y.wrapping_sub(1);
                if e.y % 16 == 1 {
                    e.active = 0;
                }
            }
        }
    }
}
