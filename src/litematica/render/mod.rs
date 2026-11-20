//! Ghost block rendering, layer slicing, and overlay generation.

use crate::litematica::placement::SchematicPlacement;
use crate::malilib::render::{Color4f, RenderBatch};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayerMode {
    #[default]
    All,
    AllBelow,
    SingleLayer,
    LayerRange,
}

#[derive(Debug, Clone)]
pub struct SchematicRenderer {
    pub layer_mode: LayerMode,
    pub current_layer: i32,
    pub min_layer: i32,
    pub max_layer: i32,
    pub render_ghost_blocks: bool,
    pub render_bounding_boxes: bool,
}

impl SchematicRenderer {
    pub fn new() -> Self {
        Self {
            layer_mode: LayerMode::All,
            current_layer: 0,
            min_layer: 0,
            max_layer: 256,
            render_ghost_blocks: true,
            render_bounding_boxes: true,
        }
    }

    pub fn page_up(&mut self) {
        self.current_layer += 1;
    }

    pub fn page_down(&mut self) {
        if self.current_layer > 0 {
            self.current_layer -= 1;
        }
    }

    pub fn is_layer_visible(&self, world_y: i32) -> bool {
        match self.layer_mode {
            LayerMode::All => true,
            LayerMode::AllBelow => world_y <= self.current_layer,
            LayerMode::SingleLayer => world_y == self.current_layer,
            LayerMode::LayerRange => world_y >= self.min_layer && world_y <= self.max_layer,
        }
    }

    pub fn build_render_batch(&self, placement: &SchematicPlacement, batch: &mut RenderBatch) {
        let bbox = placement.get_bounding_box();

        // 1. Draw placement outer bounding box
        if self.render_bounding_boxes {
            batch.add_box_wireframe(&bbox, Color4f::new(0.0, 1.0, 0.5, 0.8), 2.0);
        }

        // 2. Draw ghost blocks for visible layers
        if self.render_ghost_blocks {
            let schem_bbox = placement.schematic.get_bounding_box();
            for (reg_name, region) in &placement.schematic.regions {
                if let Some(sub_reg) = placement.sub_regions.get(reg_name) {
                    if !sub_reg.enabled {
                        continue;
                    }
                }

                let sx = region.size_x();
                let sy = region.size_y();
                let sz = region.size_z();

                for y in 0..sy {
                    for z in 0..sz {
                        for x in 0..sx {
                            let local_pos = region.position.add(x, y, z);
                            let world_pos = placement.transform_pos(&local_pos, &schem_bbox);

                            if !self.is_layer_visible(world_pos.y) {
                                continue;
                            }

                            if let Some(state) = region.get_block(x, y, z) {
                                if !state.is_air() {
                                    batch.add_shaded_block(&world_pos, Color4f::OVERLAY_MISSING);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
