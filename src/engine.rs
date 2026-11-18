//! Fixed-tick game lifecycle. Original C is an external oracle, not a runtime dependency.
use crate::{
    assets,
    audio::Audio,
    compat::{Memory, DATA},
    graphics::{rgba, Graphics},
    items::Pickup,
    player::Input,
    render::Actor,
    state::GameState,
    transition::{FadeKind, Transition},
};
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum Phase {
    Playing = 0,
    Continue = 2,
    Title = 3,
    StageClear = 4,
    Fading = 5,
}
pub struct Engine {
    pub memory: Memory,
    pub game: GameState,
    pub graphics: Graphics,
    pub audio: Audio,
    pub transition: Transition,
    pub phase: Phase,
    pub ticks: u32,
    pub advance_ready: bool,
    pub title_space_release: bool,
    pub finished: bool,
    pub intro_x: i16,
    pub intro_previous_x: i16,
    pub intro_direction: i16,
    pub previous: Vec<Actor>,
    pub error: u32,
}
impl Engine {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
            game: GameState::default(),
            graphics: Graphics::new(),
            audio: Audio::new(),
            transition: Transition::new(),
            phase: Phase::Playing,
            ticks: 0,
            advance_ready: false,
            title_space_release: false,
            finished: false,
            intro_x: 0,
            intro_previous_x: 0,
            intro_direction: 0,
            previous: Vec::with_capacity(64),
            error: 0,
        }
    }
    /// Preserve allocated framebuffer addresses: JavaScript holds typed-array views.
    pub fn reset(&mut self) {
        self.memory.0.fill(0);
        self.graphics.indexed.fill(0);
        self.graphics.rgba.fill(0);
        self.memory.0[DATA..DATA + assets::INITIAL.len()].copy_from_slice(assets::INITIAL);
        self.memory.0[assets::SPRITE_COLORS..assets::SPRITE_COLORS + 48 * 128]
            .copy_from_slice(&assets::SPRITES[..48 * 128]);
        self.memory.0[assets::SPRITE_MASKS..assets::SPRITE_MASKS + 49 * 128]
            .copy_from_slice(&assets::SPRITES[48 * 128..97 * 128]);
        self.memory.0[assets::SCENERY..assets::SCENERY + 31 * 128]
            .copy_from_slice(&assets::SPRITES[97 * 128..128 * 128]);
        self.memory.0[assets::VARIANTS..assets::VARIANTS + 28 * 128]
            .copy_from_slice(&assets::SPRITES[128 * 128..156 * 128]);
        self.memory.0[0xb020..0xb020 + assets::MAPS.len()].copy_from_slice(assets::MAPS);
        self.memory.0[DATA + 0x15f0..DATA + 0x15f0 + assets::DEATH_SOUND.len()]
            .copy_from_slice(assets::DEATH_SOUND);
        self.memory.store(0x159c, 0x2000);
        self.memory.store(0x1212, 1);
        self.game = self.memory.decode();
        self.game.stage = 1;
        self.game.lives = 5;
        self.game.credits = 5;
        self.game.high_score = 10000;
        self.game.score = 0;
        self.game.player.x = 248;
        self.game.player.y = 168;
        self.phase = Phase::Playing;
        self.transition.kind = FadeKind::None;
        self.transition.clock = 0;
        self.advance_ready = false;
        self.title_space_release = false;
        self.ticks = 0;
        self.finished = false;
        self.error = 0;
        self.audio = Audio::new();
        self.load_stage();
        self.flush();
    }
    pub fn restore(&mut self) {
        self.game = self.memory.decode();
    }
    pub fn flush(&mut self) {
        self.memory.encode(&self.game);
    }
    pub fn palette(&mut self) {
        for i in 0..16 {
            let c = self.memory.byte(DATA + 0x56d + i) as usize;
            self.graphics.colors[i] = rgba(
                assets::PALETTE[c * 3],
                assets::PALETTE[c * 3 + 1],
                assets::PALETTE[c * 3 + 2],
            );
        }
    }
    pub fn load_stage(&mut self) {
        if !(1..=assets::STAGES as i16).contains(&self.game.stage) {
            self.error = 0x1eeb;
            return;
        }
        self.palette();
        self.game.load_map();
        let start = assets::VARIANTS + self.game.stage as usize % 27 * 128;
        let variant = self.memory.0[start..start + 128].to_vec();
        self.memory.0[assets::SCENERY + 0x700..assets::SCENERY + 0x780].copy_from_slice(&variant);
        self.game.reset_player_animation();
        self.draw_stage();
        self.compose_actors(None);
        self.capture();
        self.graphics.present();
    }
    pub fn start_stage(&mut self, stage: i32) {
        if !(1..=assets::STAGES as i32).contains(&stage) {
            return;
        }
        self.reset();
        if stage != 1 {
            self.game.stage = stage as i16;
            self.load_stage();
        }
        self.flush();
    }
    pub fn show_intro(&mut self) {
        if let Err(e) = self.graphics.load_picture(&mut self.memory, assets::TITLE) {
            self.error = e;
            return;
        }
        self.intro_x = 186;
        self.intro_previous_x = 186;
        self.intro_direction = 2;
        self.phase = Phase::Title;
        self.ticks = 0;
        self.audio.title_song(&self.memory, &mut self.game);
        self.render_intro(self.intro_x);
        self.graphics.present();
        self.flush();
    }
    fn render_intro(&mut self, x: i16) {
        self.graphics.background(&self.memory);
        self.graphics
            .sprite(&self.memory, x, 119, assets::SPRITE_MASKS + 0x1800, true);
        self.graphics.sprite(
            &self.memory,
            x + 30,
            113,
            assets::SPRITE_MASKS + 0x1800,
            true,
        );
    }
    fn show_continue(&mut self) {
        if let Err(e) = self
            .graphics
            .load_picture(&mut self.memory, assets::CONTINUE)
        {
            self.error = e;
            return;
        }
        let n = format!("CREDIT:{:2}", self.game.credits);
        self.scratch(n.as_bytes());
        self.label(240, 178, 2, 0xc44);
        self.graphics.background(&self.memory);
        self.graphics.present();
        self.phase = Phase::Continue;
        self.advance_ready = false;
    }
    pub fn key(&mut self, scan: i32, mut down: bool) {
        if scan == 57 && self.title_space_release {
            if !down {
                self.title_space_release = false;
            }
            down = false;
        }
        if (0..128).contains(&scan) {
            self.memory.0[DATA + 0x10ae + scan as usize] = u8::from(down);
        }
    }
    pub fn release_keys(&mut self) {
        self.memory.0[DATA + 0x10ae..DATA + 0x10ae + 128].fill(0);
    }
    pub fn input(&self) -> Input {
        let held = |scan| self.memory.byte(DATA + 0x10ae + scan) != 0;
        Input {
            up: held(72),
            down: held(80),
            left: held(75),
            right: held(77),
            jump: held(57),
            enter: held(28),
        }
    }
    fn begin_fade(&mut self, kind: FadeKind) {
        self.transition.begin(kind);
        self.phase = Phase::Fading;
    }
    fn restart_after_death(&mut self) {
        self.game.player.x = 248;
        self.game.player.y = 168;
        self.game.lives = self.game.lives.wrapping_sub(1);
        if self.game.lives < 0 {
            self.show_continue();
        } else {
            self.release_keys();
            self.load_stage();
            self.begin_fade(FadeKind::Stage);
            self.transition.present(&mut self.graphics, 0);
        }
    }
    fn prepare_bonus(&mut self) {
        self.phase = Phase::StageClear;
        self.advance_ready = false;
        self.clear_playfield();
        self.label(80, 100, 5, 0x849);
        let n = ((135 - self.game.time_bar_units as i32) * 20) as u32;
        self.scratch(n.to_string().as_bytes());
        self.label(192, 100, 2, 0xc44);
        self.graphics.background(&self.memory);
        self.graphics.present();
    }
    pub fn collect_items(&mut self) {
        if let Some(collected) = self.game.collect_and_check_hazards() {
            self.scenery_tile(collected.erase_x, collected.erase_y, 6);
            match collected.kind {
                Pickup::Candy => {
                    let n = format!("{:2}", self.game.remaining_candy);
                    self.text(298, 120, 9, n.as_bytes());
                }
                Pickup::Egg => self.audio.play(
                    &mut self.memory,
                    &mut self.game,
                    crate::audio::Tune::EggHatching,
                ),
                Pickup::Bonus => {}
            }
        }
    }
    pub fn capture(&mut self) {
        self.previous = self.actor_list();
    }
    pub fn tick(&mut self) {
        self.restore();
        self.tick_game();
        self.flush();
    }
    fn tick_game(&mut self) {
        if self.error != 0 {
            return;
        }
        self.capture();
        self.ticks = self.ticks.wrapping_add(1);
        self.audio.tick(&self.memory, &mut self.game);
        let input = self.input();
        match self.phase {
            Phase::Fading => {
                if self.transition.advance() {
                    let kind = self.transition.kind;
                    self.transition.kind = FadeKind::None;
                    self.transition.clock = 0;
                    if kind == FadeKind::Clear {
                        self.phase = Phase::StageClear;
                        self.advance_ready = false;
                        self.graphics.present();
                    } else if kind == FadeKind::Death {
                        self.restart_after_death();
                    } else {
                        self.phase = Phase::Playing;
                        self.release_keys();
                        self.capture();
                        self.graphics.present();
                    }
                } else {
                    self.transition.present(&mut self.graphics, 0);
                }
                return;
            }
            Phase::Title => {
                if input.jump {
                    let stage = self.game.stage;
                    self.start_stage(stage as i32);
                    self.title_space_release = true;
                    self.begin_fade(FadeKind::Title);
                    self.transition.present(&mut self.graphics, 0);
                    return;
                }
                self.intro_previous_x = self.intro_x;
                self.intro_x += self.intro_direction;
                if self.intro_x >= 213 || self.intro_x <= 184 {
                    self.intro_direction = -self.intro_direction;
                }
                self.render_intro(self.intro_x);
                self.graphics.present();
                return;
            }
            Phase::StageClear | Phase::Continue => {
                let pressed = input.jump || input.enter;
                if !pressed {
                    self.advance_ready = true;
                }
                if pressed && self.advance_ready {
                    if self.phase == Phase::StageClear {
                        self.game.stage = self.game.stage % assets::STAGES as i16 + 1;
                    } else if self.game.credits > 0 {
                        self.game.credits -= 1;
                        self.game.lives = 5;
                    } else {
                        self.reset();
                        self.show_intro();
                        return;
                    }
                    self.game.player.x = 248;
                    self.game.player.y = 168;
                    self.release_keys();
                    self.load_stage();
                    self.begin_fade(FadeKind::Stage);
                    self.advance_ready = false;
                    self.transition.present(&mut self.graphics, 0);
                }
                return;
            }
            Phase::Playing => {}
        }
        self.game.move_walkers(false);
        self.game.move_walkers(true);
        self.game.move_hatchlings();
        if let Some(tune) = self.game.move_player(input) {
            self.audio.play(&mut self.memory, &mut self.game, tune);
        }
        self.game.move_floating_scores();
        self.refresh_score();
        self.advance_level_timer();
        self.collect_items();
        if self.game.stage == assets::STAGES as i16 && self.game.remaining_candy == 0 {
            self.finished = true;
        }
        self.compose_actors(None);
        if self.game.player.falling != 0 {
            let divisor = if self.game.death_pitch != 0 {
                self.game.death_pitch as u16
            } else {
                1
            };
            self.audio.frequency = 1193182 / divisor as u32;
            self.game.death_pitch = self.game.death_pitch.wrapping_add(10);
        }
        if self.game.player.respawn != 0 {
            self.transition.capture(&self.graphics);
            self.begin_fade(FadeKind::Death);
            self.audio.event = self.audio.event.wrapping_add(1);
            self.transition.present(&mut self.graphics, 0);
            return;
        }
        if self.game.remaining_candy == 0 && self.phase == Phase::Playing {
            self.refresh_score();
            self.advance_ready = false;
            self.transition.capture(&self.graphics);
            self.prepare_bonus();
            self.begin_fade(FadeKind::Clear);
            self.transition.present(&mut self.graphics, 0);
            return;
        }
        self.graphics.present();
    }
    pub fn render(&mut self, fraction: i32) {
        self.restore();
        match self.phase {
            Phase::Fading => self.transition.present(&mut self.graphics, fraction),
            Phase::Title => {
                let x = self.intro_previous_x
                    + ((self.intro_x - self.intro_previous_x) as i32 * fraction / 1000) as i16;
                self.render_intro(x);
                self.graphics.present();
            }
            Phase::Playing => {
                self.compose_actors(Some(fraction.clamp(0, 1000)));
                self.graphics.present();
            }
            _ => self.graphics.present(),
        }
    }
    pub fn replay_hash(&self) -> u32 {
        let mut hash = 2166136261u32;
        for &byte in self.memory.0.iter() {
            hash = (hash ^ byte as u32).wrapping_mul(16777619);
        }
        let extra = [
            self.phase as u32,
            self.ticks,
            self.advance_ready as u32,
            self.title_space_release as u32,
            self.finished as u32,
            self.intro_x as u32,
            self.intro_direction as u32,
            self.transition.kind as u32,
            self.transition.clock as u32,
            (self.transition.clock >> 32) as u32,
        ];
        for value in extra {
            hash = (hash ^ value).wrapping_mul(16777619);
        }
        hash
    }
}
