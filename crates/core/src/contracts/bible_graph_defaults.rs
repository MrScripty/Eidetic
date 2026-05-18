use super::{
    BibleGraphField, BibleGraphFieldId, BibleGraphFieldKey, BibleGraphNode, BibleGraphPart,
    BibleGraphPartId, BibleGraphPartKey, BibleGraphPartProjection, BibleGraphSchemaKey,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BibleGraphSchemaDefault {
    pub schema_key: &'static str,
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

        assert_eq!(schemas, ["character", "location", "prop", "theme", "event"]);
        assert!(
            BUILTIN_BIBLE_GRAPH_SCHEMAS
                .iter()
                .all(|schema| !schema.parts.is_empty())
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
}
