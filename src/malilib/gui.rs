//! Lightweight GUI widget framework for menus, lists, and configuration dialogs.

pub trait Widget {
    fn render(&self, mouse_x: i32, mouse_y: i32);
    fn on_click(&mut self, mouse_x: i32, mouse_y: i32, button: i32) -> bool;
    fn get_bounds(&self) -> (i32, i32, i32, i32); // x, y, width, height
}

pub struct ButtonWidget {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub label: String,
    pub enabled: bool,
    pub on_press: Option<Box<dyn FnMut() + Send + Sync>>,
}

impl ButtonWidget {
    pub fn new(x: i32, y: i32, width: i32, height: i32, label: &str) -> Self {
        Self {
            x,
            y,
            width,
            height,
            label: label.to_string(),
            enabled: true,
            on_press: None,
        }
    }

    pub fn set_action<F: FnMut() + Send + Sync + 'static>(&mut self, action: F) {
        self.on_press = Some(Box::new(action));
    }
}

impl Widget for ButtonWidget {
    fn render(&self, _mouse_x: i32, _mouse_y: i32) {
        // Horizon UI rendering dispatch
    }

    fn on_click(&mut self, mouse_x: i32, mouse_y: i32, button: i32) -> bool {
        if !self.enabled || button != 0 {
            return false;
        }
        if mouse_x >= self.x && mouse_x < self.x + self.width
            && mouse_y >= self.y && mouse_y < self.y + self.height
        {
            if let Some(ref mut action) = self.on_press {
                action();
                return true;
            }
        }
        false
    }

    fn get_bounds(&self) -> (i32, i32, i32, i32) {
        (self.x, self.y, self.width, self.height)
    }
}

pub struct ListView<T> {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub entry_height: i32,
    pub entries: Vec<T>,
    pub selected_index: Option<usize>,
    pub scroll_offset: usize,
}

impl<T> ListView<T> {
    pub fn new(x: i32, y: i32, width: i32, height: i32, entry_height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            entry_height,
            entries: Vec::new(),
            selected_index: None,
            scroll_offset: 0,
        }
    }

    pub fn select(&mut self, idx: usize) {
        if idx < self.entries.len() {
            self.selected_index = Some(idx);
        }
    }

    pub fn selected(&self) -> Option<&T> {
        self.selected_index.and_then(|i| self.entries.get(i))
    }
}

pub trait GuiScreen: Send + Sync {
    fn init(&mut self, width: u32, height: u32);
    fn render(&mut self, mouse_x: i32, mouse_y: i32, delta: f32);
    fn on_click(&mut self, mouse_x: i32, mouse_y: i32, button: i32) -> bool;
    fn on_key(&mut self, key: i32, scan_code: i32, action: i32, modifiers: i32) -> bool;
    fn should_close_on_esc(&self) -> bool {
        true
    }
}

pub struct ScreenManager {
    screens: Vec<Box<dyn GuiScreen + Send + Sync>>,
}

impl ScreenManager {
    pub fn new() -> Self {
        Self { screens: Vec::new() }
    }

    pub fn open_screen(&mut self, mut screen: Box<dyn GuiScreen + Send + Sync>, width: u32, height: u32) {
        screen.init(width, height);
        self.screens.push(screen);
    }

    pub fn close_current(&mut self) {
        self.screens.pop();
    }

    pub fn has_active_screen(&self) -> bool {
        !self.screens.is_empty()
    }

    pub fn current_screen_mut(&mut self) -> Option<&mut Box<dyn GuiScreen + Send + Sync>> {
        self.screens.last_mut()
    }
}
