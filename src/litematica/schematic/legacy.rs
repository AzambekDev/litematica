//! Legacy Minecraft (.schematic) format decoder.

use std::io;

use crate::litematica::schematic::{BlockState, LitematicaSchematic, SchematicRegion};
use crate::malilib::math::BlockPos;
use crate::malilib::nbt::{NbtTag, parse_nbt_buffer};

pub fn read_legacy(bytes: &[u8]) -> io::Result<LitematicaSchematic> {
    let (_, root_tag) = parse_nbt_buffer(bytes)?;
    let root = root_tag.as_compound().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "Invalid legacy schematic root")
    })?;

    let width = root.get("Width").and_then(|t| t.as_int()).unwrap_or(0);
    let height = root.get("Height").and_then(|t| t.as_int()).unwrap_or(0);
    let length = root.get("Length").and_then(|t| t.as_int()).unwrap_or(0);

    let mut schematic = LitematicaSchematic::new("LegacySchematic", "Unknown");
    let mut region = SchematicRegion::new(
        "Main",
        BlockPos::ORIGIN,
        BlockPos::new(width, height, length),
    );

    let mut palette = vec![BlockState::new(BlockState::AIR)];
    if let Some(NbtTag::ByteArray(blocks_tag)) = root.get("Blocks") {
        let volume = (width.abs() * height.abs() * length.abs()) as usize;
        let mut blocks = vec![0u32; volume];
        for i in 0..volume.min(blocks_tag.len()) {
            let id = blocks_tag[i] as u8;
            let block_name = map_legacy_id_to_name(id);
            let p_idx = if let Some(pos) = palette.iter().position(|s| s.name == block_name) {
                pos as u32
            } else {
                let pos = palette.len() as u32;
                palette.push(BlockState::new(&block_name));
                pos
            };
            blocks[i] = p_idx;
        }
        region.palette = palette;
        region.blocks = blocks;
    }

    schematic.add_region(region);
    Ok(schematic)
}

fn map_legacy_id_to_name(id: u8) -> String {
    match id {
        0 => "minecraft:air".to_string(),
        1 => "minecraft:stone".to_string(),
        2 => "minecraft:grass_block".to_string(),
        3 => "minecraft:dirt".to_string(),
        4 => "minecraft:cobblestone".to_string(),
        5 => "minecraft:oak_planks".to_string(),
        7 => "minecraft:bedrock".to_string(),
        12 => "minecraft:sand".to_string(),
        13 => "minecraft:gravel".to_string(),
        14 => "minecraft:gold_ore".to_string(),
        15 => "minecraft:iron_ore".to_string(),
        16 => "minecraft:coal_ore".to_string(),
        17 => "minecraft:oak_log".to_string(),
        18 => "minecraft:oak_leaves".to_string(),
        20 => "minecraft:glass".to_string(),
        35 => "minecraft:white_wool".to_string(),
        41 => "minecraft:gold_block".to_string(),
        42 => "minecraft:iron_block".to_string(),
        49 => "minecraft:obsidian".to_string(),
        56 => "minecraft:diamond_ore".to_string(),
        57 => "minecraft:diamond_block".to_string(),
        other => format!("minecraft:legacy_{}", other),
    }
}
