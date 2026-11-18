//! Software rendering of planar asset data, with original 4-bit palette indices.
use crate::compat::{Memory, BACKGROUND, DATA};
pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 200;
pub const PIXELS: usize = WIDTH * HEIGHT;
pub struct Graphics {
    pub indexed: Box<[u8]>,
    pub rgba: Box<[u32]>,
    pub colors: [u32; 16],
}
impl Graphics {
    pub fn new() -> Self {
        Self {
            indexed: vec![0; PIXELS].into_boxed_slice(),
            rgba: vec![0; PIXELS].into_boxed_slice(),
            colors: [0; 16],
        }
    }
    fn source_pixel(m: &Memory, source: usize, x: usize, y: usize, height: usize) -> u8 {
        let mut color = 0;
        for plane in 0..4 {
            color |= ((m.byte(source + plane * 2 * height + y * 2 + x / 8) >> (7 - (x % 8))) & 1)
                << plane;
        }
        color
    }
    pub fn tile(&mut self, m: &mut Memory, x: i16, y: i16, source: usize) {
        let x = (x >> 3) << 3;
        for row in 0..16 {
            for col in 0..16 {
                let xx = x + col;
                let yy = y + row;
                if !(0..320).contains(&xx) || !(0..200).contains(&yy) {
                    continue;
                }
                let c = Self::source_pixel(m, source, col as usize, row as usize, 16);
                for plane in 0..4 {
                    let a = BACKGROUND + plane * 8000 + yy as usize * 40 + xx as usize / 8;
                    let mask = 0x80 >> (xx % 8);
                    m.0[a] = (m.0[a] & !mask) | if c & (1 << plane) != 0 { mask } else { 0 };
                }
            }
        }
    }
    pub fn background(&mut self, m: &Memory) {
        for y in 0..200 {
            for x in 0..320 {
                let mut c = 0;
                for p in 0..4 {
                    c |= ((m.byte(BACKGROUND + p * 8000 + y * 40 + x / 8) >> (7 - (x % 8))) & 1)
                        << p;
                }
                self.indexed[y * 320 + x] = c;
            }
        }
    }
    pub fn sprite(&mut self, m: &Memory, x: i16, y: i16, source: usize, and_mask: bool) {
        for row in 0..16 {
            for col in 0..16 {
                let xx = x + col;
                let yy = y + row;
                if !(0..320).contains(&xx) || !(0..200).contains(&yy) {
                    continue;
                }
                let c = Self::source_pixel(m, source, col as usize, row as usize, 16);
                let dst = &mut self.indexed[yy as usize * 320 + xx as usize];
                if and_mask {
                    *dst &= c
                } else {
                    *dst ^= c
                };
            }
        }
    }
    pub fn text(&mut self, m: &mut Memory, x: i16, y: i16, color: u8, text: &[u8]) {
        for (k, &ch) in text.iter().take(128).enumerate() {
            if ch == 0 {
                break;
            }
            for row in 0..8 {
                for col in 0..8 {
                    let bit = (m.byte(DATA - 110 + ch as usize * 8 + row) >> (7 - col)) & 1;
                    let xx = ((x as i32 + k as i32 * 8) >> 3) * 8 + col as i32;
                    let yy = y as i32 + row as i32;
                    if !(0..320).contains(&xx) || !(0..200).contains(&yy) {
                        continue;
                    }
                    let c = if bit != 0 { 15 ^ color } else { 0 };
                    for p in 0..4 {
                        let a = BACKGROUND + p * 8000 + yy as usize * 40 + xx as usize / 8;
                        let mask = 0x80 >> (xx % 8);
                        m.0[a] = (m.0[a] & !mask) | if c & (1 << p) != 0 { mask } else { 0 };
                    }
                }
            }
        }
    }
    pub fn present(&mut self) {
        for (i, c) in self.indexed.iter().enumerate() {
            self.rgba[i] = self.colors[(*c & 15) as usize];
        }
    }
    pub fn load_picture(&mut self, m: &mut Memory, file: &[u8]) -> Result<(), u32> {
        let mut pos = 128;
        let mut written = 0;
        while written < 32000 && pos < file.len() {
            let mut value = file[pos];
            pos += 1;
            let mut count = 1;
            if value & 0xc0 == 0xc0 {
                count = (value & 63) as usize;
                if pos >= file.len() {
                    return Err(0x2477);
                }
                value = file[pos];
                pos += 1;
            }
            for _ in 0..count {
                if written >= 32000 {
                    break;
                }
                let row = written / 160;
                let plane = (written / 40 % 4 + 3) % 4;
                let col = written % 40;
                m.0[BACKGROUND + plane * 8000 + row * 40 + col] = value;
                written += 1;
            }
        }
        if written != 32000 {
            return Err(0x2477);
        }
        for i in 0..16 {
            let index = m.byte(DATA + 0x56d + i) as usize;
            let rgb = &file[16 + index * 3..19 + index * 3];
            self.colors[i] = rgba(rgb[0] >> 2, rgb[1] >> 2, rgb[2] >> 2);
        }
        Ok(())
    }
}
pub fn rgba(r: u8, g: u8, b: u8) -> u32 {
    0xff000000
        | ((b as u32 * 255 / 63) << 16)
        | ((g as u32 * 255 / 63) << 8)
        | (r as u32 * 255 / 63)
}
