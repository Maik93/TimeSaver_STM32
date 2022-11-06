//! Some pre-defined custom characters.

pub type CharMap = [u8; 8];

pub struct CustomChar {
    pub id: u8,
    pub char_map: CharMap,
}

// TODO: consider an HashMap instead
// https://doc.rust-lang.org/std/collections/hash_map/struct.HashMap.html
// https://doc.rust-lang.org/std/collections/index.html
pub const MAN_STANDING: u8 = 0;
pub const MAN_DANCING: u8 = 1;
pub const HEART_BORDER: u8 = 2;
pub const HEART_FULL: u8 = 3;

pub const CUSTOM_CHARS_MAPS: [CharMap; 4] = [
    [0x0e, 0x0e, 0x04, 0x1f, 0x04, 0x04, 0x0a, 0x0a],
    [0x0e, 0x0e, 0x15, 0x0e, 0x04, 0x04, 0x0a, 0x11],
    [0x00, 0x00, 0x0a, 0x15, 0x11, 0x0a, 0x04, 0x00], // FIXME: this seems displayed incorrectly
    [0x00, 0x00, 0x0a, 0x1f, 0x1f, 0x0e, 0x04, 0x00],
];
