//! Input management: keycodes, modifier handling, keybinds, and hotkey callbacks.

use std::collections::HashMap;

// GLFW / Horizon standard keycodes
pub const KEY_M: i32 = 77;
pub const KEY_PAGE_UP: i32 = 266;
pub const KEY_PAGE_DOWN: i32 = 267;
pub const KEY_ESCAPE: i32 = 256;
pub const KEY_ENTER: i32 = 257;
pub const KEY_LEFT_CONTROL: i32 = 341;
pub const KEY_LEFT_SHIFT: i32 = 340;
pub const KEY_LEFT_ALT: i32 = 342;

// Modifier bit flags
pub const MOD_SHIFT: i32 = 1;
pub const MOD_CONTROL: i32 = 2;
pub const MOD_ALT: i32 = 4;
pub const MOD_SUPER: i32 = 8;

// Actions
pub const ACTION_RELEASE: i32 = 0;
pub const ACTION_PRESS: i32 = 1;
pub const ACTION_REPEAT: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Keybind {
    pub key: i32,
    pub modifiers: i32,
}

impl Keybind {
    pub const fn new(key: i32, modifiers: i32) -> Self {
        Self { key, modifiers }
    }

    pub fn matches(&self, key: i32, modifiers: i32) -> bool {
        self.key == key && (self.modifiers == 0 || (self.modifiers & modifiers) == self.modifiers)
    }
}

pub type HotkeyCallback = Box<dyn Fn() -> bool + Send + Sync>;

pub struct HotkeyManager {
    hotkeys: HashMap<String, (Keybind, HotkeyCallback)>,
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            hotkeys: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, name: &str, keybind: Keybind, callback: F)
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        self.hotkeys.insert(name.to_string(), (keybind, Box::new(callback)));
    }

    pub fn handle_key_press(&self, key: i32, action: i32, modifiers: i32) -> bool {
        if action != ACTION_PRESS {
            return false;
        }
        for (_, (keybind, callback)) in &self.hotkeys {
            if keybind.matches(key, modifiers) {
                if callback() {
                    return true;
                }
            }
        }
        false
    }
}
