//! The original 30×11 character maps expand into an 8-pixel collision grid.
use crate::{
    assets,
    state::{GameState, Walker},
};
impl GameState {
    pub fn load_map(&mut self) {
        let start = (self.stage as usize - 1) * 330;
        self.map.copy_from_slice(&assets::MAPS[start..start + 330]);
    }
    pub fn rebuild_collision_grid(&mut self) {
        self.tiles.fill(0);
        self.fast_count = 0;
        self.hatchling_count = 0;
        self.walker_count = 0;
        self.remaining_candy = 0;
        for row in 0..11 {
            for column in 0..28 {
                let code = self.map[row * 30 + column];
                let cell = row * 80 + column + 1;
                match code {
                    b'a' => {
                        self.tiles[cell] = 1;
                        self.tiles[cell + 1] = 0;
                    }
                    b'b' => {
                        self.tiles[cell] = 0;
                        self.tiles[cell + 1] = 1;
                    }
                    b'c'..=b'f' => {
                        self.tiles[cell] = 1;
                        self.tiles[cell + 1] = 1;
                    }
                    b'g' => {
                        self.tiles[cell] = 0;
                        self.tiles[cell + 1] = 0;
                        self.tiles[cell + 40] = 0;
                        self.tiles[cell + 41] = 0;
                    }
                    b'h' => self.tiles[cell + 40] = 3,
                    b'i' => self.tiles[cell + 41] = 3,
                    b'j' => {
                        self.tiles[cell] = 2;
                        self.tiles[cell + 40] = 2;
                        self.tiles[cell + 1] = 1;
                    }
                    b'k' => {
                        self.tiles[cell] = 1;
                        self.tiles[cell + 1] = 2;
                        self.tiles[cell + 41] = 2;
                    }
                    b'l' => {
                        self.tiles[cell + 40] = 7;
                        self.tiles[cell + 41] = 10;
                    }
                    b'm' | b'n' => {
                        self.tiles[cell + 1] = 1;
                        self.tiles[cell + 41] = 1;
                    }
                    b'o' => {
                        self.tiles[cell + 40] = 5;
                        self.tiles[cell + 41] = 8;
                        self.remaining_candy += 1;
                    }
                    b'p' => {
                        self.tiles[cell] = 2;
                        self.tiles[cell + 40] = 2;
                    }
                    b'q' => {
                        self.tiles[cell + 1] = 2;
                        self.tiles[cell + 41] = 2;
                    }
                    b'r' => {
                        self.tiles[cell + 40] = 6;
                        self.tiles[cell + 41] = 9;
                    }
                    b's'..=b'v' => {
                        self.map[row * 30 + column] = b' ';
                        let enemy = Walker {
                            x: column as i16 * 8 + 16,
                            y: row as i16 * 16 + 24,
                            right: if code == b't' || code == b'v' { 1 } else { 0 },
                            sprite: 24,
                        };
                        if code <= b't' {
                            self.walkers[self.walker_count as usize] = enemy;
                            self.walker_count += 1;
                        } else {
                            self.fast_walkers[self.fast_count as usize] = enemy;
                            self.fast_count += 1;
                        }
                    }
                    _ => {}
                }
                // The original uses a wrapping pool, rather than retaining a full count of ten.
                if self.fast_count >= 10 {
                    self.fast_count = 0;
                }
                if self.walker_count >= 10 {
                    self.walker_count = 0;
                }
            }
        }
        for row in 0..21 {
            self.tiles[row * 40] = 1;
        }
        for row in 0..17 {
            self.tiles[row * 40 + 29] = 1;
        }
        for row in 17..=20 {
            self.tiles[row * 40 + 32] = 1;
        }
        self.tiles[800..840].fill(1);
    }
}
