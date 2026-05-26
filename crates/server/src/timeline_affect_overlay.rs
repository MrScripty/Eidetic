use std::collections::HashMap;

use eidetic_core::contracts::{AffectTarget, TimelineRenderAffectSample, TimelineRenderProjection};
use eidetic_core::timeline::Timeline;
use eidetic_core::timeline::node::NodeId;
use rusqlite::Connection;

use crate::affect_store;
use crate::history_store::HistoryStoreError;

pub(crate) fn apply_timeline_affect_overlays(
    conn: &Connection,
    timeline: &Timeline,
    projection: &mut TimelineRenderProjection,
) -> Result<(), HistoryStoreError> {
    let ranges_by_node_id = timeline
        .nodes
        .iter()
        .map(|node| (node.id, (node.time_range.start_ms, node.time_range.end_ms)))
        .collect::<HashMap<NodeId, (u64, u64)>>();
    let mut overlays = Vec::new();
    for value in affect_store::load_timeline_node_affect_values(conn)? {
        let AffectTarget::TimelineNode { node_id } = value.target else {
            continue;
        };
        let Some((start_ms, end_ms)) = ranges_by_node_id.get(&node_id).copied() else {
            continue;
        };
        overlays.push(TimelineRenderAffectSample {
            affect_id: value.id,
            node_id,
            start_ms,
            end_ms,
            valence: value.valence,
            arousal: value.arousal,
            intensity: value.intensity,
            confidence: value.confidence,
            mood_labels: value.mood_labels,
            provenance: value.provenance,
        });
    }
    overlays.sort_by_key(|overlay| (overlay.start_ms, overlay.node_id.0, overlay.affect_id.0));
    projection.affect_overlays = overlays;
    Ok(())
}

#[cfg(test)]
mod tests {
    use eidetic_core::Template;
    use eidetic_core::contracts::{
        AffectConfidence, AffectProvenance, AffectTarget, AffectValueId, Arousal, CommandEnvelope,
        CommandId, EmotionalIntensity, MoodLabel, SetAffectValueCommand, TimelineRenderProjection,
        Valence,
    };

    use super::*;

    #[test]
    fn timeline_affect_overlays_follow_timeline_clip_ranges() {
        let mut conn = Connection::open_in_memory().unwrap();
        let project = Template::MultiCam.build_project("Timeline Affect Test");
        let node = project.timeline.nodes[0].clone();
        affect_store::record_set_affect_value(
            &mut conn,
            &CommandEnvelope::new(SetAffectValueCommand {
                command_id: CommandId::new(),
                affect_id: AffectValueId::new(),
                target: AffectTarget::TimelineNode { node_id: node.id },
                valence: Valence::new(-250).unwrap(),
                arousal: Arousal::new(650).unwrap(),
                intensity: EmotionalIntensity::new(700).unwrap(),
                confidence: AffectConfidence::new(900).unwrap(),
                mood_labels: vec![MoodLabel::new("uneasy").unwrap()],
                provenance: AffectProvenance::UserAuthored,
                rationale: Some("Opening mood".to_string()),
            }),
            100,
        )
        .unwrap();
        let mut projection = TimelineRenderProjection::from_timeline(&project.timeline);

        apply_timeline_affect_overlays(&conn, &project.timeline, &mut projection).unwrap();

        assert_eq!(projection.affect_overlays.len(), 1);
        assert_eq!(projection.affect_overlays[0].node_id, node.id);
        assert_eq!(
            projection.affect_overlays[0].start_ms,
            node.time_range.start_ms
        );
        assert_eq!(projection.affect_overlays[0].end_ms, node.time_range.end_ms);
        assert_eq!(projection.affect_overlays[0].valence.basis_points(), -250);
    }
}
