//! Schematic representation and data structures.

use std::collections::HashMap;
use crate::malilib::math::{BlockBox, BlockPos};

pub mod legacy;
pub mod litematic;
pub mod sponge;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockState {
    pub name: String,
    pub properties: HashMap<String, String>,
}

impl BlockState {
    pub const AIR: &'static str = "minecraft:air";

    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            properties: HashMap::new(),
        }
    }

    pub fn is_air(&self) -> bool {
        self.name == "minecraft:air" || self.name == "minecraft:cave_air" || self.name == "minecraft:void_air"
    }

    pub fn with_property(mut self, key: &str, value: &str) -> Self {
        self.properties.insert(key.to_string(), value.to_string());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct SchematicMetadata {
    pub name: String,
    pub author: String,
    pub description: String,
    pub time_created: i64,
    pub time_modified: i64,
    pub total_blocks: u64,
    pub total_volume: u64,
    pub enclosing_size: BlockPos,
}

#[derive(Debug, Clone)]
pub struct SchematicRegion {
    pub name: String,
    pub position: BlockPos,
    pub size: BlockPos,
    pub palette: Vec<BlockState>,
    pub blocks: Vec<u32>, // Flat palette indices [x + z * size_x + y * size_x * size_z]
}

impl SchematicRegion {
    pub fn new(name: &str, position: BlockPos, size: BlockPos) -> Self {
        let sx = size.x.abs() as usize;
        let sy = size.y.abs() as usize;
        let sz = size.z.abs() as usize;
        let volume = sx * sy * sz;
        let mut palette = Vec::new();
        palette.push(BlockState::new(BlockState::AIR));

        Self {
            name: name.to_string(),
            position,
            size,
            palette,
            blocks: vec![0; volume],
        }
    }

    pub fn size_x(&self) -> i32 {
        self.size.x.abs()
    }

    pub fn size_y(&self) -> i32 {
        self.size.y.abs()
    }

    pub fn size_z(&self) -> i32 {
        self.size.z.abs()
    }

    pub fn get_block_index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        let sx = self.size_x();
        let sy = self.size_y();
        let sz = self.size_z();
        if x < 0 || x >= sx || y < 0 || y >= sy || z < 0 || z >= sz {
            return None;
        }
        Some((x as usize) + (z as usize) * (sx as usize) + (y as usize) * (sx as usize) * (sz as usize))
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<&BlockState> {
        let idx = self.get_block_index(x, y, z)?;
        let palette_idx = self.blocks.get(idx)?;
        self.palette.get(*palette_idx as usize)
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, state: BlockState) {
        let idx = match self.get_block_index(x, y, z) {
            Some(i) => i,
            None => return,
        };

        // Find or insert into palette
        let palette_idx = if let Some(pos) = self.palette.iter().position(|s| s == &state) {
            pos as u32
        } else {
            let pos = self.palette.len() as u32;
            self.palette.push(state);
            pos
        };

        self.blocks[idx] = palette_idx;
    }
}

#[derive(Debug, Clone, Default)]
pub struct LitematicaSchematic {
    pub metadata: SchematicMetadata,
    pub regions: HashMap<String, SchematicRegion>,
    pub minecraft_data_version: i32,
}

impl LitematicaSchematic {
    pub fn new(name: &str, author: &str) -> Self {
        Self {
            metadata: SchematicMetadata {
                name: name.to_string(),
                author: author.to_string(),
                time_created: 0,
                time_modified: 0,
                total_blocks: 0,
                total_volume: 0,
                enclosing_size: BlockPos::ORIGIN,
                description: String::new(),
            },
            regions: HashMap::new(),
            minecraft_data_version: 3955, // 26.3 parity version
        }
    }

    pub fn add_region(&mut self, region: SchematicRegion) {
        self.regions.insert(region.name.clone(), region);
        self.update_totals();
    }

    pub fn update_totals(&mut self) {
        let mut total_blocks = 0;
        let mut total_volume = 0;
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut min_z = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        let mut max_z = i32::MIN;

        for region in self.regions.values() {
            let r_min_x = region.position.x;
            let r_min_y = region.position.y;
            let r_min_z = region.position.z;
            let r_max_x = r_min_x + region.size.x;
            let r_max_y = r_min_y + region.size.y;
            let r_max_z = r_min_z + region.size.z;

            min_x = min_x.min(r_min_x).min(r_max_x);
            min_y = min_y.min(r_min_y).min(r_max_y);
            min_z = min_z.min(r_min_z).min(r_max_z);
            max_x = max_x.max(r_min_x).max(r_max_x);
            max_y = max_y.max(r_min_y).max(r_max_y);
            max_z = max_z.max(r_min_z).max(r_max_z);

            let vol = (region.size_x() as u64) * (region.size_y() as u64) * (region.size_z() as u64);
            total_volume += vol;

            for &p_idx in &region.blocks {
                if let Some(state) = region.palette.get(p_idx as usize) {
                    if !state.is_air() {
                        total_blocks += 1;
                    }
                }
            }
        }

        self.metadata.total_blocks = total_blocks;
        self.metadata.total_volume = total_volume;
        if min_x != i32::MAX {
            self.metadata.enclosing_size = BlockPos::new(max_x - min_x, max_y - min_y, max_z - min_z);
        }
    }

    pub fn get_bounding_box(&self) -> BlockBox {
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut min_z = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        let mut max_z = i32::MIN;

        for region in self.regions.values() {
            let p1 = region.position;
            let p2 = region.position.add(region.size.x, region.size.y, region.size.z);
            min_x = min_x.min(p1.x).min(p2.x);
            min_y = min_y.min(p1.y).min(p2.y);
            min_z = min_z.min(p1.z).min(p2.z);
            max_x = max_x.max(p1.x).max(p2.x);
            max_y = max_y.max(p1.y).max(p2.y);
            max_z = max_z.max(p1.z).max(p2.z);
        }

        if min_x == i32::MAX {
            BlockBox::default()
        } else {
            BlockBox::new(min_x, min_y, min_z, max_x, max_y, max_z)
        }
    }
}
