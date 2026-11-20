//! Render hooks into Horizon engine
use crate::litematica;

/// Hook called after level rendering is completed to draw ghost blocks, bounding boxes, and overlays.
pub fn on_render_world_last(view_matrix: &[f32; 16], proj_matrix: &[f32; 16], camera_pos: [f64; 3]) {
    litematica::render_world(view_matrix, proj_matrix, camera_pos);
}
