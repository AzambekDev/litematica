//! Material list calculation and inventory tracking.

use std::collections::HashMap;

use crate::litematica::schematic::LitematicaSchematic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialEntry {
    pub item_name: String,
    pub total_required: u64,
    pub available: u64,
    pub missing: u64,
}

pub struct MaterialList;

impl MaterialList {
    pub fn calculate(
        schematic: &LitematicaSchematic,
        inventory: &HashMap<String, u64>,
    ) -> Vec<MaterialEntry> {
        let mut required_counts: HashMap<String, u64> = HashMap::new();

        for region in schematic.regions.values() {
            for &p_idx in &region.blocks {
                if let Some(state) = region.palette.get(p_idx as usize) {
                    if !state.is_air() {
                        *required_counts.entry(state.name.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut entries: Vec<MaterialEntry> = required_counts
            .into_iter()
            .map(|(name, total)| {
                let avail = inventory.get(&name).copied().unwrap_or(0);
                let missing = if total > avail { total - avail } else { 0 };
                MaterialEntry {
                    item_name: name,
                    total_required: total,
                    available: avail,
                    missing,
                }
            })
            .collect();

        // Sort descending by missing count, then total required
        entries.sort_by(|a, b| b.missing.cmp(&a.missing).then_with(|| b.total_required.cmp(&a.total_required)));
        entries
    }
}
