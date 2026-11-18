//! Stage scenery, HUD and sprite composition. No gameplay advances here.
use crate::{assets, compat::DATA, engine::Engine};
#[derive(Clone, Copy, Default)]
pub struct Actor {
    pub x: i16,
    pub y: i16,
    pub sprite: i16,
    pub visible: bool,
}
impl Engine {
    pub fn text(&mut self, x: i16, y: i16, color: u8, bytes: &[u8]) {
        self.graphics.text(&mut self.memory, x, y, color, bytes);
    }
    pub fn label(&mut self, x: i16, y: i16, color: u8, offset: usize) {
        let bytes = self.cstring(offset);
        self.text(x, y, color, &bytes);
    }
    pub fn cstring(&self, offset: usize) -> Vec<u8> {
        self.memory.0[DATA + offset..]
            .iter()
            .copied()
            .take(128)
            .take_while(|v| *v != 0)
            .collect()
    }
    pub fn scratch(&mut self, bytes: &[u8]) {
        let n = bytes.len().min(95);
        self.memory.0[DATA + 0xc44..DATA + 0xc44 + n].copy_from_slice(&bytes[..n]);
        self.memory.0[DATA + 0xc44 + n] = 0;
    }
    pub fn scenery_tile(&mut self, x: i16, y: i16, index: usize) {
        self.graphics
            .tile(&mut self.memory, x, y, assets::SCENERY + index * 128);
    }
    pub fn clear_playfield(&mut self) {
        for row in 0..24 {
            for col in 1..40 {
                self.scenery_tile(col * 8 - 1, row * 8, 6);
            }
        }
        for col in 1..40 {
            self.scenery_tile(col * 8 - 1, 1, 6);
        }
    }
    pub fn draw_stage(&mut self) {
        self.game.rebuild_collision_grid();
        self.clear_playfield();
        for row in 0..11 {
            let y = row * 16 + 24;
            self.scenery_tile(0, y, 12 + (row % 2) as usize);
            self.scenery_tile(232, y, 12 + (row % 2) as usize);
            for col in 0..28 {
                let ch = self.game.map[row as usize * 30 + col];
                if ch != b' ' {
                    self.scenery_tile(col as i16 * 8 + 16, y, ch.wrapping_sub(b'a') as usize);
                }
            }
        }
        for col in 0..=1 {
            for row in 0..=5 {
                self.scenery_tile(col * 16 + 272, row * 16 + 24, 18 + (col * 6 + row) as usize);
            }
        }
        for (x, y, index) in [
            (288, 184, 30),
            (240, 168, 6),
            (240, 160, 6),
            (256, 168, 13),
            (256, 152, 12),
            (256, 184, 12),
            (232, 184, 5),
            (248, 152, 5),
            (248, 184, 5),
            (264, 116, 14),
        ] {
            self.scenery_tile(x, y, index);
        }
        self.label(288, 120, 16, 0x794);
        let n = format!("{:2}", self.game.remaining_candy);
        self.scratch(n.as_bytes());
        self.label(298, 120, 9, 0xc44);
        self.graphics
            .tile(&mut self.memory, 264, 132, assets::SPRITE_COLORS + 0xe00);
        self.label(288, 136, 16, 0x794);
        let n = format!("{:2}", self.game.lives);
        self.scratch(n.as_bytes());
        self.label(298, 136, 3, 0xc44);
        self.game.score_dirty = 0;
        self.game.candy_chain = 0;
        self.label(160, 0, 3, 0x796);
        let n = format!("{:07}0", self.game.high_score);
        self.scratch(n.as_bytes());
        self.label(240, 0, 9, 0xc44);
        self.label(0, 0, 16, 0x7a8);
        let n = format!("{:07}0", self.game.score);
        self.scratch(n.as_bytes());
        self.label(64, 0, 2, 0xc44);
        let n = format!("STG:{:2}", self.game.stage);
        self.scratch(n.as_bytes());
        self.label(272, 162, 2, 0xc44);
        let n = format!("CRD:{:2}", self.game.credits);
        self.scratch(n.as_bytes());
        self.label(272, 170, 2, 0xc44);
        self.label(64, 192, 7, 0x7c1);
        self.game.time_bar_units = 0;
        self.game.time_bar_tick = 0;
        self.graphics.background(&self.memory);
    }
    pub fn actor_list(&self) -> Vec<Actor> {
        let mut actors = Vec::with_capacity(64);
        for e in self
            .game
            .walkers
            .iter()
            .take(self.game.walker_count.max(0) as usize)
        {
            actors.push(Actor {
                x: e.x,
                y: e.y,
                sprite: e.sprite,
                visible: true,
            });
        }
        for e in self
            .game
            .fast_walkers
            .iter()
            .take(self.game.fast_count.max(0) as usize)
        {
            actors.push(Actor {
                x: e.x,
                y: e.y,
                sprite: e.sprite,
                visible: true,
            });
        }
        for e in self
            .game
            .hatchlings
            .iter()
            .take(self.game.hatchling_count.max(0) as usize)
        {
            actors.push(Actor {
                x: e.x,
                y: e.y,
                sprite: e.sprite,
                visible: e.active != 0 && e.age % 2 != 0,
            });
        }
        let p = &self.game.player;
        actors.push(Actor {
            x: p.x,
            y: p.y,
            sprite: p.sprite,
            visible: true,
        });
        for e in &self.game.floating_scores {
            actors.push(Actor {
                x: e.x,
                y: e.y,
                sprite: e.sprite,
                visible: e.active != 0,
            });
        }
        actors
    }
    pub fn compose_actors(&mut self, fraction: Option<i32>) {
        self.graphics.background(&self.memory);
        let actors = self.actor_list();
        for (i, a) in actors.iter().enumerate() {
            if !a.visible {
                continue;
            }
            let (mut x, mut y) = (a.x, a.y);
            if let Some(f) = fraction {
                if let Some(old) = self.previous.get(i) {
                    let dx = x - old.x;
                    let dy = y - old.y;
                    if (-8..=8).contains(&dx) && (-8..=8).contains(&dy) {
                        x = old.x
                            + ((dx as i32 * f + if dx >= 0 { 500 } else { -500 }) / 1000) as i16;
                        y = old.y
                            + ((dy as i32 * f + if dy >= 0 { 500 } else { -500 }) / 1000) as i16;
                    }
                }
            }
            let offset = (a.sprite as u16).wrapping_mul(128) as usize;
            self.graphics
                .sprite(&self.memory, x, y, assets::SPRITE_MASKS + offset, true);
            self.graphics
                .sprite(&self.memory, x, y, assets::SPRITE_COLORS + offset, false);
        }
        if fraction.is_none() {
            self.game.presentation_tick = self.game.presentation_tick.wrapping_add(1);
            self.game.player.display_page = if self.game.player.display_page == 1 {
                0
            } else {
                1
            };
        }
    }
    pub fn refresh_score(&mut self) {
        if self.game.score_dirty == 0 {
            return;
        }
        if self.game.music_playing == 0 {
            self.audio.play(
                &mut self.memory,
                &mut self.game,
                crate::audio::Tune::CandyCollected,
            );
        }
        let n = format!("{:07}", self.game.score);
        self.scratch(n.as_bytes());
        self.label(64, 0, 2, 0xc44);
        self.game.score_dirty = 0;
    }
    pub fn advance_level_timer(&mut self) {
        if self.game.player.falling != 0 {
            return;
        }
        self.game.time_bar_tick += 1;
        if self.game.time_bar_tick == 30 {
            self.game.time_bar_units += 1;
            self.game.time_bar_tick = 0;
            if self.game.time_bar_units == 135 {
                self.game.player.falling = 1;
            }
        }
        let ch = b'7'.wrapping_sub((self.game.time_bar_units % 8) as u8);
        let ch = if ch == b'0' { b' ' } else { ch };
        self.memory.0[DATA + 0xc44] = ch;
        self.memory.0[DATA + 0xc45] = 0;
        self.label(self.game.time_bar_units + 64, 192, 5, 0xc44);
    }
}
