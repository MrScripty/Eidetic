use serde::{Deserialize, Serialize};

use super::FieldValue;

macro_rules! non_empty_string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, BibleGraphContractError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(BibleGraphContractError::EmptyIdentifier(stringify!($name)));
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = BibleGraphContractError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

non_empty_string_id!(BibleGraphNodeId);
non_empty_string_id!(BibleGraphPartId);
non_empty_string_id!(BibleGraphFieldId);
non_empty_string_id!(BibleGraphEdgeId);
non_empty_string_id!(BibleGraphSnapshotId);
non_empty_string_id!(BibleGraphSnapshotFieldId);
non_empty_string_id!(BibleGraphSchemaKey);
non_empty_string_id!(BibleGraphPartKey);
non_empty_string_id!(BibleGraphFieldKey);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalBibleRoot {
    Characters,
    Places,
    Objects,
    Cultures,
    Events,
    Themes,
    Rules,
    References,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BibleGraphNodeCategory {
    Character,
    Location,
    Prop,
    Culture,
    Theme,
    Event,
    Rule,
    Reference,
    Canonical,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleGraphCategoryVisualStyle {
    pub fill_color: String,
}

impl BibleGraphNodeCategory {
    pub fn for_node(node: &BibleGraphNode) -> Self {
        Self::for_schema_and_parent(node.schema_key.as_str(), node.parent_id.as_ref())
    }

    pub fn for_schema_and_parent(schema_key: &str, parent_id: Option<&BibleGraphNodeId>) -> Self {
        match schema_key {
            "canonical.root.characters" | "character" => Self::Character,
            "canonical.root.places" | "location" | "place" => Self::Location,
            "canonical.root.objects" | "object" | "prop" => Self::Prop,
            "canonical.root.cultures" | "culture" => Self::Culture,
            "canonical.root.themes" | "theme" => Self::Theme,
            "canonical.root.events" | "event" => Self::Event,
            "canonical.root.rules" | "rule" => Self::Rule,
            "canonical.root.references" | "reference" => Self::Reference,
            schema if schema.starts_with("canonical.") => Self::Canonical,
            _ => parent_id.map(Self::for_parent_id).unwrap_or(Self::Other),
        }
    }

    fn for_parent_id(parent_id: &BibleGraphNodeId) -> Self {
        match parent_id.as_str() {
            "canonical.characters" => Self::Character,
            "canonical.places" => Self::Location,
            "canonical.objects" => Self::Prop,
            "canonical.cultures" => Self::Culture,
            "canonical.themes" => Self::Theme,
            "canonical.events" => Self::Event,
            "canonical.rules" => Self::Rule,
            "canonical.references" => Self::Reference,
            _ => Self::Other,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Character => "Character",
            Self::Location => "Location",
            Self::Prop => "Prop",
            Self::Culture => "Culture",
            Self::Theme => "Theme",
            Self::Event => "Event",
            Self::Rule => "Rule",
            Self::Reference => "Reference",
            Self::Canonical => "Canonical",
            Self::Other => "Other",
        }
    }

    pub fn fill_color(&self) -> &'static str {
        match self {
            Self::Character => "#6495ed",
            Self::Location => "#22c55e",
            Self::Prop => "#f97316",
            Self::Culture => "#14b8a6",
            Self::Theme => "#a855f7",
            Self::Event => "#ef4444",
            Self::Rule => "#eab308",
            Self::Reference => "#38bdf8",
            Self::Canonical => "#536f88",
            Self::Other => "#34495e",
        }
    }

    pub fn visual_style(&self) -> BibleGraphCategoryVisualStyle {
        BibleGraphCategoryVisualStyle {
            fill_color: self.fill_color().to_string(),
        }
    }
}

impl CanonicalBibleRoot {
    pub fn node_id(&self) -> BibleGraphNodeId {
        BibleGraphNodeId::new(match self {
            Self::Characters => "canonical.characters",
            Self::Places => "canonical.places",
            Self::Objects => "canonical.objects",
            Self::Cultures => "canonical.cultures",
            Self::Events => "canonical.events",
            Self::Themes => "canonical.themes",
            Self::Rules => "canonical.rules",
            Self::References => "canonical.references",
        })
        .expect("canonical root identifiers are non-empty")
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Characters => "Characters",
            Self::Places => "Places",
            Self::Objects => "Objects",
            Self::Cultures => "Cultures",
            Self::Events => "Events",
            Self::Themes => "Themes",
            Self::Rules => "Rules",
            Self::References => "References",
        }
    }

    pub fn schema_key(&self) -> BibleGraphSchemaKey {
        BibleGraphSchemaKey::new(match self {
            Self::Characters => "canonical.root.characters",
            Self::Places => "canonical.root.places",
            Self::Objects => "canonical.root.objects",
            Self::Cultures => "canonical.root.cultures",
            Self::Events => "canonical.root.events",
            Self::Themes => "canonical.root.themes",
            Self::Rules => "canonical.root.rules",
            Self::References => "canonical.root.references",
        })
        .expect("canonical root schema keys are non-empty")
    }

    pub fn category(&self) -> BibleGraphNodeCategory {
        match self {
            Self::Characters => BibleGraphNodeCategory::Character,
            Self::Places => BibleGraphNodeCategory::Location,
            Self::Objects => BibleGraphNodeCategory::Prop,
            Self::Cultures => BibleGraphNodeCategory::Culture,
            Self::Events => BibleGraphNodeCategory::Event,
            Self::Themes => BibleGraphNodeCategory::Theme,
            Self::Rules => BibleGraphNodeCategory::Rule,
            Self::References => BibleGraphNodeCategory::Reference,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleGraphNode {
    pub id: BibleGraphNodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<BibleGraphNodeId>,
    pub schema_key: BibleGraphSchemaKey,
    pub name: String,
    #[serde(default)]
    pub system_owned: bool,
    #[serde(default)]
    pub sort_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateBibleGraphNodeCommand {
    pub node_id: BibleGraphNodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<BibleGraphNodeId>,
    pub schema_key: BibleGraphSchemaKey,
    pub name: String,
    #[serde(default)]
    pub sort_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteBibleGraphNodeCommand {
    pub node_id: BibleGraphNodeId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnsureCanonicalBibleRootsCommand {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetBibleGraphFieldCommand {
    pub node_id: BibleGraphNodeId,
    pub part_id: BibleGraphPartId,
    pub part_key: BibleGraphPartKey,
    pub part_name: String,
    #[serde(default)]
    pub part_sort_order: u32,
    pub field_id: BibleGraphFieldId,
    pub field_key: BibleGraphFieldKey,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<FieldValue>,
    #[serde(default)]
    pub field_sort_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetBibleGraphEdgeCommand {
    pub edge_id: BibleGraphEdgeId,
    pub from_node_id: BibleGraphNodeId,
    pub to_node_id: BibleGraphNodeId,
    pub edge_kind: BibleGraphEdgeKind,
    pub label: String,
    #[serde(default = "default_directed")]
    pub directed: bool,
    #[serde(default)]
    pub sort_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteBibleGraphEdgeCommand {
    pub edge_id: BibleGraphEdgeId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetBibleGraphSnapshotFieldCommand {
    pub snapshot_id: BibleGraphSnapshotId,
    pub node_id: BibleGraphNodeId,
    pub at_ms: u64,
    pub label: String,
    #[serde(default)]
    pub snapshot_sort_order: u32,
    pub field_id: BibleGraphSnapshotFieldId,
    pub part_key: BibleGraphPartKey,
    pub part_name: String,
    pub field_key: BibleGraphFieldKey,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<FieldValue>,
    #[serde(default)]
    pub field_sort_order: u32,
}

impl CreateBibleGraphNodeCommand {
    pub fn into_node(self) -> BibleGraphNode {
        BibleGraphNode {
            id: self.node_id,
            parent_id: self.parent_id,
            schema_key: self.schema_key,
            name: self.name,
            system_owned: false,
            sort_order: self.sort_order,
        }
    }
}

impl SetBibleGraphEdgeCommand {
    pub fn into_edge(self) -> BibleGraphEdge {
        BibleGraphEdge {
            id: self.edge_id,
            from_node_id: self.from_node_id,
            to_node_id: self.to_node_id,
            edge_kind: self.edge_kind,
            label: self.label,
            directed: self.directed,
            sort_order: self.sort_order,
        }
    }
}

impl SetBibleGraphSnapshotFieldCommand {
    pub fn to_snapshot(&self) -> BibleGraphSnapshot {
        BibleGraphSnapshot {
            id: self.snapshot_id.clone(),
            node_id: self.node_id.clone(),
            at_ms: self.at_ms,
            label: self.label.clone(),
            sort_order: self.snapshot_sort_order,
        }
    }

    pub fn into_field(self) -> BibleGraphSnapshotField {
        BibleGraphSnapshotField {
            id: self.field_id,
            snapshot_id: self.snapshot_id,
            part_key: self.part_key,
            part_name: self.part_name,
            field_key: self.field_key,
            value: self.value,
            sort_order: self.field_sort_order,
        }
    }
}

impl BibleGraphNode {
    pub fn canonical_root(root: CanonicalBibleRoot, sort_order: u32) -> Self {
        Self {
            id: root.node_id(),
            parent_id: None,
            schema_key: root.schema_key(),
            name: root.display_name().to_string(),
            system_owned: true,
            sort_order,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleGraphPart {
    pub id: BibleGraphPartId,
    pub node_id: BibleGraphNodeId,
    pub part_key: BibleGraphPartKey,
    pub name: String,
    #[serde(default)]
    pub system_owned: bool,
    #[serde(default)]
    pub sort_order: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleGraphField {
    pub id: BibleGraphFieldId,
    pub part_id: BibleGraphPartId,
    pub field_key: BibleGraphFieldKey,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<FieldValue>,
    #[serde(default)]
    pub sort_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BibleGraphEdgeKind {
    References,
    LocatedIn,
    Owns,
    MemberOf,
    ConflictsWith,
    SupportsTheme,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleGraphEdge {
    pub id: BibleGraphEdgeId,
    pub from_node_id: BibleGraphNodeId,
    pub to_node_id: BibleGraphNodeId,
    pub edge_kind: BibleGraphEdgeKind,
    pub label: String,
    #[serde(default = "default_directed")]
    pub directed: bool,
    #[serde(default)]
    pub sort_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleGraphSnapshot {
    pub id: BibleGraphSnapshotId,
    pub node_id: BibleGraphNodeId,
    pub at_ms: u64,
    pub label: String,
    #[serde(default)]
    pub sort_order: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleGraphSnapshotField {
    pub id: BibleGraphSnapshotFieldId,
    pub snapshot_id: BibleGraphSnapshotId,
    pub part_key: BibleGraphPartKey,
    pub part_name: String,
    pub field_key: BibleGraphFieldKey,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<FieldValue>,
    #[serde(default)]
    pub sort_order: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleGraphPartProjection {
    pub part: BibleGraphPart,
    #[serde(default)]
    pub fields: Vec<BibleGraphField>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleGraphSnapshotProjection {
    pub snapshot: BibleGraphSnapshot,
    #[serde(default)]
    pub fields: Vec<BibleGraphSnapshotField>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleNodeDetailProjection {
    pub node: BibleGraphNode,
    #[serde(default)]
    pub parts: Vec<BibleGraphPartProjection>,
    #[serde(default)]
    pub incoming_edges: Vec<BibleGraphEdge>,
    #[serde(default)]
    pub outgoing_edges: Vec<BibleGraphEdge>,
    #[serde(default)]
    pub snapshots: Vec<BibleGraphSnapshotProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleGraphNodeListProjection {
    #[serde(default)]
    pub nodes: Vec<BibleGraphNode>,
}

pub fn canonical_bible_root_nodes() -> Vec<BibleGraphNode> {
    [
        CanonicalBibleRoot::Characters,
        CanonicalBibleRoot::Places,
        CanonicalBibleRoot::Objects,
        CanonicalBibleRoot::Cultures,
        CanonicalBibleRoot::Events,
        CanonicalBibleRoot::Themes,
        CanonicalBibleRoot::Rules,
        CanonicalBibleRoot::References,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, root)| BibleGraphNode::canonical_root(root, index as u32))
    .collect()
}

fn default_directed() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BibleGraphContractError {
    #[error("{0} must not be empty")]
    EmptyIdentifier(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_roots_are_system_owned_and_stable() {
        let roots = canonical_bible_root_nodes();

        assert_eq!(roots.len(), 8);
        assert_eq!(roots[0].id.as_str(), "canonical.characters");
        assert!(roots.iter().all(|root| root.parent_id.is_none()));
        assert!(roots.iter().all(|root| root.system_owned));
        assert_eq!(roots[7].sort_order, 7);
    }

    #[test]
    fn identifiers_reject_empty_values() {
        let error = BibleGraphNodeId::new("  ").unwrap_err();

        assert_eq!(
            error,
            BibleGraphContractError::EmptyIdentifier("BibleGraphNodeId")
        );
    }

    #[test]
    fn identifiers_reject_empty_deserialized_values() {
        let error = serde_json::from_str::<BibleGraphNodeId>(r#""  ""#).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("BibleGraphNodeId must not be empty")
        );
    }

    #[test]
    fn detail_projection_round_trips_without_json_canonical_fields() {
        let projection = BibleNodeDetailProjection {
            node: BibleGraphNode {
                id: BibleGraphNodeId::new("node.location.beach").unwrap(),
                parent_id: Some(CanonicalBibleRoot::Places.node_id()),
                schema_key: BibleGraphSchemaKey::new("location").unwrap(),
                name: "Beach".to_string(),
                system_owned: false,
                sort_order: 12,
            },
            parts: vec![BibleGraphPartProjection {
                part: BibleGraphPart {
                    id: BibleGraphPartId::new("part.location.weather").unwrap(),
                    node_id: BibleGraphNodeId::new("node.location.beach").unwrap(),
                    part_key: BibleGraphPartKey::new("weather").unwrap(),
                    name: "Weather".to_string(),
                    system_owned: false,
                    sort_order: 1,
                },
                fields: vec![BibleGraphField {
                    id: BibleGraphFieldId::new("field.location.weather.current").unwrap(),
                    part_id: BibleGraphPartId::new("part.location.weather").unwrap(),
                    field_key: BibleGraphFieldKey::new("current").unwrap(),
                    value: Some(FieldValue::Text("rainy".to_string())),
                    sort_order: 0,
                }],
            }],
            incoming_edges: Vec::new(),
            outgoing_edges: vec![BibleGraphEdge {
                id: BibleGraphEdgeId::new("edge.beach.theme").unwrap(),
                from_node_id: BibleGraphNodeId::new("node.location.beach").unwrap(),
                to_node_id: BibleGraphNodeId::new("node.theme.isolation").unwrap(),
                edge_kind: BibleGraphEdgeKind::SupportsTheme,
                label: "reinforces".to_string(),
                directed: true,
                sort_order: 0,
            }],
            snapshots: vec![BibleGraphSnapshotProjection {
                snapshot: BibleGraphSnapshot {
                    id: BibleGraphSnapshotId::new("snapshot.beach.sequence-1").unwrap(),
                    node_id: BibleGraphNodeId::new("node.location.beach").unwrap(),
                    at_ms: 12_000,
                    label: "Sequence 1 state".to_string(),
                    sort_order: 0,
                },
                fields: vec![BibleGraphSnapshotField {
                    id: BibleGraphSnapshotFieldId::new("snapshot-field.beach.weather.current")
                        .unwrap(),
                    snapshot_id: BibleGraphSnapshotId::new("snapshot.beach.sequence-1").unwrap(),
                    part_key: BibleGraphPartKey::new("weather").unwrap(),
                    part_name: "Weather".to_string(),
                    field_key: BibleGraphFieldKey::new("current").unwrap(),
                    value: Some(FieldValue::Text("rainy".to_string())),
                    sort_order: 0,
                }],
            }],
        };

        let json = serde_json::to_string(&projection).unwrap();
        let round_trip: BibleNodeDetailProjection = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, projection);
    }

    #[test]
    fn create_node_command_round_trips_with_validated_ids() {
        let command = CreateBibleGraphNodeCommand {
            node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            parent_id: Some(CanonicalBibleRoot::Characters.node_id()),
            schema_key: BibleGraphSchemaKey::new("character").unwrap(),
            name: "Ada".to_string(),
            sort_order: 5,
        };

        let json = serde_json::to_string(&command).unwrap();
        let round_trip: CreateBibleGraphNodeCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, command);
    }

    #[test]
    fn ensure_canonical_roots_command_round_trips() {
        let command = EnsureCanonicalBibleRootsCommand {};

        let json = serde_json::to_string(&command).unwrap();
        let round_trip: EnsureCanonicalBibleRootsCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, command);
    }

    #[test]
    fn set_field_command_round_trips_with_typed_value() {
        let command = SetBibleGraphFieldCommand {
            node_id: BibleGraphNodeId::new("node.place.beach").unwrap(),
            part_id: BibleGraphPartId::new("part.place.weather").unwrap(),
            part_key: BibleGraphPartKey::new("weather").unwrap(),
            part_name: "Weather".to_string(),
            part_sort_order: 1,
            field_id: BibleGraphFieldId::new("field.place.weather.current").unwrap(),
            field_key: BibleGraphFieldKey::new("current").unwrap(),
            value: Some(FieldValue::Text("rainy".to_string())),
            field_sort_order: 2,
        };

        let json = serde_json::to_string(&command).unwrap();
        let round_trip: SetBibleGraphFieldCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, command);
    }

    #[test]
    fn set_snapshot_field_command_round_trips_with_typed_value() {
        let command = SetBibleGraphSnapshotFieldCommand {
            snapshot_id: BibleGraphSnapshotId::new("snapshot.place.beach.sequence-1").unwrap(),
            node_id: BibleGraphNodeId::new("node.place.beach").unwrap(),
            at_ms: 12_000,
            label: "Sequence 1 state".to_string(),
            snapshot_sort_order: 3,
            field_id: BibleGraphSnapshotFieldId::new("snapshot-field.place.beach.weather.current")
                .unwrap(),
            part_key: BibleGraphPartKey::new("weather").unwrap(),
            part_name: "Weather".to_string(),
            field_key: BibleGraphFieldKey::new("current").unwrap(),
            value: Some(FieldValue::Text("rainy".to_string())),
            field_sort_order: 2,
        };

        let json = serde_json::to_string(&command).unwrap();
        let round_trip: SetBibleGraphSnapshotFieldCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, command);
    }

    #[test]
    fn set_edge_command_round_trips() {
        let command = SetBibleGraphEdgeCommand {
            edge_id: BibleGraphEdgeId::new("edge.ada.beach").unwrap(),
            from_node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            to_node_id: BibleGraphNodeId::new("node.place.beach").unwrap(),
            edge_kind: BibleGraphEdgeKind::LocatedIn,
            label: "located in".to_string(),
            directed: true,
            sort_order: 4,
        };

        let json = serde_json::to_string(&command).unwrap();
        let round_trip: SetBibleGraphEdgeCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, command);
    }

    #[test]
    fn delete_edge_command_round_trips() {
        let command = DeleteBibleGraphEdgeCommand {
            edge_id: BibleGraphEdgeId::new("edge.ada.beach").unwrap(),
        };

        let json = serde_json::to_string(&command).unwrap();
        let round_trip: DeleteBibleGraphEdgeCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, command);
    }

    #[test]
    fn delete_node_command_round_trips() {
        let command = DeleteBibleGraphNodeCommand {
            node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
        };

        let json = serde_json::to_string(&command).unwrap();
        let round_trip: DeleteBibleGraphNodeCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, command);
    }

    #[test]
    fn node_list_projection_round_trips() {
        let projection = BibleGraphNodeListProjection {
            nodes: canonical_bible_root_nodes(),
        };

        let json = serde_json::to_string(&projection).unwrap();
        let round_trip: BibleGraphNodeListProjection = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, projection);
    }
}
