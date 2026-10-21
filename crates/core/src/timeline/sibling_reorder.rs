//! Atomic adjacent sibling order and placement, preserving authored content.
use super::{Timeline, node::NodeId, timing::TimeRange};
use crate::{Error, Result};

/// Swap an earlier/later pair without changing durations, gap or outer bounds.
pub fn swapped_adjacent_ranges(
    first: TimeRange,
    second: TimeRange,
) -> Result<(TimeRange, TimeRange)> {
    first.validate()?;
    second.validate()?;
    if first.end_ms > second.start_ms {
        return Err(Error::InvalidOperation(
            "Reorder requires non-overlapping adjacent clips".into(),
        ));
    }
    let second_end = first
        .start_ms
        .checked_add(second.duration_ms())
        .ok_or_else(|| Error::InvalidOperation("Reorder range overflow".into()))?;
    let first_start = second_end
        .checked_add(second.start_ms - first.end_ms)
        .ok_or_else(|| Error::InvalidOperation("Reorder range overflow".into()))?;
    Ok((
        TimeRange::new(first_start, second.end_ms)?,
        TimeRange::new(first.start_ms, second_end)?,
    ))
}

impl Timeline {
    /// Reorder one clip with its adjacent sibling. All validation precedes mutation.
    /// Content locks retain their AI meaning and do not prohibit structural moves.
    pub fn reorder_adjacent_nodes(&mut self, node_id: NodeId, neighbor_id: NodeId) -> Result<()> {
        let node = self.node(node_id)?.clone();
        let neighbor = self.node(neighbor_id)?.clone();
        if node_id == neighbor_id
            || node.parent_id.is_none()
            || node.parent_id != neighbor.parent_id
            || node.level != neighbor.level
        {
            return Err(Error::InvalidOperation(
                "Reorder requires two distinct clips on the same parent and story-level track"
                    .into(),
            ));
        }
        let mut siblings = self
            .nodes
            .iter()
            .filter(|candidate| {
                candidate.parent_id == node.parent_id && candidate.level == node.level
            })
            .collect::<Vec<_>>();
        siblings.sort_by_key(|candidate| (candidate.time_range.start_ms, candidate.id.0));
        let index = siblings
            .iter()
            .position(|candidate| candidate.id == node_id)
            .expect("node is sibling");
        let neighbor_index = siblings
            .iter()
            .position(|candidate| candidate.id == neighbor_id)
            .expect("neighbor is sibling");
        if index.abs_diff(neighbor_index) != 1 {
            return Err(Error::InvalidOperation(
                "Selected clips are no longer adjacent".into(),
            ));
        }
        let (first, second) = if index < neighbor_index {
            (&node, &neighbor)
        } else {
            (&neighbor, &node)
        };
        let outer = TimeRange::new(first.time_range.start_ms, second.time_range.end_ms)?;
        if siblings.iter().any(|candidate| {
            candidate.id != node_id
                && candidate.id != neighbor_id
                && candidate.time_range.overlaps(&outer)
        }) {
            return Err(Error::InvalidOperation(
                "Reorder cannot cross overlapping sibling clips".into(),
            ));
        }
        siblings.sort_by_key(|candidate| {
            (
                candidate.sort_order,
                candidate.time_range.start_ms,
                candidate.id.0,
            )
        });
        let first_index = siblings
            .iter()
            .position(|candidate| candidate.id == first.id)
            .expect("first sibling");
        if first.sort_order > second.sort_order
            || siblings
                .get(first_index + 1)
                .is_none_or(|candidate| candidate.id != second.id)
        {
            return Err(Error::InvalidOperation("Screen placement and sibling order disagree; review their placement before reordering".into()));
        }
        let parent = self.node(node.parent_id.expect("parent checked"))?;
        if outer.start_ms < parent.time_range.start_ms
            || outer.end_ms > parent.time_range.end_ms
            || outer.end_ms > self.total_duration_ms
        {
            return Err(Error::InvalidOperation(
                "Reorder must remain inside the parent and timeline".into(),
            ));
        }
        let (first_range, second_range) =
            swapped_adjacent_ranges(first.time_range, second.time_range)?;
        let mut plan = Vec::new();
        for (root, range) in [(first, first_range), (second, second_range)] {
            let offset = i128::from(range.start_ms) - i128::from(root.time_range.start_ms);
            plan.push((root.id, range));
            for child in self.descendants_of(root.id) {
                child.time_range.validate()?;
                let owner = self.node(child.parent_id.expect("descendant parent"))?;
                if child.time_range.start_ms < owner.time_range.start_ms
                    || child.time_range.end_ms > owner.time_range.end_ms
                {
                    return Err(Error::InvalidOperation(
                        "Reorder requires every child to remain contained by its parent".into(),
                    ));
                }
                let shifted = |value: u64| {
                    u64::try_from(i128::from(value) + offset).map_err(|_| {
                        Error::InvalidOperation("Reorder descendant range overflow".into())
                    })
                };
                plan.push((
                    child.id,
                    TimeRange::new(
                        shifted(child.time_range.start_ms)?,
                        shifted(child.time_range.end_ms)?,
                    )?,
                ));
            }
        }
        for (id, range) in plan {
            self.node_mut(id)?.time_range = range;
        }
        self.node_mut(first.id)?.sort_order = second.sort_order;
        self.node_mut(second.id)?.sort_order = first.sort_order;
        Ok(())
    }
}

#[cfg(test)]
#[path = "sibling_reorder_tests.rs"]
mod tests;
