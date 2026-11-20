//! In-game HUD hooks into Horizon engine
use crate::litematica;

/// Hook called when rendering the in-game HUD to draw InfoHUD, selected layer info, and active tool status.
pub fn on_render_hud(screen_width: u32, screen_height: u32, delta_time: f32) {
    litematica::render_hud(screen_width, screen_height, delta_time);
}
