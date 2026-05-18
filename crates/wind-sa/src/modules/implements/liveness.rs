use super::super::types::GatherContext;
use crate::modules::types::*;
use log::debug;

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

    pub fn analyze(&mut self, ctx: &GatherContext) {
        debug!("[Liveness] Starting liveness analysis");

        let mut position = 0usize;
        let mut seen_dead: std::collections::HashSet<WindValueID> = std::collections::HashSet::new();

        for (_name, value_id) in &ctx.dead_values {
            let desc = self.describe_value(ctx, *value_id);
            seen_dead.insert(*value_id);

            if !self.drop_points.iter().any(|dp| dp.value == *value_id) {
                self.drop_points.push(DropPoint {
                    value: *value_id,
                    description: desc.clone(),
                    at_position: position,
                });
                position += 1;
            }

            if !self.live_ranges.iter().any(|lr| lr.value == *value_id) {
                self.live_ranges.push(LiveRange {
                    value: *value_id,
                    description: desc,
                    born_at: position,
                    last_use: position,
                    drop_at: Some(position),
                    dropped_by_scope_exit: true,
                });
            }
        }

        let dead_count = seen_dead.len();

        for (value_id, value_info) in &ctx.value_pool.values {
            if seen_dead.contains(value_id) {
                continue;
            }
            let desc = self.describe_value(ctx, *value_id);
            let live_range = LiveRange {
                value: *value_id,
                description: desc.clone(),
                born_at: position,
                last_use: position,
                drop_at: None,
                dropped_by_scope_exit: false,
            };

            self.live_ranges.push(live_range);
            position += 1;

            if value_info.ref_count == 0
                && !matches!(value_info.kind, ValueKind::Reference { .. })
            {
                self.drop_points.push(DropPoint {
                    value: *value_id,
                    description: desc,
                    at_position: position,
                });
            }
        }

        debug!(
            "[Liveness] {} live ranges, {} drop points ({} from scope exit)",
            self.live_ranges.len(),
            self.drop_points.len(),
            dead_count,
        );
    }

    fn describe_value(&self, ctx: &GatherContext, value_id: WindValueID) -> String {
        if let Some(name) = ctx.value_names.get(&value_id) {
            if let Some(info) = ctx.value_pool.get(value_id) {
                return format!("'{}' {:?} (id:{})", name, info.kind, value_id.get());
            }
            return format!("'{}' (id:{})", name, value_id.get());
        }
        let names = ctx.bindings.get_names_for_value(value_id);
        if !names.is_empty() {
            let name_str: Vec<String> = names.iter().map(|n| n.var_name.clone()).collect();
            if let Some(info) = ctx.value_pool.get(value_id) {
                return format!(
                    "'{}' {:?} (id:{})",
                    name_str.join(", "),
                    info.kind,
                    value_id.get()
                );
            }
            return format!("'{}' (id:{})", name_str.join(", "), value_id.get());
        }
        if let Some(info) = ctx.value_pool.get(value_id) {
            return format!(
                "{:?} (id:{})",
                info.kind,
                value_id.get()
            );
        }
        format!("<unknown> (id:{})", value_id.get())
    }
}
