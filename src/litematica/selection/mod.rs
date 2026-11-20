//! Area selection and exporting world regions into .litematic schematics.

use crate::litematica::schematic::litematic::write_litematic;
use crate::litematica::schematic::{LitematicaSchematic, SchematicRegion};
use crate::litematica::verifier::WorldBlockQuery;
use crate::malilib::math::{BlockBox, BlockPos};

#[derive(Debug, Clone)]
pub struct AreaSelection {
    pub name: String,
    pub pos1: Option<BlockPos>,
    pub pos2: Option<BlockPos>,
}

impl AreaSelection {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            pos1: None,
            pos2: None,
        }
    }

    pub fn set_pos1(&mut self, pos: BlockPos) {
        self.pos1 = Some(pos);
    }

    pub fn set_pos2(&mut self, pos: BlockPos) {
        self.pos2 = Some(pos);
    }

    pub fn get_bounding_box(&self) -> Option<BlockBox> {
        match (self.pos1, self.pos2) {
            (Some(p1), Some(p2)) => Some(BlockBox::from_corners(&p1, &p2)),
            (Some(p1), None) => Some(BlockBox::from_corners(&p1, &p1)),
            (None, Some(p2)) => Some(BlockBox::from_corners(&p2, &p2)),
            (None, None) => None,
        }
    }

    pub fn save_to_schematic<W: WorldBlockQuery>(
        &self,
        name: &str,
        author: &str,
        world: &W,
    ) -> Option<Vec<u8>> {
        let bbox = self.get_bounding_box()?;
        let mut schematic = LitematicaSchematic::new(name, author);

        let origin = bbox.min_pos();
        let size = BlockPos::new(bbox.size_x(), bbox.size_y(), bbox.size_z());
        let mut region = SchematicRegion::new(name, BlockPos::ORIGIN, size);

        for y in 0..bbox.size_y() {
            for z in 0..bbox.size_z() {
                for x in 0..bbox.size_x() {
                    let world_pos = origin.add(x, y, z);
                    let state = world.get_block_state(&world_pos);
                    region.set_block(x, y, z, state);
                }
            }
        }

        schematic.add_region(region);
        write_litematic(&schematic).ok()
    }
}
