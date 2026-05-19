use serde::{Deserialize, Serialize};

use super::{ChangeEvent, ObjectRevision};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeReviewProjection {
    #[serde(default)]
    pub changes: Vec<ChangeReviewChange>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeReviewChange {
    pub event: ChangeEvent,
    #[serde(default)]
    pub revisions: Vec<ObjectRevision>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{ChangeEventKind, CommandId};

    #[test]
    fn change_review_projection_round_trips_empty_history() {
        let projection = ChangeReviewProjection {
            changes: vec![ChangeReviewChange {
                event: ChangeEvent::new(
                    CommandId::new(),
                    ChangeEventKind::AiProposalAccepted,
                    "accept proposal",
                ),
                revisions: Vec::new(),
            }],
        };

        let encoded = serde_json::to_string(&projection).unwrap();
        let decoded: ChangeReviewProjection = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, projection);
    }
}
