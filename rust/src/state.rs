//! Gameplay data. Units are original 320×200 pixels and fixed simulation ticks.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Player {
    pub x: i16,
    pub y: i16,
    pub display_page: i16,
    pub sprite: i16,
    pub walk_step: i16,
    pub climb_step: i16,
    /// 0: on foot, 1: short jump, 2: running jump, 3: vertical jump.
    pub jump: i16,
    pub jump_tick: i16,
    pub climbing: i16,
    /// Original falling/death animation, also used when the player loses support.
    pub falling: i16,
    pub respawn: i16,
    pub death_animation_tick: i16,
    pub reserved: i16,
    /// 0: left, 1: right, 2: facing the ladder.
    pub facing: i16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Walker {
    pub x: i16,
    pub y: i16,
    pub right: i16,
    pub sprite: i16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Hatchling {
    pub x: i16,
    pub y: i16,
    pub right: i16,
    pub sprite: i16,
    pub active: i16,
    pub age: i16,
    pub reserved: i16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FloatingScore {
    pub x: i16,
    pub y: i16,
    pub sprite: i16,
    pub active: i16,
}
#[derive(Clone, Debug, PartialEq)]
pub struct GameState {
    pub player: Player,
    /// Tile collision grid: 40 columns × 24 rows, 8 pixels per cell.
    pub tiles: [u8; 960],
    pub map: [u8; 330],
    pub walkers: [Walker; 16],
    pub fast_walkers: [Walker; 10],
    pub hatchlings: [Hatchling; 10],
    pub floating_scores: [FloatingScore; 8],
    pub walker_count: i16,
    pub fast_count: i16,
    pub hatchling_count: i16,
    pub remaining_candy: i16,
    pub stage: i16,
    pub lives: i16,
    pub credits: i16,
    /// Original countdown display: one unit per 30 simulation ticks, not wall time.
    pub time_bar_units: i16,
    pub time_bar_tick: i16,
    pub death_pitch: i16,
    pub score: u32,
    pub high_score: u32,
    pub score_dirty: i16,
    pub candy_chain: i16,
    pub floating_cursor: i16,
    pub music_playing: i16,
    pub music_count: i16,
    pub music_cursor: i16,
    pub exit_requested: i16,
    pub presentation_tick: i16,
}
impl Default for GameState {
    fn default() -> Self {
        Self {
            player: Player::default(),
            tiles: [0; 960],
            map: [0; 330],
            walkers: [Walker::default(); 16],
            fast_walkers: [Walker::default(); 10],
            hatchlings: [Hatchling::default(); 10],
            floating_scores: [FloatingScore::default(); 8],
            walker_count: 0,
            fast_count: 0,
            hatchling_count: 0,
            remaining_candy: 0,
            stage: 1,
            lives: 5,
            credits: 5,
            time_bar_units: 0,
            time_bar_tick: 0,
            death_pitch: 1200,
            score: 0,
            high_score: 10000,
            score_dirty: 0,
            candy_chain: 0,
            floating_cursor: 0,
            music_playing: 0,
            music_count: 0,
            music_cursor: 0,
            exit_requested: 0,
            presentation_tick: 0,
        }
    }
}
impl GameState {
    pub fn tile(&self, x: i16, y: i16) -> u8 {
        let index = (y as i32 >> 3) * 40 + (x as i32 >> 3);
        self.tiles.get(index as usize).copied().unwrap_or(0)
    }
    pub fn hit_player(&mut self) {
        self.player.falling = 1;
        self.player.jump = 0;
        self.player.climbing = 0;
    }
    pub fn reset_player_animation(&mut self) {
        let p = &mut self.player;
        p.jump = 0;
        p.jump_tick = 0;
        p.climbing = 0;
        p.falling = 0;
        p.respawn = 0;
        p.death_animation_tick = 0;
        p.reserved = 0;
        p.facing = 2;
        p.sprite = 28;
        self.floating_cursor = 0;
        for effect in &mut self.floating_scores {
            effect.active = 0;
        }
        self.music_playing = 0;
        self.death_pitch = 1200;
        self.exit_requested = 0;
    }
}
