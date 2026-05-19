use eidetic_core::contracts::PropagationProposalTarget;

use crate::history_store::HistoryStoreError;

#[derive(Debug)]
pub(crate) struct SqlPropagationTarget {
    pub(crate) kind: String,
    pub(crate) id: String,
    pub(crate) part_key: Option<String>,
    pub(crate) field_key: Option<String>,
    pub(crate) field_id: Option<String>,
    pub(crate) snapshot_id: Option<String>,
}

impl SqlPropagationTarget {
    pub(crate) fn from_target(target: &PropagationProposalTarget) -> Self {
        match target {
            PropagationProposalTarget::BibleField {
                node_id,
                part_key,
                field_key,
                field_id,
            } => Self {
                kind: "bible_field".to_string(),
                id: node_id.as_str().to_string(),
                part_key: Some(part_key.as_str().to_string()),
                field_key: Some(field_key.as_str().to_string()),
                field_id: field_id.as_ref().map(|id| id.as_str().to_string()),
                snapshot_id: None,
            },
            PropagationProposalTarget::BibleSnapshotField {
                node_id,
                snapshot_id,
                field_key,
            } => Self {
                kind: "bible_snapshot_field".to_string(),
                id: node_id.as_str().to_string(),
                part_key: None,
                field_key: Some(field_key.as_str().to_string()),
                field_id: None,
                snapshot_id: Some(snapshot_id.as_str().to_string()),
            },
            PropagationProposalTarget::ScriptBlock { block_id } => Self {
                kind: "script_block".to_string(),
                id: block_id.as_str().to_string(),
                part_key: None,
                field_key: None,
                field_id: None,
                snapshot_id: None,
            },
            PropagationProposalTarget::ScriptSegment { segment_id } => Self {
                kind: "script_segment".to_string(),
                id: segment_id.as_str().to_string(),
                part_key: None,
                field_key: None,
                field_id: None,
                snapshot_id: None,
            },
        }
    }

    pub(crate) fn into_target(
        self,
    ) -> Result<PropagationProposalTarget, PropagationProposalTargetError> {
        match self.kind.as_str() {
            "bible_field" => Ok(PropagationProposalTarget::BibleField {
                node_id: eidetic_core::contracts::BibleGraphNodeId::new(self.id)?,
                part_key: eidetic_core::contracts::BibleGraphPartKey::new(required(
                    self.part_key,
                    "part_key",
                )?)?,
                field_key: eidetic_core::contracts::BibleGraphFieldKey::new(required(
                    self.field_key,
                    "field_key",
                )?)?,
                field_id: self
                    .field_id
                    .map(eidetic_core::contracts::BibleGraphFieldId::new)
                    .transpose()?,
            }),
            "bible_snapshot_field" => Ok(PropagationProposalTarget::BibleSnapshotField {
                node_id: eidetic_core::contracts::BibleGraphNodeId::new(self.id)?,
                snapshot_id: eidetic_core::contracts::BibleGraphSnapshotId::new(required(
                    self.snapshot_id,
                    "snapshot_id",
                )?)?,
                field_key: eidetic_core::contracts::BibleGraphFieldKey::new(required(
                    self.field_key,
                    "field_key",
                )?)?,
            }),
            "script_block" => Ok(PropagationProposalTarget::ScriptBlock {
                block_id: eidetic_core::contracts::ScriptBlockId::new(self.id)?,
            }),
            "script_segment" => Ok(PropagationProposalTarget::ScriptSegment {
                segment_id: eidetic_core::contracts::ScriptSegmentId::new(self.id)?,
            }),
            other => Err(PropagationProposalTargetError::InvalidTarget(format!(
                "unknown propagation proposal target kind: {other}"
            ))),
        }
    }
}

pub(crate) fn target_label(target: &PropagationProposalTarget) -> String {
    let target = SqlPropagationTarget::from_target(target);
    match (target.snapshot_id, target.part_key, target.field_key) {
        (Some(snapshot_id), _, Some(field_key)) => {
            format!("{}:{}.{snapshot_id}.{field_key}", target.kind, target.id)
        }
        (_, Some(part_key), Some(field_key)) => {
            format!("{}:{}.{part_key}.{field_key}", target.kind, target.id)
        }
        _ => format!("{}:{}", target.kind, target.id),
    }
}

fn required<T>(value: Option<T>, field_name: &'static str) -> Result<T, HistoryStoreError> {
    value.ok_or(HistoryStoreError::MissingColumn(field_name))
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PropagationProposalTargetError {
    #[error("{0}")]
    InvalidTarget(String),
    #[error(transparent)]
    History(#[from] HistoryStoreError),
    #[error(transparent)]
    BibleGraphContract(#[from] eidetic_core::contracts::BibleGraphContractError),
    #[error(transparent)]
    ScriptContract(#[from] eidetic_core::contracts::ScriptContractError),
}
