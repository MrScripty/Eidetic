use super::{
    BibleGraphField, BibleGraphFieldId, BibleGraphFieldKey, BibleGraphNode, BibleGraphNodeCategory,
    BibleGraphNodeId, BibleGraphPart, BibleGraphPartId, BibleGraphPartKey,
    BibleGraphPartProjection, BibleGraphSchemaKey, CanonicalBibleRoot, ProjectionEnvelope,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BibleGraphSchemaDefault {
    pub schema_key: &'static str,
    pub category: BibleGraphNodeCategory,
    pub display_name: &'static str,
    pub default_node_name: &'static str,
    pub canonical_parent_id: Option<&'static str>,
    pub canonical_root_schema_key: Option<&'static str>,
    pub parts: &'static [BibleGraphPartDefault],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BibleGraphPartDefault {
    pub part_key: &'static str,
    pub name: &'static str,
    pub sort_order: u32,
    pub fields: &'static [BibleGraphFieldDefault],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BibleGraphFieldDefault {
    pub field_key: &'static str,
    pub sort_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleGraphSchemaListProjection {
    pub categories: Vec<BibleGraphCategoryProjection>,
    pub schemas: Vec<BibleGraphSchemaProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleGraphCategoryProjection {
    pub category: BibleGraphNodeCategory,
    pub display_name: String,
    pub visual_style: super::BibleGraphCategoryVisualStyle,
    pub root_node_id: BibleGraphNodeId,
    pub root_schema_key: BibleGraphSchemaKey,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_schema_key: Option<BibleGraphSchemaKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_node_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleGraphSchemaProjection {
    pub schema_key: BibleGraphSchemaKey,
    pub category: BibleGraphNodeCategory,
    pub display_name: String,
    pub visual_style: super::BibleGraphCategoryVisualStyle,
    pub default_node_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_parent_id: Option<BibleGraphNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_root_schema_key: Option<BibleGraphSchemaKey>,
    pub parts: Vec<BibleGraphPartSchemaProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleGraphPartSchemaProjection {
    pub part_key: BibleGraphPartKey,
    pub name: String,
    pub sort_order: u32,
    pub fields: Vec<BibleGraphFieldSchemaProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleGraphFieldSchemaProjection {
    pub field_key: BibleGraphFieldKey,
    pub sort_order: u32,
}

impl BibleGraphSchemaDefault {
    pub fn part(&self, part_key: &BibleGraphPartKey) -> Option<&'static BibleGraphPartDefault> {
        self.parts
            .iter()
            .find(|part| part.part_key == part_key.as_str())
    }
}

impl BibleGraphPartDefault {
    pub fn field(&self, field_key: &BibleGraphFieldKey) -> Option<&'static BibleGraphFieldDefault> {
        self.fields
            .iter()
            .find(|field| field.field_key == field_key.as_str())
    }
}

pub fn builtin_bible_graph_schema(
    schema_key: &BibleGraphSchemaKey,
) -> Option<&'static BibleGraphSchemaDefault> {
    BUILTIN_BIBLE_GRAPH_SCHEMAS
        .iter()
        .find(|schema| schema.schema_key == schema_key.as_str())
}

pub fn default_part_projections_for_node(node: &BibleGraphNode) -> Vec<BibleGraphPartProjection> {
    let Some(schema) = builtin_bible_graph_schema(&node.schema_key) else {
        return Vec::new();
    };

    schema
        .parts
        .iter()
        .map(|part| BibleGraphPartProjection {
            part: BibleGraphPart {
                id: BibleGraphPartId::new(default_part_id(node, part))
                    .expect("default part identifiers are non-empty"),
                node_id: node.id.clone(),
                part_key: BibleGraphPartKey::new(part.part_key)
                    .expect("default part keys are non-empty"),
                name: part.name.to_string(),
                system_owned: true,
                sort_order: part.sort_order,
            },
            fields: part
                .fields
                .iter()
                .map(|field| BibleGraphField {
                    id: BibleGraphFieldId::new(default_field_id(node, part, field))
                        .expect("default field identifiers are non-empty"),
                    part_id: BibleGraphPartId::new(default_part_id(node, part))
                        .expect("default part identifiers are non-empty"),
                    field_key: BibleGraphFieldKey::new(field.field_key)
                        .expect("default field keys are non-empty"),
                    value: None,
                    sort_order: field.sort_order,
                })
                .collect(),
        })
        .collect()
}

pub fn builtin_bible_graph_schema_list_projection()
-> ProjectionEnvelope<BibleGraphSchemaListProjection> {
    ProjectionEnvelope::initial(BibleGraphSchemaListProjection {
        categories: [
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
        .map(|root| {
            let category = root.category();
            let create_schema = BUILTIN_BIBLE_GRAPH_SCHEMAS
                .iter()
                .find(|schema| schema.category == category);
            BibleGraphCategoryProjection {
                category,
                display_name: category.display_name().to_string(),
                visual_style: category.visual_style(),
                root_node_id: root.node_id(),
                root_schema_key: root.schema_key(),
                create_schema_key: create_schema.map(|schema| {
                    BibleGraphSchemaKey::new(schema.schema_key)
                        .expect("default schema keys are non-empty")
                }),
                default_node_name: create_schema.map(|schema| schema.default_node_name.to_string()),
            }
        })
        .collect(),
        schemas: BUILTIN_BIBLE_GRAPH_SCHEMAS
            .iter()
            .map(|schema| BibleGraphSchemaProjection {
                schema_key: BibleGraphSchemaKey::new(schema.schema_key)
                    .expect("default schema keys are non-empty"),
                category: schema.category.clone(),
                display_name: schema.display_name.to_string(),
                visual_style: schema.category.visual_style(),
                default_node_name: schema.default_node_name.to_string(),
                canonical_parent_id: schema.canonical_parent_id.map(|parent_id| {
                    BibleGraphNodeId::new(parent_id)
                        .expect("default canonical parent identifiers are non-empty")
                }),
                canonical_root_schema_key: schema.canonical_root_schema_key.map(|schema_key| {
                    BibleGraphSchemaKey::new(schema_key)
                        .expect("default canonical root schema keys are non-empty")
                }),
                parts: schema
                    .parts
                    .iter()
                    .map(|part| BibleGraphPartSchemaProjection {
                        part_key: BibleGraphPartKey::new(part.part_key)
                            .expect("default part keys are non-empty"),
                        name: part.name.to_string(),
                        sort_order: part.sort_order,
                        fields: part
                            .fields
                            .iter()
                            .map(|field| BibleGraphFieldSchemaProjection {
                                field_key: BibleGraphFieldKey::new(field.field_key)
                                    .expect("default field keys are non-empty"),
                                sort_order: field.sort_order,
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
    })
}

fn default_part_id(node: &BibleGraphNode, part: &BibleGraphPartDefault) -> String {
    format!("part.default.{}.{}", node.id.as_str(), part.part_key)
}

fn default_field_id(
    node: &BibleGraphNode,
    part: &BibleGraphPartDefault,
    field: &BibleGraphFieldDefault,
) -> String {
    format!(
        "field.default.{}.{}.{}",
        node.id.as_str(),
        part.part_key,
        field.field_key
    )
}

pub const BUILTIN_BIBLE_GRAPH_SCHEMAS: &[BibleGraphSchemaDefault] = &[
    BibleGraphSchemaDefault {
        schema_key: "character",
        category: BibleGraphNodeCategory::Character,
        display_name: "Character",
        default_node_name: "New Character",
        canonical_parent_id: Some("canonical.characters"),
        canonical_root_schema_key: Some("canonical.root.characters"),
        parts: &[
            BibleGraphPartDefault {
                part_key: "profile",
                name: "Profile",
                sort_order: 10,
                fields: &[
                    BibleGraphFieldDefault {
                        field_key: "summary",
                        sort_order: 10,
                    },
                    BibleGraphFieldDefault {
                        field_key: "tagline",
                        sort_order: 20,
                    },
                    BibleGraphFieldDefault {
                        field_key: "motivation",
                        sort_order: 30,
                    },
                ],
            },
            BibleGraphPartDefault {
                part_key: "appearance",
                name: "Appearance",
                sort_order: 20,
                fields: &[
                    BibleGraphFieldDefault {
                        field_key: "description",
                        sort_order: 10,
                    },
                    BibleGraphFieldDefault {
                        field_key: "costume",
                        sort_order: 20,
                    },
                ],
            },
        ],
    },
    BibleGraphSchemaDefault {
        schema_key: "location",
        category: BibleGraphNodeCategory::Location,
        display_name: "Location",
        default_node_name: "New Location",
        canonical_parent_id: Some("canonical.places"),
        canonical_root_schema_key: Some("canonical.root.places"),
        parts: &[BibleGraphPartDefault {
            part_key: "environment",
            name: "Environment",
            sort_order: 10,
            fields: &[
                BibleGraphFieldDefault {
                    field_key: "description",
                    sort_order: 10,
                },
                BibleGraphFieldDefault {
                    field_key: "weather",
                    sort_order: 20,
                },
            ],
        }],
    },
    BibleGraphSchemaDefault {
        schema_key: "prop",
        category: BibleGraphNodeCategory::Prop,
        display_name: "Prop",
        default_node_name: "New Prop",
        canonical_parent_id: Some("canonical.objects"),
        canonical_root_schema_key: Some("canonical.root.objects"),
        parts: &[BibleGraphPartDefault {
            part_key: "identity",
            name: "Identity",
            sort_order: 10,
            fields: &[
                BibleGraphFieldDefault {
                    field_key: "description",
                    sort_order: 10,
                },
                BibleGraphFieldDefault {
                    field_key: "significance",
                    sort_order: 20,
                },
            ],
        }],
    },
    BibleGraphSchemaDefault {
        schema_key: "theme",
        category: BibleGraphNodeCategory::Theme,
        display_name: "Theme",
        default_node_name: "New Theme",
        canonical_parent_id: Some("canonical.themes"),
        canonical_root_schema_key: Some("canonical.root.themes"),
        parts: &[BibleGraphPartDefault {
            part_key: "meaning",
            name: "Meaning",
            sort_order: 10,
            fields: &[
                BibleGraphFieldDefault {
                    field_key: "statement",
                    sort_order: 10,
                },
                BibleGraphFieldDefault {
                    field_key: "counterpoint",
                    sort_order: 20,
                },
            ],
        }],
    },
    BibleGraphSchemaDefault {
        schema_key: "event",
        category: BibleGraphNodeCategory::Event,
        display_name: "Event",
        default_node_name: "New Event",
        canonical_parent_id: Some("canonical.events"),
        canonical_root_schema_key: Some("canonical.root.events"),
        parts: &[BibleGraphPartDefault {
            part_key: "story",
            name: "Story",
            sort_order: 10,
            fields: &[
                BibleGraphFieldDefault {
                    field_key: "summary",
                    sort_order: 10,
                },
                BibleGraphFieldDefault {
                    field_key: "consequence",
                    sort_order: 20,
                },
            ],
        }],
    },
    BibleGraphSchemaDefault {
        schema_key: "culture",
        category: BibleGraphNodeCategory::Culture,
        display_name: "Culture",
        default_node_name: "New Culture",
        canonical_parent_id: Some("canonical.cultures"),
        canonical_root_schema_key: Some("canonical.root.cultures"),
        parts: &[],
    },
    BibleGraphSchemaDefault {
        schema_key: "rule",
        category: BibleGraphNodeCategory::Rule,
        display_name: "Rule",
        default_node_name: "New Rule",
        canonical_parent_id: Some("canonical.rules"),
        canonical_root_schema_key: Some("canonical.root.rules"),
        parts: &[],
    },
    BibleGraphSchemaDefault {
        schema_key: "reference",
        category: BibleGraphNodeCategory::Reference,
        display_name: "Reference",
        default_node_name: "New Reference",
        canonical_parent_id: Some("canonical.references"),
        canonical_root_schema_key: Some("canonical.root.references"),
        parts: &[],
    },
    BibleGraphSchemaDefault {
        schema_key: "detail",
        category: BibleGraphNodeCategory::Detail,
        display_name: "Detail",
        default_node_name: "New Detail",
        canonical_parent_id: None,
        canonical_root_schema_key: None,
        parts: &[],
    },
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn built_in_schemas_are_stable_and_non_empty() {
        let schemas: Vec<_> = BUILTIN_BIBLE_GRAPH_SCHEMAS
            .iter()
            .map(|schema| schema.schema_key)
            .collect();

        assert_eq!(
            schemas,
            [
                "character",
                "location",
                "prop",
                "theme",
                "event",
                "culture",
                "rule",
                "reference",
                "detail"
            ]
        );
        assert!(
            BUILTIN_BIBLE_GRAPH_SCHEMAS
                .iter()
                .filter(|schema| schema.category != BibleGraphNodeCategory::Detail)
                .all(|schema| schema.canonical_parent_id.is_some()
                    && schema.canonical_root_schema_key.is_some())
        );
        assert!(
            BUILTIN_BIBLE_GRAPH_SCHEMAS
                .iter()
                .find(|schema| schema.schema_key == "detail")
                .is_some_and(|schema| schema.canonical_parent_id.is_none()
                    && schema.canonical_root_schema_key.is_none()
                    && schema.parts.is_empty())
        );
    }

    #[test]
    fn built_in_schema_keys_are_unique() {
        let mut schema_keys = HashSet::new();

        for schema in BUILTIN_BIBLE_GRAPH_SCHEMAS {
            assert!(schema_keys.insert(schema.schema_key));
            let mut part_keys = HashSet::new();
            for part in schema.parts {
                assert!(part_keys.insert(part.part_key));
                let mut field_keys = HashSet::new();
                for field in part.fields {
                    assert!(field_keys.insert(field.field_key));
                }
            }
        }
    }

    #[test]
    fn default_projection_is_derived_without_values() {
        let node = BibleGraphNode {
            id: super::super::BibleGraphNodeId::new("node.character.ada").unwrap(),
            parent_id: None,
            schema_key: BibleGraphSchemaKey::new("character").unwrap(),
            name: "Ada".to_string(),
            system_owned: false,
            sort_order: 0,
        };

        let parts = default_part_projections_for_node(&node);

        assert_eq!(parts[0].part.part_key.as_str(), "profile");
        assert!(parts[0].part.system_owned);
        assert_eq!(parts[0].fields[1].field_key.as_str(), "tagline");
        assert!(parts[0].fields.iter().all(|field| field.value.is_none()));
    }

    #[test]
    fn schema_list_projection_round_trips() {
        let projection = builtin_bible_graph_schema_list_projection();

        let json = serde_json::to_string(&projection).unwrap();
        let round_trip: ProjectionEnvelope<BibleGraphSchemaListProjection> =
            serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, projection);
        assert_eq!(
            round_trip.payload.schemas[0].parts[0].fields[1]
                .field_key
                .as_str(),
            "tagline"
        );
    }
}
