//! Litematic format (.litematic) encoder and decoder.

use std::collections::HashMap;
use std::io;

use crate::litematica::schematic::{BlockState, LitematicaSchematic, SchematicMetadata, SchematicRegion};
use crate::malilib::math::BlockPos;
use crate::malilib::nbt::{BitArray, NbtTag, parse_nbt_buffer, serialize_nbt_buffer};

pub fn read_litematic(bytes: &[u8]) -> io::Result<LitematicaSchematic> {
    let (_, root_tag) = parse_nbt_buffer(bytes)?;
    let root = root_tag.as_compound().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "Root tag is not a compound")
    })?;

    let version = root.get("Version").and_then(|t| t.as_int()).unwrap_or(6);
    let minecraft_data_version = root
        .get("MinecraftDataVersion")
        .and_then(|t| t.as_int())
        .unwrap_or(3955);

    let mut metadata = SchematicMetadata::default();
    if let Some(NbtTag::Compound(meta_map)) = root.get("Metadata") {
        if let Some(name) = meta_map.get("Name").and_then(|t| t.as_string()) {
            metadata.name = name.to_string();
        }
        if let Some(author) = meta_map.get("Author").and_then(|t| t.as_string()) {
            metadata.author = author.to_string();
        }
        if let Some(desc) = meta_map.get("Description").and_then(|t| t.as_string()) {
            metadata.description = desc.to_string();
        }
        if let Some(tc) = meta_map.get("TimeCreated").and_then(|t| t.as_long()) {
            metadata.time_created = tc;
        }
        if let Some(tm) = meta_map.get("TimeModified").and_then(|t| t.as_long()) {
            metadata.time_modified = tm;
        }
        if let Some(tb) = meta_map.get("TotalBlocks").and_then(|t| t.as_long()) {
            metadata.total_blocks = tb as u64;
        }
        if let Some(tv) = meta_map.get("TotalVolume").and_then(|t| t.as_long()) {
            metadata.total_volume = tv as u64;
        }
        if let Some(NbtTag::Compound(size_map)) = meta_map.get("EnclosingSize") {
            let x = size_map.get("x").and_then(|t| t.as_int()).unwrap_or(0);
            let y = size_map.get("y").and_then(|t| t.as_int()).unwrap_or(0);
            let z = size_map.get("z").and_then(|t| t.as_int()).unwrap_or(0);
            metadata.enclosing_size = BlockPos::new(x, y, z);
        }
    }

    let mut regions = HashMap::new();
    if let Some(NbtTag::Compound(regions_map)) = root.get("Regions") {
        for (reg_name, reg_tag) in regions_map {
            if let NbtTag::Compound(reg_map) = reg_tag {
                let pos = if let Some(NbtTag::Compound(pos_map)) = reg_map.get("Position") {
                    let x = pos_map.get("x").and_then(|t| t.as_int()).unwrap_or(0);
                    let y = pos_map.get("y").and_then(|t| t.as_int()).unwrap_or(0);
                    let z = pos_map.get("z").and_then(|t| t.as_int()).unwrap_or(0);
                    BlockPos::new(x, y, z)
                } else {
                    BlockPos::ORIGIN
                };

                let size = if let Some(NbtTag::Compound(size_map)) = reg_map.get("Size") {
                    let x = size_map.get("x").and_then(|t| t.as_int()).unwrap_or(0);
                    let y = size_map.get("y").and_then(|t| t.as_int()).unwrap_or(0);
                    let z = size_map.get("z").and_then(|t| t.as_int()).unwrap_or(0);
                    BlockPos::new(x, y, z)
                } else {
                    BlockPos::ORIGIN
                };

                let mut palette = Vec::new();
                if let Some(NbtTag::List(palette_list)) = reg_map.get("BlockStatePalette") {
                    for entry in palette_list {
                        if let NbtTag::Compound(entry_map) = entry {
                            let name = entry_map
                                .get("Name")
                                .and_then(|t| t.as_string())
                                .unwrap_or("minecraft:air");
                            let mut state = BlockState::new(name);
                            if let Some(NbtTag::Compound(props_map)) = entry_map.get("Properties") {
                                for (k, v) in props_map {
                                    if let Some(val_str) = v.as_string() {
                                        state.properties.insert(k.clone(), val_str.to_string());
                                    }
                                }
                            }
                            palette.push(state);
                        }
                    }
                }

                if palette.is_empty() {
                    palette.push(BlockState::new(BlockState::AIR));
                }

                let sx = size.x.abs() as usize;
                let sy = size.y.abs() as usize;
                let sz = size.z.abs() as usize;
                let volume = sx * sy * sz;

                let mut blocks = vec![0u32; volume];
                if let Some(NbtTag::LongArray(long_arr)) = reg_map.get("BlockStates") {
                    let bits = calculate_bits_per_entry(palette.len());
                    let bit_array = BitArray::from_data(bits, volume, long_arr.clone());
                    for i in 0..volume {
                        blocks[i] = bit_array.get(i);
                    }
                }

                let mut region = SchematicRegion::new(reg_name, pos, size);
                region.palette = palette;
                region.blocks = blocks;
                regions.insert(reg_name.clone(), region);
            }
        }
    }

    let _ = version;
    let mut schematic = LitematicaSchematic {
        metadata,
        regions,
        minecraft_data_version,
    };
    schematic.update_totals();
    Ok(schematic)
}

