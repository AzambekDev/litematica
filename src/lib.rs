//! Litematica & MaLiLib native mod for Horizon (Rust Minecraft).

pub mod malilib;
pub mod litematica;

pub use litematica::{init, on_key_press, render_hud, render_world};

#[no_mangle]
pub extern "C" fn horizon_mod_init() {
    litematica::init();
}

#[no_mangle]
pub extern "C" fn horizon_mod_cleanup() {
    // Cleanup mod resources
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use crate::malilib::math::{BlockBox, BlockPos, Direction, Mirror, Rotation};
    use crate::malilib::nbt::{BitArray, NbtTag, parse_nbt_buffer, serialize_nbt_buffer};
    use crate::malilib::config::{ConfigManager, ConfigOption};
    use crate::malilib::input::{HotkeyManager, Keybind, KEY_M};
    use crate::litematica::schematic::{BlockState, LitematicaSchematic, SchematicRegion};
    use crate::litematica::schematic::litematic::{read_litematic, write_litematic};
    use crate::litematica::placement::SchematicPlacement;
    use crate::litematica::verifier::{MismatchType, SchematicVerifier, SimpleWorld};
    use crate::litematica::materials::MaterialList;
    use crate::litematica::selection::AreaSelection;

    #[test]
    fn test_math_primitives() {
        let pos1 = BlockPos::new(10, 64, -20);
        let pos2 = pos1.offset(Direction::Up);
        assert_eq!(pos2, BlockPos::new(10, 65, -20));

        let bbox = BlockBox::from_corners(&BlockPos::new(0, 0, 0), &BlockPos::new(4, 4, 4));
        assert_eq!(bbox.size_x(), 5);
        assert_eq!(bbox.volume(), 125);
        assert!(bbox.contains(&BlockPos::new(2, 2, 2)));
        assert!(!bbox.contains(&BlockPos::new(6, 2, 2)));

        let p = BlockPos::new(1, 0, 0);
        let rotated = Rotation::Clockwise90.rotate_pos(&p, 2, 2);
        assert_eq!(rotated, BlockPos::new(2, 0, 1));

        let mirrored = Mirror::LeftRight.mirror_pos(&p, 2, 2);
        assert_eq!(mirrored, BlockPos::new(1, 0, 0));
    }

    #[test]
    fn test_nbt_serialization_roundtrip() {
        let mut map = HashMap::new();
        map.insert("StringVal".to_string(), NbtTag::String("Hello Horizon".to_string()));
        map.insert("IntVal".to_string(), NbtTag::Int(12345));
        map.insert("LongVal".to_string(), NbtTag::Long(9876543210));
        map.insert("LongArray".to_string(), NbtTag::LongArray(vec![10, 20, 30]));

        let tag = NbtTag::Compound(map);
        let serialized = serialize_nbt_buffer("Root", &tag).expect("Failed to serialize NBT");
        let (name, deserialized) = parse_nbt_buffer(&serialized).expect("Failed to parse NBT");

        assert_eq!(name, "Root");
        assert_eq!(tag, deserialized);
    }

    #[test]
    fn test_bit_array_packing() {
        let mut bit_arr = BitArray::new(4, 16);
        bit_arr.set(0, 1);
        bit_arr.set(1, 5);
        bit_arr.set(2, 15);
        bit_arr.set(15, 7);

        assert_eq!(bit_arr.get(0), 1);
        assert_eq!(bit_arr.get(1), 5);
        assert_eq!(bit_arr.get(2), 15);
        assert_eq!(bit_arr.get(15), 7);
        assert_eq!(bit_arr.get(3), 0);
    }

    #[test]
    fn test_litematic_roundtrip() {
        let mut schematic = LitematicaSchematic::new("TestHouse", "masa");
        let mut region = SchematicRegion::new("Main", BlockPos::new(0, 0, 0), BlockPos::new(3, 3, 3));

        let stone = BlockState::new("minecraft:stone");
        let oak_planks = BlockState::new("minecraft:oak_planks").with_property("axis", "y");

        region.set_block(0, 0, 0, stone.clone());
        region.set_block(1, 1, 1, oak_planks.clone());
        schematic.add_region(region);

        let encoded = write_litematic(&schematic).expect("Failed to write litematic");
        assert!(!encoded.is_empty());

        let decoded = read_litematic(&encoded).expect("Failed to read litematic");
        assert_eq!(decoded.metadata.name, "TestHouse");
        assert_eq!(decoded.metadata.author, "masa");
        assert_eq!(decoded.regions.len(), 1);

        let decoded_reg = decoded.regions.get("Main").expect("Region 'Main' missing");
        assert_eq!(decoded_reg.get_block(0, 0, 0), Some(&stone));
        assert_eq!(decoded_reg.get_block(1, 1, 1), Some(&oak_planks));
    }

    #[test]
    fn test_schematic_verifier() {
        let mut schematic = LitematicaSchematic::new("Test", "Author");
        let mut region = SchematicRegion::new("Main", BlockPos::ORIGIN, BlockPos::new(2, 1, 1));
        let stone = BlockState::new("minecraft:stone");
        let dirt = BlockState::new("minecraft:dirt");

        region.set_block(0, 0, 0, stone.clone());
        region.set_block(1, 0, 0, dirt.clone());
        schematic.add_region(region);

        let placement = SchematicPlacement::new("P1", Arc::new(schematic), BlockPos::new(100, 64, 100));

        let mut world = SimpleWorld::new();
        // At pos (100, 64, 100): place stone (exact match)
        world.set_block(BlockPos::new(100, 64, 100), stone.clone());
        // At pos (101, 64, 100): place air (missing block)
        world.set_block(BlockPos::new(101, 64, 100), BlockState::new(BlockState::AIR));

        let report = SchematicVerifier::verify(&placement, &world);
        assert_eq!(report.correct, 1);
        assert_eq!(report.missing, 1);
        assert_eq!(report.mismatches.len(), 1);
        assert_eq!(report.mismatches[0].mismatch_type, MismatchType::Missing);
    }

    #[test]
    fn test_material_list() {
        let mut schematic = LitematicaSchematic::new("TestMaterials", "Author");
        let mut region = SchematicRegion::new("Main", BlockPos::ORIGIN, BlockPos::new(2, 2, 2));
        let stone = BlockState::new("minecraft:stone");
        let glass = BlockState::new("minecraft:glass");

        region.set_block(0, 0, 0, stone.clone());
        region.set_block(1, 0, 0, stone.clone());
        region.set_block(0, 1, 0, glass.clone());
        schematic.add_region(region);

        let mut inv = HashMap::new();
        inv.insert("minecraft:stone".to_string(), 1); // 1 available, 2 required -> missing 1

        let list = MaterialList::calculate(&schematic, &inv);
        assert_eq!(list.len(), 2);

        let stone_entry = list.iter().find(|e| e.item_name == "minecraft:stone").unwrap();
        assert_eq!(stone_entry.total_required, 2);
        assert_eq!(stone_entry.available, 1);
        assert_eq!(stone_entry.missing, 1);

        let glass_entry = list.iter().find(|e| e.item_name == "minecraft:glass").unwrap();
        assert_eq!(glass_entry.total_required, 1);
        assert_eq!(glass_entry.available, 0);
        assert_eq!(glass_entry.missing, 1);
    }

    #[test]
    fn test_area_selection_and_export() {
        let mut sel = AreaSelection::new("Selection1");
        sel.set_pos1(BlockPos::new(10, 64, 10));
        sel.set_pos2(BlockPos::new(11, 64, 11));

        let mut world = SimpleWorld::new();
        world.set_block(BlockPos::new(10, 64, 10), BlockState::new("minecraft:diamond_block"));

        let bytes = sel.save_to_schematic("Exported", "Player", &world).expect("Export failed");
        let decoded = read_litematic(&bytes).expect("Failed to read exported schematic");
        assert_eq!(decoded.metadata.name, "Exported");
    }

    #[test]
    fn test_config_and_hotkey() {
        let mut cfg = ConfigManager::new();
        cfg.add_option("Generic", ConfigOption::new_bool("testFlag", true, "Test"));
        assert!(cfg.get_option("Generic", "testFlag").unwrap().get_bool());

        let json = cfg.save_to_json();
        assert!(json.contains("testFlag"));

        let mut hotkeys = HotkeyManager::new();
        hotkeys.register("testKey", Keybind::new(KEY_M, 0), || {
            true
        });
        assert!(hotkeys.handle_key_press(KEY_M, 1, 0));
        assert!(!hotkeys.handle_key_press(KEY_M, 0, 0)); // release does not trigger
    }
}
