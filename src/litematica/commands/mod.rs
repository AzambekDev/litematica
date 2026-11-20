//! Command handler for /litematica and /schematics.

#[derive(Debug, PartialEq, Eq)]
pub enum LitematicaCommand {
    Help,
    ListLoaded,
    UnloadAll,
    Rotate,
    Mirror,
    LayerUp,
    LayerDown,
    SaveArea(String),
}

pub struct CommandHandler;

impl CommandHandler {
    pub fn parse(input: &str) -> Option<LitematicaCommand> {
        let trimmed = input.trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let prefix = parts[0];
        if prefix != "/litematica" && prefix != "/schematics" && prefix != "/sch" {
            return None;
        }

        if parts.len() == 1 {
            return Some(LitematicaCommand::Help);
        }

        match parts[1] {
            "list" => Some(LitematicaCommand::ListLoaded),
            "unload" => Some(LitematicaCommand::UnloadAll),
            "rotate" => Some(LitematicaCommand::Rotate),
            "mirror" => Some(LitematicaCommand::Mirror),
            "layer-up" | "up" => Some(LitematicaCommand::LayerUp),
            "layer-down" | "down" => Some(LitematicaCommand::LayerDown),
            "save" => {
                let name = if parts.len() > 2 { parts[2] } else { "schematic" };
                Some(LitematicaCommand::SaveArea(name.to_string()))
            }
            _ => Some(LitematicaCommand::Help),
        }
    }
}
