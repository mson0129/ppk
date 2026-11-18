//! Legacy address mapping for C-oracle tests and the existing browser ABI.
//! Gameplay never indexes these addresses; named structs are decoded at API boundaries.
use crate::state::*;
pub const DATA: usize = 0xf0a0;
pub const BACKGROUND: usize = 0x20000;
pub const MEMORY_SIZE: usize = 0x50000;
pub struct Memory(pub Box<[u8]>);
impl Memory {
    pub fn new() -> Self {
        Self(vec![0; MEMORY_SIZE].into_boxed_slice())
    }
    pub fn byte(&self, offset: usize) -> u8 {
        self.0.get(offset).copied().unwrap_or(0)
    }
    pub fn word(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.byte(offset), self.byte(offset + 1)])
    }
    pub fn set_word(&mut self, offset: usize, value: u16) {
        self.0[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    pub fn data_word(&self, offset: usize) -> i16 {
        self.word(DATA + offset) as i16
    }
    pub fn data_long(&self, offset: usize) -> u32 {
        self.word(DATA + offset) as u32 | ((self.word(DATA + offset + 2) as u32) << 16)
    }
    pub fn store(&mut self, offset: usize, value: i16) {
        self.set_word(DATA + offset, value as u16);
    }
    pub fn store_long(&mut self, offset: usize, value: u32) {
        self.set_word(DATA + offset, value as u16);
        self.set_word(DATA + offset + 2, (value >> 16) as u16);
    }
    pub fn decode(&self) -> GameState {
        let mut g = GameState::default();
        let p = &mut g.player;
        p.x = self.data_word(0x561);
        p.y = self.data_word(0x563);
        p.display_page = self.data_word(0x565);
        p.sprite = self.data_word(0x567);
        p.walk_step = self.data_word(0x569);
        p.climb_step = self.data_word(0x56b);
        p.jump = self.data_word(0x57d);
        p.jump_tick = self.data_word(0x57f);
        p.climbing = self.data_word(0x581);
        p.falling = self.data_word(0x583);
        p.respawn = self.data_word(0x585);
        p.death_animation_tick = self.data_word(0x587);
        p.reserved = self.data_word(0x589);
        p.facing = self.data_word(0x58b);
        g.tiles
            .copy_from_slice(&self.0[DATA + 0xcec..DATA + 0xcec + 960]);
        g.map
            .copy_from_slice(&self.0[DATA + 0x1244..DATA + 0x1244 + 330]);
        for (i, e) in g.walkers.iter_mut().enumerate() {
            *e = self.walker(0x147e + i * 8);
        }
        for (i, e) in g.fast_walkers.iter_mut().enumerate() {
            *e = self.walker(0x15a0 + i * 8);
        }
        for (i, e) in g.hatchlings.iter_mut().enumerate() {
            let a = 0x14fe + i * 14;
            *e = Hatchling {
                x: self.data_word(a),
                y: self.data_word(a + 2),
                right: self.data_word(a + 4),
                sprite: self.data_word(a + 6),
                active: self.data_word(a + 8),
                age: self.data_word(a + 10),
                reserved: self.data_word(a + 12),
            };
        }
        for (i, e) in g.floating_scores.iter_mut().enumerate() {
            let a = 0x13a0 + i * 8;
            *e = FloatingScore {
                x: self.data_word(a),
                y: self.data_word(a + 2),
                sprite: self.data_word(a + 4),
                active: self.data_word(a + 6),
            };
        }
        g.walker_count = self.data_word(0x13f0);
        g.fast_count = self.data_word(0x1592);
        g.hatchling_count = self.data_word(0x13fc);
        g.remaining_candy = self.data_word(0x13fa);
        g.stage = self.data_word(0x13f4);
        g.lives = self.data_word(0x1398);
        g.credits = self.data_word(0x159a);
        g.time_bar_units = self.data_word(0x1396);
        g.time_bar_tick = self.data_word(0x139c);
        g.death_pitch = self.data_word(0x139a);
        g.score_dirty = self.data_word(0xcea);
        g.candy_chain = self.data_word(0x1590);
        g.floating_cursor = self.data_word(0x10ac);
        g.music_playing = self.data_word(0x158a);
        g.music_count = self.data_word(0x1594);
        g.music_cursor = self.data_word(0x1598);
        g.exit_requested = self.data_word(0xcb4);
        g.presentation_tick = self.data_word(0x1596);
        g.score = self.data_long(0xcb6);
        g.high_score = self.data_long(0x158c);
        g
    }
    fn walker(&self, a: usize) -> Walker {
        Walker {
            x: self.data_word(a),
            y: self.data_word(a + 2),
            right: self.data_word(a + 4),
            sprite: self.data_word(a + 6),
        }
    }
    pub fn encode(&mut self, g: &GameState) {
        self.store(0x561, g.player.x);
        self.store(0x563, g.player.y);
        self.store(0x565, g.player.display_page);
        self.store(0x567, g.player.sprite);
        self.store(0x569, g.player.walk_step);
        self.store(0x56b, g.player.climb_step);
        self.store(0x57d, g.player.jump);
        self.store(0x57f, g.player.jump_tick);
        self.store(0x581, g.player.climbing);
        self.store(0x583, g.player.falling);
        self.store(0x585, g.player.respawn);
        self.store(0x587, g.player.death_animation_tick);
        self.store(0x589, g.player.reserved);
        self.store(0x58b, g.player.facing);
        self.store(0x13f0, g.walker_count);
        self.store(0x1592, g.fast_count);
        self.store(0x13fc, g.hatchling_count);
        self.store(0x13fa, g.remaining_candy);
        self.store(0x13f4, g.stage);
        self.store(0x1398, g.lives);
        self.store(0x159a, g.credits);
        self.store(0x1396, g.time_bar_units);
        self.store(0x139c, g.time_bar_tick);
        self.store(0x139a, g.death_pitch);
        self.store(0xcea, g.score_dirty);
        self.store(0x1590, g.candy_chain);
        self.store(0x10ac, g.floating_cursor);
        self.store(0x158a, g.music_playing);
        self.store(0x1594, g.music_count);
        self.store(0x1598, g.music_cursor);
        self.store(0xcb4, g.exit_requested);
        self.store(0x1596, g.presentation_tick);
        self.store_long(0xcb6, g.score);
        self.store_long(0x158c, g.high_score);
        self.0[DATA + 0xcec..DATA + 0xcec + 960].copy_from_slice(&g.tiles);
        self.0[DATA + 0x1244..DATA + 0x1244 + 330].copy_from_slice(&g.map);
        for (i, e) in g.walkers.iter().enumerate() {
            self.store_walker(0x147e + i * 8, *e);
        }
        for (i, e) in g.fast_walkers.iter().enumerate() {
            self.store_walker(0x15a0 + i * 8, *e);
        }
        for (i, e) in g.hatchlings.iter().enumerate() {
            let a = 0x14fe + i * 14;
            for (j, v) in [e.x, e.y, e.right, e.sprite, e.active, e.age, e.reserved]
                .iter()
                .enumerate()
            {
                self.store(a + j * 2, *v);
            }
        }
        for (i, e) in g.floating_scores.iter().enumerate() {
            let a = 0x13a0 + i * 8;
            for (j, v) in [e.x, e.y, e.sprite, e.active].iter().enumerate() {
                self.store(a + j * 2, *v);
            }
        }
    }
    fn store_walker(&mut self, a: usize, e: Walker) {
        for (j, v) in [e.x, e.y, e.right, e.sprite].iter().enumerate() {
            self.store(a + j * 2, *v);
        }
    }
}
