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
non_empty_string_id!(BibleGraphSchemaKey);
non_empty_string_id!(BibleGraphPartKey);
non_empty_string_id!(BibleGraphFieldKey);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleGraphPartProjection {
    pub part: BibleGraphPart,
    #[serde(default)]
    pub fields: Vec<BibleGraphField>,
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
}
