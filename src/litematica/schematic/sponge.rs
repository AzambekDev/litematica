//! Sponge schematic (.schem) format decoder.

use std::io;

use crate::litematica::schematic::{BlockState, LitematicaSchematic, SchematicRegion};
use crate::malilib::math::BlockPos;
use crate::malilib::nbt::{NbtTag, parse_nbt_buffer};

pub fn read_sponge(bytes: &[u8]) -> io::Result<LitematicaSchematic> {
    let (_, root_tag) = parse_nbt_buffer(bytes)?;
    let root = root_tag.as_compound().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "Invalid Sponge schem root")
    })?;

    let width = root.get("Width").and_then(|t| t.as_int()).unwrap_or(0);
    let height = root.get("Height").and_then(|t| t.as_int()).unwrap_or(0);
    let length = root.get("Length").and_then(|t| t.as_int()).unwrap_or(0);

    let mut schematic = LitematicaSchematic::new("SpongeSchematic", "Unknown");
    let mut region = SchematicRegion::new(
        "Main",
        BlockPos::ORIGIN,
        BlockPos::new(width, height, length),
    );

    let mut palette_map: Vec<(String, u32)> = Vec::new();
    if let Some(NbtTag::Compound(palette_comp)) = root.get("Palette") {
        for (name, val) in palette_comp {
            if let Some(idx) = val.as_int() {
                palette_map.push((name.clone(), idx as u32));
            }
        }
    }
    palette_map.sort_by_key(|&(_, idx)| idx);

    let mut palette = Vec::new();
    for (name, _) in palette_map {
        palette.push(BlockState::new(&name));
    }
    if palette.is_empty() {
        palette.push(BlockState::new(BlockState::AIR));
    }
    region.palette = palette;

    if let Some(NbtTag::ByteArray(block_data)) = root.get("BlockData") {
        let volume = (width.abs() * height.abs() * length.abs()) as usize;
        let mut blocks = vec![0u32; volume];
        for i in 0..volume.min(block_data.len()) {
            blocks[i] = (block_data[i] as u8) as u32;
        }
        region.blocks = blocks;
    }

    schematic.add_region(region);
    Ok(schematic)
}
