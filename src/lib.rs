//! Human-maintained Rust engine for 돌아온 너구리.
//! The browser ABI is intentionally unchanged. All gameplay is native Rust.
mod assets;
mod audio;
mod compat;
mod enemies;
mod engine;
mod graphics;
mod items;
mod player;
mod render;
mod stage;
mod state;
mod transition;
use engine::Engine;
use std::cell::RefCell;
thread_local! {static ENGINE:RefCell<Engine>=RefCell::new(Engine::new());}
fn with<T>(f: impl FnOnce(&mut Engine) -> T) -> T {
    ENGINE.with(|e| f(&mut e.borrow_mut()))
}
#[no_mangle]
pub extern "C" fn init() {
    with(|e| e.reset());
}
#[no_mangle]
pub extern "C" fn tick() {
    with(|e| e.tick());
}
#[no_mangle]
pub extern "C" fn show_intro() {
    with(|e| {
        e.restore();
        e.show_intro();
    });
}
#[no_mangle]
pub extern "C" fn start_stage(stage: i32) {
    with(|e| e.start_stage(stage));
}
#[no_mangle]
pub extern "C" fn key(scan: i32, down: i32) {
    with(|e| e.key(scan, down != 0));
}
#[no_mangle]
pub extern "C" fn release_keys() {
    with(|e| e.release_keys());
}
#[no_mangle]
pub extern "C" fn render_interpolated(fraction: i32) {
    with(|e| e.render(fraction));
}
#[no_mangle]
pub extern "C" fn framebuffer() -> u32 {
    with(|e| e.graphics.rgba.as_ptr() as usize as u32)
}
#[no_mangle]
pub extern "C" fn data_pointer() -> u32 {
    with(|e| e.memory.0.as_ptr() as usize as u32)
}
#[no_mangle]
pub extern "C" fn get_error() -> u32 {
    with(|e| e.error)
}
#[no_mangle]
pub extern "C" fn get_phase() -> u32 {
    with(|e| e.phase as u32)
}
#[no_mangle]
pub extern "C" fn get_stage() -> u32 {
    with(|e| e.game.stage as u32)
}
#[no_mangle]
pub extern "C" fn stage_count() -> u32 {
    assets::STAGES as u32
}
#[no_mangle]
pub extern "C" fn debug_state(offset: u32) -> u32 {
    with(|e| e.memory.data_word(offset as usize) as u16 as u32)
}
#[no_mangle]
pub extern "C" fn audio_event() -> u32 {
    with(|e| e.audio.event)
}
#[no_mangle]
pub extern "C" fn audio_frequency() -> u32 {
    with(|e| if e.audio.muted { 0 } else { e.audio.frequency })
}
#[no_mangle]
pub extern "C" fn mute(m: i32) {
    with(|e| e.audio.muted = m != 0);
}
#[no_mangle]
pub extern "C" fn run_finished() -> u32 {
    with(|e| e.finished as u32)
}
#[no_mangle]
pub extern "C" fn frame_count() -> u32 {
    with(|e| e.ticks)
}
#[no_mangle]
pub extern "C" fn replay_hash() -> u32 {
    with(|e| e.replay_hash())
}
#[no_mangle]
pub extern "C" fn transition_frame(fraction: i32) -> u32 {
    with(|e| {
        if e.transition.kind == transition::FadeKind::None {
            0
        } else {
            e.transition.frames(fraction)
        }
    })
}
#[no_mangle]
pub extern "C" fn transition_kind() -> u32 {
    with(|e| e.transition.kind as u32)
}
