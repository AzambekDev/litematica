//! Schematic placement, sub-regions, rotation, and mirror transformations.

use std::collections::HashMap;
use std::sync::Arc;

use crate::litematica::schematic::LitematicaSchematic;
use crate::malilib::math::{BlockBox, BlockPos, Mirror, Rotation};

#[derive(Debug, Clone)]
pub struct SubRegionPlacement {
    pub name: String,
    pub offset: BlockPos,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct SchematicPlacement {
    pub name: String,
    pub schematic: Arc<LitematicaSchematic>,
    pub origin: BlockPos,
    pub rotation: Rotation,
    pub mirror: Mirror,
    pub enabled: bool,
    pub locked: bool,
    pub sub_regions: HashMap<String, SubRegionPlacement>,
}

impl SchematicPlacement {
    pub fn new(name: &str, schematic: Arc<LitematicaSchematic>, origin: BlockPos) -> Self {
        let mut sub_regions = HashMap::new();
        for reg_name in schematic.regions.keys() {
            sub_regions.insert(
                reg_name.clone(),
                SubRegionPlacement {
                    name: reg_name.clone(),
                    offset: BlockPos::ORIGIN,
                    enabled: true,
                },
            );
        }

        Self {
            name: name.to_string(),
            schematic,
            origin,
            rotation: Rotation::None,
            mirror: Mirror::None,
            enabled: true,
            locked: false,
            sub_regions,
        }
    }

    pub fn cycle_rotation(&mut self) {
        if !self.locked {
            self.rotation = self.rotation.next();
        }
    }

    pub fn cycle_mirror(&mut self) {
        if !self.locked {
            self.mirror = self.mirror.next();
        }
    }

    pub fn get_bounding_box(&self) -> BlockBox {
        let raw_box = self.schematic.get_bounding_box();
        let min_pos = self.transform_pos(&raw_box.min_pos(), &raw_box);
        let max_pos = self.transform_pos(&raw_box.max_pos(), &raw_box);
        BlockBox::from_corners(&min_pos, &max_pos)
    }

    pub fn transform_pos(&self, pos: &BlockPos, bbox: &BlockBox) -> BlockPos {
        let rel_x = pos.x - bbox.min_x;
        let rel_y = pos.y - bbox.min_y;
        let rel_z = pos.z - bbox.min_z;
        let local = BlockPos::new(rel_x, rel_y, rel_z);

        // Apply mirror then rotation
        let mirrored = self.mirror.mirror_pos(&local, bbox.size_x() - 1, bbox.size_z() - 1);
        let rotated = self.rotation.rotate_pos(&mirrored, bbox.size_x() - 1, bbox.size_z() - 1);

        self.origin.add(rotated.x, rotated.y, rotated.z)
    }
}
