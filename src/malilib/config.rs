//! Typed configuration system supporting Booleans, Integers, Doubles, Strings, Colors, and Hotkeys.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    Boolean(bool),
    Integer {
        value: i32,
        min: i32,
        max: i32,
        default: i32,
    },
    Double {
        value: f64,
        min: f64,
        max: f64,
        default: f64,
    },
    String(String),
    Color {
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    },
    Hotkey {
        key: i32,
        modifiers: i32,
    },
    Enum {
        current: usize,
        options: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct ConfigOption {
    pub name: String,
    pub comment: String,
    pub value: ConfigValue,
}

impl ConfigOption {
    pub fn new_bool(name: &str, default: bool, comment: &str) -> Self {
        Self {
            name: name.to_string(),
            comment: comment.to_string(),
            value: ConfigValue::Boolean(default),
        }
    }

    pub fn new_int(name: &str, default: i32, min: i32, max: i32, comment: &str) -> Self {
        Self {
            name: name.to_string(),
            comment: comment.to_string(),
            value: ConfigValue::Integer {
                value: default,
                min,
                max,
                default,
            },
        }
    }

    pub fn new_double(name: &str, default: f64, min: f64, max: f64, comment: &str) -> Self {
        Self {
            name: name.to_string(),
            comment: comment.to_string(),
            value: ConfigValue::Double {
                value: default,
                min,
                max,
                default,
            },
        }
    }

    pub fn new_string(name: &str, default: &str, comment: &str) -> Self {
        Self {
            name: name.to_string(),
            comment: comment.to_string(),
            value: ConfigValue::String(default.to_string()),
        }
    }

    pub fn new_color(name: &str, r: f32, g: f32, b: f32, a: f32, comment: &str) -> Self {
        Self {
            name: name.to_string(),
            comment: comment.to_string(),
            value: ConfigValue::Color { r, g, b, a },
        }
    }

    pub fn new_hotkey(name: &str, key: i32, modifiers: i32, comment: &str) -> Self {
        Self {
            name: name.to_string(),
            comment: comment.to_string(),
            value: ConfigValue::Hotkey { key, modifiers },
        }
    }

    pub fn new_enum(name: &str, default_idx: usize, options: &[&str], comment: &str) -> Self {
        Self {
            name: name.to_string(),
            comment: comment.to_string(),
            value: ConfigValue::Enum {
                current: default_idx,
                options: options.iter().map(|s| s.to_string()).collect(),
            },
        }
    }

    pub fn get_bool(&self) -> bool {
        if let ConfigValue::Boolean(b) = self.value {
            b
        } else {
            false
        }
    }

    pub fn set_bool(&mut self, val: bool) {
        if let ConfigValue::Boolean(ref mut b) = self.value {
            *b = val;
        }
    }

    pub fn get_int(&self) -> i32 {
        if let ConfigValue::Integer { value, .. } = self.value {
            value
        } else {
            0
        }
    }

    pub fn set_int(&mut self, val: i32) {
        if let ConfigValue::Integer { ref mut value, min, max, .. } = self.value {
            *value = val.clamp(min, max);
        }
    }

    pub fn get_double(&self) -> f64 {
        if let ConfigValue::Double { value, .. } = self.value {
            value
        } else {
            0.0
        }
    }

    pub fn get_color(&self) -> (f32, f32, f32, f32) {
        if let ConfigValue::Color { r, g, b, a } = self.value {
            (r, g, b, a)
        } else {
            (1.0, 1.0, 1.0, 1.0)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfigCategory {
    Generic,
    Visuals,
    Colors,
    Hotkeys,
    InfoOverlays,
}

impl ConfigCategory {
    pub fn name(&self) -> &'static str {
        match self {
            ConfigCategory::Generic => "Generic",
            ConfigCategory::Visuals => "Visuals",
            ConfigCategory::Colors => "Colors",
            ConfigCategory::Hotkeys => "Hotkeys",
            ConfigCategory::InfoOverlays => "InfoOverlays",
        }
    }
}

pub struct ConfigManager {
    categories: HashMap<String, Vec<ConfigOption>>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            categories: HashMap::new(),
        }
    }

    pub fn add_option(&mut self, category: &str, option: ConfigOption) {
        self.categories
            .entry(category.to_string())
            .or_default()
            .push(option);
    }

    pub fn get_option(&self, category: &str, name: &str) -> Option<&ConfigOption> {
        self.categories
            .get(category)
            .and_then(|list| list.iter().find(|opt| opt.name == name))
    }

    pub fn get_option_mut(&mut self, category: &str, name: &str) -> Option<&mut ConfigOption> {
        self.categories
            .get_mut(category)
            .and_then(|list| list.iter_mut().find(|opt| opt.name == name))
    }

    pub fn get_category_options(&self, category: &str) -> Option<&Vec<ConfigOption>> {
        self.categories.get(category)
    }

    pub fn save_to_json(&self) -> String {
        let mut out = String::from("{\n");
        let mut cat_entries = self.categories.iter().peekable();
        while let Some((cat_name, options)) = cat_entries.next() {
            out.push_str(&format!("  \"{}\": {{\n", cat_name));
            let mut opt_entries = options.iter().peekable();
            while let Some(opt) = opt_entries.next() {
                let val_str = match &opt.value {
                    ConfigValue::Boolean(b) => format!("{}", b),
                    ConfigValue::Integer { value, .. } => format!("{}", value),
                    ConfigValue::Double { value, .. } => format!("{}", value),
                    ConfigValue::String(s) => format!("\"{}\"", s),
                    ConfigValue::Color { r, g, b, a } => {
                        format!("{{\"r\": {}, \"g\": {}, \"b\": {}, \"a\": {}}}", r, g, b, a)
                    }
                    ConfigValue::Hotkey { key, modifiers } => {
                        format!("{{\"key\": {}, \"modifiers\": {}}}", key, modifiers)
                    }
                    ConfigValue::Enum { current, options } => {
                        let current_opt = options.get(*current).map(|s| s.as_str()).unwrap_or("");
                        format!("\"{}\"", current_opt)
                    }
                };
                let comma = if opt_entries.peek().is_some() { "," } else { "" };
                out.push_str(&format!("    \"{}\": {}{}\n", opt.name, val_str, comma));
            }
            let cat_comma = if cat_entries.peek().is_some() { "," } else { "" };
            out.push_str(&format!("  }}{}\n", cat_comma));
        }
        out.push_str("}\n");
        out
    }
}
