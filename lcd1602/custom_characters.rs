//! Some pre-defined custom characters.

pub type CharMap = [u8; 8];

pub const _MAN_STANDING: CharMap = [0x0e, 0x0e, 0x04, 0x04, 0x1f, 0x04, 0x0a, 0x0a];
pub const _MAN_DANCING: CharMap = [0x0e, 0x0e, 0x15, 0x0e, 0x04, 0x04, 0x0a, 0x11];

pub const _HEART_BORDER: CharMap = [0x00, 0x00, 0x0a, 0x15, 0x11, 0x0a, 0x04, 0x00];
pub const _HEART_FULL: CharMap = [0x00, 0x00, 0x0a, 0x1f, 0x1f, 0x0e, 0x04, 0x00];