pub fn write_litematic(schematic: &LitematicaSchematic) -> io::Result<Vec<u8>> {
    let mut root_map = HashMap::new();
    root_map.insert("Version".to_string(), NbtTag::Int(6));
    root_map.insert("SubVersion".to_string(), NbtTag::Int(1));
    root_map.insert(
        "MinecraftDataVersion".to_string(),
        NbtTag::Int(schematic.minecraft_data_version),
    );

    // Metadata
    let mut meta_map = HashMap::new();
    meta_map.insert("Name".to_string(), NbtTag::String(schematic.metadata.name.clone()));
    meta_map.insert(
        "Author".to_string(),
        NbtTag::String(schematic.metadata.author.clone()),
    );
    meta_map.insert(
        "Description".to_string(),
        NbtTag::String(schematic.metadata.description.clone()),
    );
    meta_map.insert(
        "TimeCreated".to_string(),
        NbtTag::Long(schematic.metadata.time_created),
    );
    meta_map.insert(
        "TimeModified".to_string(),
        NbtTag::Long(schematic.metadata.time_modified),
    );
    meta_map.insert(
        "TotalBlocks".to_string(),
        NbtTag::Long(schematic.metadata.total_blocks as i64),
    );
    meta_map.insert(
        "TotalVolume".to_string(),
        NbtTag::Long(schematic.metadata.total_volume as i64),
    );

    let mut enclosing_map = HashMap::new();
    enclosing_map.insert("x".to_string(), NbtTag::Int(schematic.metadata.enclosing_size.x));
    enclosing_map.insert("y".to_string(), NbtTag::Int(schematic.metadata.enclosing_size.y));
    enclosing_map.insert("z".to_string(), NbtTag::Int(schematic.metadata.enclosing_size.z));
    meta_map.insert("EnclosingSize".to_string(), NbtTag::Compound(enclosing_map));
    root_map.insert("Metadata".to_string(), NbtTag::Compound(meta_map));

    // Regions
    let mut regions_map = HashMap::new();
    for (name, region) in &schematic.regions {
        let mut reg_map = HashMap::new();

        let mut pos_map = HashMap::new();
        pos_map.insert("x".to_string(), NbtTag::Int(region.position.x));
        pos_map.insert("y".to_string(), NbtTag::Int(region.position.y));
        pos_map.insert("z".to_string(), NbtTag::Int(region.position.z));
        reg_map.insert("Position".to_string(), NbtTag::Compound(pos_map));

        let mut size_map = HashMap::new();
        size_map.insert("x".to_string(), NbtTag::Int(region.size.x));
        size_map.insert("y".to_string(), NbtTag::Int(region.size.y));
        size_map.insert("z".to_string(), NbtTag::Int(region.size.z));
        reg_map.insert("Size".to_string(), NbtTag::Compound(size_map));

        // Palette
        let mut palette_list = Vec::new();
        for state in &region.palette {
            let mut state_map = HashMap::new();
            state_map.insert("Name".to_string(), NbtTag::String(state.name.clone()));
            if !state.properties.is_empty() {
                let mut props_map = HashMap::new();
                for (k, v) in &state.properties {
                    props_map.insert(k.clone(), NbtTag::String(v.clone()));
                }
                state_map.insert("Properties".to_string(), NbtTag::Compound(props_map));
            }
            palette_list.push(NbtTag::Compound(state_map));
        }
        reg_map.insert("BlockStatePalette".to_string(), NbtTag::List(palette_list));

        // BlockStates packing
        let bits = calculate_bits_per_entry(region.palette.len());
        let mut bit_array = BitArray::new(bits, region.blocks.len());
        for (i, &palette_idx) in region.blocks.iter().enumerate() {
            bit_array.set(i, palette_idx);
        }
        reg_map.insert(
            "BlockStates".to_string(),
            NbtTag::LongArray(bit_array.data().to_vec()),
        );

        reg_map.insert("TileEntities".to_string(), NbtTag::List(Vec::new()));
        reg_map.insert("Entities".to_string(), NbtTag::List(Vec::new()));
        reg_map.insert("PendingBlockTicks".to_string(), NbtTag::List(Vec::new()));

        regions_map.insert(name.clone(), NbtTag::Compound(reg_map));
    }
    root_map.insert("Regions".to_string(), NbtTag::Compound(regions_map));

    serialize_nbt_buffer("", &NbtTag::Compound(root_map))
}

fn calculate_bits_per_entry(palette_len: usize) -> usize {
    if palette_len <= 1 {
        2
    } else {
        let bits = (usize::BITS - (palette_len - 1).leading_zeros()) as usize;
        bits.max(2)
    }
}
