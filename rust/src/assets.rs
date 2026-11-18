//! Original data only; no executable code is embedded in the Rust module.
pub const INITIAL: &[u8] = include_bytes!("../../assets/initial-data.bin");
pub const SPRITES: &[u8] = include_bytes!("../../assets/PPK.B16");
pub const MAPS: &[u8] = include_bytes!("../../assets/STAGE.PPK");
pub const TITLE: &[u8] = include_bytes!("../../assets/STAGE.PP1");
pub const CONTINUE: &[u8] = include_bytes!("../../assets/STAGE.PP2");
pub const PALETTE: &[u8] = include_bytes!("../../assets/STAGE.PP3");
pub const DEATH_SOUND: &[u8] = include_bytes!("../../assets/STAGE.PP4");
pub const STAGES: usize = MAPS.len() / 330;
pub const SPRITE_COLORS: usize = 0x6020;
pub const SPRITE_MASKS: usize = 0x7920;
pub const SCENERY: usize = 0x9220;
pub const VARIANTS: usize = 0xa220;
