//! Litematica core mod implementation and state management.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod commands;
pub mod materials;
pub mod placement;
pub mod render;
pub mod schematic;
pub mod selection;
pub mod ui;
pub mod verifier;

use crate::litematica::placement::SchematicPlacement;
use crate::litematica::render::SchematicRenderer;
use crate::litematica::schematic::LitematicaSchematic;
use crate::litematica::selection::AreaSelection;
use crate::litematica::ui::LitematicaMainMenu;
use crate::malilib::config::{ConfigManager, ConfigOption};
use crate::malilib::gui::ScreenManager;
use crate::malilib::input::{HotkeyManager, Keybind, KEY_M, KEY_PAGE_DOWN, KEY_PAGE_UP};
use crate::malilib::render::RenderBatch;

pub struct LitematicaState {
    pub loaded_schematics: HashMap<String, Arc<LitematicaSchematic>>,
    pub placements: Vec<SchematicPlacement>,
    pub selection: AreaSelection,
    pub renderer: SchematicRenderer,
    pub config: ConfigManager,
    pub hotkeys: HotkeyManager,
    pub screens: ScreenManager,
}

impl LitematicaState {
    pub fn new() -> Self {
        let mut config = ConfigManager::new();
        // Register default MaLiLib and Litematica configurations
        config.add_option(
            "Generic",
            ConfigOption::new_bool("enableRendering", true, "Enable schematic overlay rendering"),
        );
        config.add_option(
            "Generic",
            ConfigOption::new_bool("easyPlaceMode", false, "Automatically place blocks matching schematic"),
        );
        config.add_option(
            "Visuals",
            ConfigOption::new_bool("renderGhostBlocks", true, "Render see-through ghost blocks"),
        );
        config.add_option(
            "Visuals",
            ConfigOption::new_bool("renderBoundingBoxes", true, "Render wireframe bounding boxes"),
        );
        config.add_option(
            "Colors",
            ConfigOption::new_color("colorMissingBlock", 0.2, 0.6, 1.0, 0.35, "Color for missing blocks"),
        );
        config.add_option(
            "Colors",
            ConfigOption::new_color("colorWrongBlock", 1.0, 0.1, 0.1, 0.5, "Color for wrong blocks"),
        );
        config.add_option(
            "Hotkeys",
            ConfigOption::new_hotkey("openGui", KEY_M, 0, "Open Litematica Main Menu"),
        );
        config.add_option(
            "Hotkeys",
            ConfigOption::new_hotkey("layerUp", KEY_PAGE_UP, 0, "Move visible layer up"),
        );
        config.add_option(
            "Hotkeys",
            ConfigOption::new_hotkey("layerDown", KEY_PAGE_DOWN, 0, "Move visible layer down"),
        );

        let mut hotkeys = HotkeyManager::new();
        hotkeys.register("openGui", Keybind::new(KEY_M, 0), || {
            // Hotkey handler for 'M' key
            true
        });
        hotkeys.register("layerUp", Keybind::new(KEY_PAGE_UP, 0), || {
            true
        });
        hotkeys.register("layerDown", Keybind::new(KEY_PAGE_DOWN, 0), || {
            true
        });

        Self {
            loaded_schematics: HashMap::new(),
            placements: Vec::new(),
            selection: AreaSelection::new("default"),
            renderer: SchematicRenderer::new(),
            config,
            hotkeys,
            screens: ScreenManager::new(),
        }
    }

    pub fn load_schematic(&mut self, schematic: LitematicaSchematic) -> Arc<LitematicaSchematic> {
        let name = schematic.metadata.name.clone();
        let arc = Arc::new(schematic);
        self.loaded_schematics.insert(name, arc.clone());
        arc
    }

    pub fn add_placement(&mut self, placement: SchematicPlacement) {
        self.placements.push(placement);
    }
}

// Global thread-safe state container
static LITEMATICA_INSTANCE: Mutex<Option<LitematicaState>> = Mutex::new(None);

pub fn init() {
    let mut lock = LITEMATICA_INSTANCE.lock().unwrap();
    if lock.is_none() {
        *lock = Some(LitematicaState::new());
    }
}

pub fn render_world(_view: &[f32; 16], _proj: &[f32; 16], _camera: [f64; 3]) {
    let mut lock = LITEMATICA_INSTANCE.lock().unwrap();
    if let Some(ref mut state) = *lock {
        let mut batch = RenderBatch::new();
        for placement in &state.placements {
            if placement.enabled {
                state.renderer.build_render_batch(placement, &mut batch);
            }
        }
        // batch lines and faces are sent to Horizon level renderer
    }
}

pub fn on_key_press(key: i32, _scan: i32, action: i32, mods: i32) -> bool {
    let mut lock = LITEMATICA_INSTANCE.lock().unwrap();
    if let Some(ref mut state) = *lock {
        if action == 1 {
            if key == KEY_M && mods == 0 {
                if !state.screens.has_active_screen() {
                    state.screens.open_screen(Box::new(LitematicaMainMenu::new()), 800, 600);
                    return true;
                }
            } else if key == KEY_PAGE_UP {
                state.renderer.page_up();
                return true;
            } else if key == KEY_PAGE_DOWN {
                state.renderer.page_down();
                return true;
            }
        }

        if let Some(screen) = state.screens.current_screen_mut() {
            if screen.on_key(key, _scan, action, mods) {
                if key == crate::malilib::input::KEY_ESCAPE {
                    state.screens.close_current();
                }
                return true;
            }
        }
        return state.hotkeys.handle_key_press(key, action, mods);
    }
    false
}

pub fn render_hud(_width: u32, _height: u32, _delta: f32) {
    // InfoHUD rendering: placement name, current layer, selection coords
}
