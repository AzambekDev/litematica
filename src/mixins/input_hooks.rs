//! Keyboard input hooks into Horizon engine
use crate::litematica;

/// Hook called on raw key input before Horizon handles it.
/// Returns true if the key event was consumed by Litematica or MaLiLib.
pub fn on_key_event(key_code: i32, scan_code: i32, action: i32, modifiers: i32) -> bool {
    litematica::on_key_press(key_code, scan_code, action, modifiers)
}
