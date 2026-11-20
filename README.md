# Litematica for Horizon (Rust Minecraft)

An idiomatic Rust rewrite and refactor of **Litematica** and its prerequisite core library **MaLiLib**, packaged for the **Horizon** native Rust Minecraft engine.

## Overview

In the Java Minecraft ecosystem, Litematica (developed by masa and contributed by Sakura-Ryoko) is the standard schematic mod for building, displaying ghost block overlays, calculating material lists, verifying placed structures, and exporting areas. It depends upon MaLiLib for configuration management, hotkey binding, math, rendering utilities, and GUI screens.

Because Horizon is a clean-room reimplementation of Minecraft in Rust without a JVM, standard Java JAR files do not execute directly. This package refactors and rebuilds both mods into native Rust, producing a unified `.hmod` package.

## Features

- **Multi-Format Schematic Decoding & Encoding**:
  - Full `.litematic` (v5 & v6) support with NBT compound tags, metadata, sub-regions, block state palettes, and bit-packed block state arrays (`BitArray`).
  - Sponge `.schem` format reader.
  - Legacy Minecraft `.schematic` format reader.
- **Schematic Placements & Sub-Regions**:
  - Independent positioning, 90°/180°/270° rotation, and X/Z mirroring.
  - Sub-region toggling and relative offset positioning.
- **Schematic Verifier**:
  - Tick-accurate world comparison oracle.
  - Color-coded overlay classification:
    - **Missing blocks**: See-through ghost blocks (translucent cyan/blue)
    - **Wrong blocks**: Highlighted in red
    - **Wrong block states**: Highlighted in orange
    - **Extra blocks**: Highlighted in magenta
- **Material List Calculation**:
  - Aggregation of all required blocks and items across schematic regions.
  - Real-time comparison with player inventory and missing item counts.
- **Area Selection & Saving**:
  - Tool stick selection (corner 1 and corner 2).
  - One-click export to standard `.litematic` files.
- **Layer Slicing Modes**:
  - `All`: Render complete schematic.
  - `AllBelow`: Render all layers up to the current layer.
  - `SingleLayer`: Render only the active layer.
  - `LayerRange`: Render between configured min and max Y layers.
  - Quick hotkeys: `Page Up` / `Page Down` for layer navigation.
- **MaLiLib Subsystem in Rust**:
  - Typed configuration system (Boolean, Integer, Double, String, Color, Hotkey, Enum).
  - Hotkey & keybinding manager with modifier support (Shift, Ctrl, Alt).
  - In-game GUI menu (`M` key) with tab navigation.
  - Command interface: `/litematica` and `/schematics`.

## Package Structure (`.hmod`)

```text
litematica.hmod/
├── horizon-mod.toml        # Root manifest with metadata, capabilities, ported-from tracking
├── horizon-mixin.toml      # Mixin injection declarations for Horizon client hooks
├── Cargo.toml              # Rust crate manifest
├── src/                    # Refactored Rust source code
│   ├── lib.rs              # Mod entrypoints & test suite
│   ├── malilib/            # Rebuilt MaLiLib core library
│   │   ├── config.rs       # Configuration options & JSON persistence
│   │   ├── gui.rs          # UI widget framework & screens
│   │   ├── input.rs        # Keycodes & hotkey dispatch
│   │   ├── math.rs         # BlockPos, BlockBox, Rotation, Mirror
│   │   ├── nbt.rs          # NBT serializer/parser & BitArray packing
│   │   └── render.rs       # 3D line, quad, and box rendering
│   └── litematica/         # Rebuilt Litematica main mod
│       ├── commands/       # /litematica and /schematics commands
│       ├── materials/      # Material list & inventory checks
│       ├── placement/      # Placement transforms & sub-regions
│       ├── render/         # Ghost block renderer & layer slicing
│       ├── schematic/      # .litematic, .schem, .schematic codecs
│       ├── selection/      # Area selection & export to file
│       └── ui/             # In-game GUI screens
├── mixins/                 # Injection helpers for Horizon engine
│   ├── hud_hooks.rs
│   ├── input_hooks.rs
│   └── render_hooks.rs
└── assets/                 # Carried-over textures, models, blockstates, lang files
    ├── litematica/
    └── malilib/
```

## License

Original Java mods by masa & Sakura-Ryoko are licensed under LGPLv3. This port maintains full license compliance under LGPLv3.
