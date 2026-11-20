//! Schematic verifier: compares placed schematic blocks against world blocks.

use std::collections::HashMap;

use crate::litematica::placement::SchematicPlacement;
use crate::litematica::schematic::BlockState;
use crate::malilib::math::BlockPos;
use crate::malilib::render::Color4f;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MismatchType {
    Missing,     // Expected block, found air
    WrongBlock,  // Expected block X, found block Y
    WrongState,  // Expected block X with state A, found block X with state B
    Extra,       // Expected air, found block
}

impl MismatchType {
    pub fn overlay_color(&self) -> Color4f {
        match self {
            MismatchType::Missing => Color4f::OVERLAY_MISSING,
            MismatchType::WrongBlock => Color4f::OVERLAY_WRONG_BLOCK,
            MismatchType::WrongState => Color4f::OVERLAY_WRONG_STATE,
            MismatchType::Extra => Color4f::OVERLAY_EXTRA_BLOCK,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MismatchEntry {
    pub world_pos: BlockPos,
    pub expected: BlockState,
    pub actual: BlockState,
    pub mismatch_type: MismatchType,
}

#[derive(Debug, Clone, Default)]
pub struct VerifierReport {
    pub total_checked: u64,
    pub correct: u64,
    pub missing: u64,
    pub wrong_block: u64,
    pub wrong_state: u64,
    pub extra: u64,
    pub mismatches: Vec<MismatchEntry>,
}

pub trait WorldBlockQuery {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;
}

pub struct SimpleWorld {
    pub blocks: HashMap<BlockPos, BlockState>,
}

impl SimpleWorld {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }

    pub fn set_block(&mut self, pos: BlockPos, state: BlockState) {
        self.blocks.insert(pos, state);
    }
}

impl WorldBlockQuery for SimpleWorld {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState {
        self.blocks
            .get(pos)
            .cloned()
            .unwrap_or_else(|| BlockState::new(BlockState::AIR))
    }
}

pub struct SchematicVerifier;

impl SchematicVerifier {
    pub fn verify<W: WorldBlockQuery>(
        placement: &SchematicPlacement,
        world: &W,
    ) -> VerifierReport {
        let mut report = VerifierReport::default();
        let bbox = placement.schematic.get_bounding_box();

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
                        let expected = region
                            .get_block(x, y, z)
                            .cloned()
                            .unwrap_or_else(|| BlockState::new(BlockState::AIR));

                        let local_pos = region.position.add(x, y, z);
                        let world_pos = placement.transform_pos(&local_pos, &bbox);
                        let actual = world.get_block_state(&world_pos);

                        report.total_checked += 1;

                        if expected.is_air() && actual.is_air() {
                            report.correct += 1;
                        } else if expected == actual {
                            report.correct += 1;
                        } else if !expected.is_air() && actual.is_air() {
                            report.missing += 1;
                            report.mismatches.push(MismatchEntry {
                                world_pos,
                                expected,
                                actual,
                                mismatch_type: MismatchType::Missing,
                            });
                        } else if expected.is_air() && !actual.is_air() {
                            report.extra += 1;
                            report.mismatches.push(MismatchEntry {
                                world_pos,
                                expected,
                                actual,
                                mismatch_type: MismatchType::Extra,
                            });
                        } else if expected.name != actual.name {
                            report.wrong_block += 1;
                            report.mismatches.push(MismatchEntry {
                                world_pos,
                                expected,
                                actual,
                                mismatch_type: MismatchType::WrongBlock,
                            });
                        } else {
                            // Name matches, but properties differ
                            report.wrong_state += 1;
                            report.mismatches.push(MismatchEntry {
                                world_pos,
                                expected,
                                actual,
                                mismatch_type: MismatchType::WrongState,
                            });
                        }
                    }
                }
            }
        }

        report
    }
}
