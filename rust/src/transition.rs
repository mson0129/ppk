//! Palette transitions run at VGA retrace rate, independent of simulation ticks.
use crate::graphics::{Graphics, PIXELS};
const RETRACE_DENOMINATOR: u64 = 1193182 * 800 * 449;
const RETRACES_PER_SIMULATION_TICK: u64 = 65536 * 25175000;
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum FadeKind {
    None = 0,
    Title = 1,
    Clear = 2,
    Stage = 3,
}
pub struct Transition {
    pub kind: FadeKind,
    pub clock: u64,
    pub outgoing: Box<[u8]>,
    pub colors: [u32; 16],
}
impl Transition {
    pub fn new() -> Self {
        Self {
            kind: FadeKind::None,
            clock: 0,
            outgoing: vec![0; PIXELS].into_boxed_slice(),
            colors: [0; 16],
        }
    }
    pub fn begin(&mut self, kind: FadeKind) {
        self.kind = kind;
        self.clock = 0;
    }
    pub fn frames(&self, fraction: i32) -> u32 {
        ((self.clock + RETRACES_PER_SIMULATION_TICK * fraction.clamp(0, 1000) as u64 / 1000)
            / RETRACE_DENOMINATOR) as u32
    }
    pub fn advance(&mut self) -> bool {
        self.clock += RETRACES_PER_SIMULATION_TICK;
        self.frames(0) >= if self.kind == FadeKind::Clear { 64 } else { 65 }
    }
    pub fn capture(&mut self, g: &Graphics) {
        self.outgoing.copy_from_slice(&g.indexed);
        self.colors = g.colors;
    }
    pub fn present(&self, g: &mut Graphics, fraction: i32) {
        let frame = self.frames(fraction);
        let outgoing = self.kind == FadeKind::Clear;
        if outgoing && frame >= 64 {
            g.present();
            return;
        }
        let amount = if outgoing {
            frame
        } else {
            frame.saturating_sub(1)
        }
        .min(64);
        let source = if outgoing { self.colors } else { g.colors };
        let mut palette = [0u32; 16];
        for (i, &color) in source.iter().enumerate() {
            let mut result = 0xff000000;
            for channel in 0..3 {
                let value = (((color >> (channel * 8)) & 255) * 63 + 127) / 255;
                let value = if outgoing {
                    value.saturating_sub(amount)
                } else {
                    value.min(amount)
                };
                result |= (value * 255 / 63) << (channel * 8);
            }
            palette[i] = result;
        }
        let pixels = if outgoing { &self.outgoing } else { &g.indexed };
        for (i, c) in pixels.iter().enumerate() {
            g.rgba[i] = palette[(c & 15) as usize];
        }
    }
}
