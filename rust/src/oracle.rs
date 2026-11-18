//! Development-only adapter to compare named Rust routines with original addresses.
//! This module is excluded from the shipped WASM.
use super::with;
#[no_mangle]
pub extern "C" fn test_routine(address: u32) {
    with(|e| {
        e.restore();
        match address {
            0x04db => e.game.rebuild_collision_grid(),
            0x0d98 => {
                if let Some(tune) = e.game.move_player(e.input()) {
                    e.audio.play(&mut e.memory, &mut e.game, tune);
                }
            }
            0x11c1 => e.game.move_walkers(false),
            0x12f5 => e.game.move_walkers(true),
            0x1429 => e.game.move_hatchlings(),
            0x15a3 => e.collect_items(),
            0x1cca => e.game.move_floating_scores(),
            0x1db1 => e.refresh_score(),
            0x1e09 => e.advance_level_timer(),
            0x1f34 => e.draw_stage(),
            0x260d => e.game.reset_player_animation(),
            _ => e.error = address,
        }
        e.flush();
    });
}
