//! Main in-game user interface: menus, loaded schematics, placements, and verifier screens.

use crate::malilib::gui::{ButtonWidget, GuiScreen, Widget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuTab {
    LoadedSchematics,
    SchematicPlacements,
    SchematicVerifier,
    MaterialList,
    AreaSelection,
    Configuration,
}

pub struct LitematicaMainMenu {
    pub active_tab: MainMenuTab,
    pub buttons: Vec<ButtonWidget>,
    pub width: u32,
    pub height: u32,
    pub status_message: String,
}

impl LitematicaMainMenu {
    pub fn new() -> Self {
        Self {
            active_tab: MainMenuTab::LoadedSchematics,
            buttons: Vec::new(),
            width: 800,
            height: 600,
            status_message: "Litematica (Horizon Port) Ready".to_string(),
        }
    }

    pub fn set_tab(&mut self, tab: MainMenuTab) {
        self.active_tab = tab;
        self.rebuild_buttons();
    }

    fn rebuild_buttons(&mut self) {
        self.buttons.clear();
        let tabs = [
            ("Loaded Schematics", MainMenuTab::LoadedSchematics),
            ("Placements", MainMenuTab::SchematicPlacements),
            ("Verifier", MainMenuTab::SchematicVerifier),
            ("Material List", MainMenuTab::MaterialList),
            ("Area Selection", MainMenuTab::AreaSelection),
            ("Configuration", MainMenuTab::Configuration),
        ];

        let mut x = 20;
        let y = 30;
        let width = 120;
        let height = 24;

        for (label, tab) in tabs {
            let mut btn = ButtonWidget::new(x, y, width, height, label);
            if self.active_tab == tab {
                btn.enabled = false;
            }
            self.buttons.push(btn);
            x += width + 8;
        }
    }
}

impl GuiScreen for LitematicaMainMenu {
    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.rebuild_buttons();
    }

    fn render(&mut self, _mouse_x: i32, _mouse_y: i32, _delta: f32) {
        // Horizon UI render dispatch
    }

    fn on_click(&mut self, mouse_x: i32, mouse_y: i32, button: i32) -> bool {
        for btn in &mut self.buttons {
            if btn.on_click(mouse_x, mouse_y, button) {
                return true;
            }
        }
        false
    }

    fn on_key(&mut self, key: i32, _scan_code: i32, action: i32, _modifiers: i32) -> bool {
        if action == 1 && key == crate::malilib::input::KEY_ESCAPE {
            return true;
        }
        false
    }
}
