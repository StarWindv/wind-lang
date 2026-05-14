use super::gather::GatherContext;
use crate::modules::types::*;
use log::info;

pub struct LivenessAnalyzer {
    pub live_ranges: Vec<LiveRange>,
    pub drop_points: Vec<DropPoint>,
}

impl LivenessAnalyzer {
    pub fn new() -> Self {
        Self {
            live_ranges: Vec::new(),
            drop_points: Vec::new(),
        }
    }

    pub fn analyze(&mut self, ctx: &mut GatherContext) {
        info!("[Liveness] Starting liveness analysis");

        let mut position = 0usize;

        for (value_id, value_info) in &ctx.value_pool.values {
            let names = ctx.bindings.get_names_for_value(*value_id);
            let last_use = names.len() as usize;

            let live_range = LiveRange {
                value: *value_id,
                born_at: position,
                last_use: position + last_use,
                drop_at: None,
                dropped_by_scope_exit: false,
            };

            if value_info.ref_count == 0 {
                let drop_point = DropPoint {
                    value: *value_id,
                    at_position: position + last_use + 1,
                };
                self.drop_points.push(drop_point);
            }

            self.live_ranges.push(live_range);
            position += 1;
        }

        self.mark_scope_exit_drops(ctx);

        info!(
            "[Liveness] Found {} live ranges, {} drop points",
            self.live_ranges.len(),
            self.drop_points.len()
        );
    }

    fn mark_scope_exit_drops(&mut self, ctx: &GatherContext) {
        for scope in ctx.scope_tree.scopes.values() {
            let scope_exit_pos = self.live_ranges.len();

            for mangled_name in &scope.local_mangled_names {
                if let Some(&value_id) = ctx.bindings.name_to_value.get(mangled_name) {
                    if let Some(info) = ctx.value_pool.get(value_id) {
                        if info.ref_count <= 1 {
                            let drop_point = DropPoint {
                                value: value_id,
                                at_position: scope_exit_pos,
                            };

                            if !self.drop_points.iter().any(|dp| dp.value == value_id) {
                                self.drop_points.push(drop_point);
                            }
                        }
                    }
                }
            }
        }
    }
}
