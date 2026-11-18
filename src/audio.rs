//! PC-speaker note sequencing. JavaScript still owns the audio device.
use crate::{
    compat::{Memory, DATA},
    state::GameState,
};
#[derive(Clone, Copy, Debug)]
pub enum Tune {
    StandingJump,
    RunningJump,
    CandyCollected,
    EggHatching,
}
impl Tune {
    fn notes(self) -> (usize, usize) {
        match self {
            Self::StandingJump => (0x506, 3),
            Self::RunningJump => (0x509, 7),
            Self::CandyCollected => (0x4f2, 4),
            Self::EggHatching => (0x4f7, 14),
        }
    }
}
pub struct Audio {
    pub notes: [u8; 128],
    pub count: usize,
    pub cursor: usize,
    pub frequency: u32,
    pub muted: bool,
    pub event: u32,
}
impl Audio {
    pub fn new() -> Self {
        Self {
            notes: [0; 128],
            count: 0,
            cursor: 0,
            frequency: 0,
            muted: false,
            event: 0,
        }
    }
    pub fn play(&mut self, m: &mut Memory, g: &mut GameState, tune: Tune) {
        let (offset, count) = tune.notes();
        self.queue(m, g, offset, count);
    }
    fn queue(&mut self, m: &mut Memory, g: &mut GameState, offset: usize, count: usize) {
        self.count = count.min(128);
        self.cursor = 0;
        for i in 0..self.count {
            self.notes[i] = m.byte(DATA + offset + i);
            m.0[DATA + 0x48e + i] = self.notes[i];
        }
        g.music_cursor = 0;
        g.music_count = self.count as i16;
        g.music_playing = 1;
    }
    pub fn title_song(&mut self, m: &Memory, g: &mut GameState) {
        self.count = 54;
        self.cursor = 0;
        for i in 0..54 {
            self.notes[i] = m.byte(DATA + 0x511 + i);
        }
        g.music_playing = 1;
        g.music_cursor = 0;
        g.music_count = 54;
    }
    pub fn tick(&mut self, m: &Memory, g: &mut GameState) {
        if self.cursor < self.count {
            let note = self.notes[self.cursor];
            self.cursor += 1;
            g.music_cursor = self.cursor as i16;
            let divisor = m.data_word(0x3c6 + note as usize * 2) as u16;
            self.frequency = if divisor != 0 {
                1193182 / divisor as u32
            } else {
                0
            };
            g.music_playing = 1;
        } else {
            self.frequency = 0;
            g.music_playing = 0;
        }
    }
}
