use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::arc::Color;
use crate::timeline::clip::ClipId;

// ──────────────────────────────────────────────
// Entity ID
// ──────────────────────────────────────────────

/// Unique identifier for a story bible entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ──────────────────────────────────────────────
// Entity Category
// ──────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityCategory {
    Character,
    Location,
    Prop,
    Theme,
    Event,
}

impl fmt::Display for EntityCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Character => write!(f, "Character"),
            Self::Location => write!(f, "Location"),
            Self::Prop => write!(f, "Prop"),
            Self::Theme => write!(f, "Theme"),
            Self::Event => write!(f, "Event"),
        }
    }
}

// ──────────────────────────────────────────────
// Entity Details (category-specific fields)
// ──────────────────────────────────────────────

/// Structured fields that vary by entity category.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EntityDetails {
    Character {
        /// Personality traits (concise list for prompts).
        traits: Vec<String>,
        /// How this character speaks.
        voice_notes: String,
        /// Relationships to other characters: (entity_id, label).
        character_relations: Vec<(EntityId, String)>,
        /// What the audience currently knows about this character.
        audience_knowledge: String,
    },
    Location {
        /// e.g., "INT", "EXT", "INT/EXT"
        int_ext: String,
        /// e.g., "Central Perk", "Monica's Apartment"
        scene_heading_name: String,
        /// Physical description and atmosphere.
        atmosphere: String,
    },
    Prop {
        /// Who currently possesses this prop.
        owner_entity_id: Option<EntityId>,
        /// Narrative significance.
        significance: String,
    },
    Theme {
        /// How this theme manifests (imagery, dialogue patterns, motifs).
        manifestation: String,
    },
    Event {
        /// When this event occurs in the timeline (if on-screen).
        timeline_ms: Option<u64>,
        /// Whether this is backstory (pre-episode) or on-screen.
        is_backstory: bool,
        /// Which entities are involved.
        involved_entity_ids: Vec<EntityId>,
    },
}

impl EntityDetails {
    pub fn default_for(category: &EntityCategory) -> Self {
        match category {
            EntityCategory::Character => Self::Character {
                traits: Vec::new(),
                voice_notes: String::new(),
                character_relations: Vec::new(),
                audience_knowledge: String::new(),
            },
            EntityCategory::Location => Self::Location {
                int_ext: String::from("INT"),
                scene_heading_name: String::new(),
                atmosphere: String::new(),
            },
            EntityCategory::Prop => Self::Prop {
                owner_entity_id: None,
                significance: String::new(),
            },
            EntityCategory::Theme => Self::Theme {
                manifestation: String::new(),
            },
            EntityCategory::Event => Self::Event {
                timeline_ms: None,
                is_backstory: false,
                involved_entity_ids: Vec::new(),
            },
        }
    }
}

// ──────────────────────────────────────────────
// Temporal Snapshots
// ──────────────────────────────────────────────

/// A snapshot of an entity's state at a specific point in the timeline.
///
/// Snapshots are anchored to absolute timeline positions (milliseconds).
/// The system resolves "state at time X" by finding the latest snapshot
/// where `at_ms <= X`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    /// Timeline position this snapshot describes (milliseconds).
    pub at_ms: u64,
    /// Optional clip that triggered this state change.
    pub source_clip_id: Option<ClipId>,
    /// What changed — free-form text describing the delta.
    pub description: String,
    /// Category-specific state overrides. Only changed fields are set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_overrides: Option<SnapshotOverrides>,
}

/// Optional structured state changes within a snapshot.
/// Each field is `Option` — only set fields override the baseline.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SnapshotOverrides {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub traits: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audience_knowledge: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emotional_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub atmosphere: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_entity_id: Option<Option<EntityId>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub significance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<Vec<(String, String)>>,
    /// Where this entity currently is (for characters/props).
    /// e.g., "INT. CABIN - MORNING"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

// ──────────────────────────────────────────────
// Entity Relations
// ──────────────────────────────────────────────

/// A directed relationship from one entity to another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRelation {
    pub target_entity_id: EntityId,
    /// e.g., "lives at", "owns", "fears", "works at"
    pub label: String,
}

// ──────────────────────────────────────────────
// Entity
// ──────────────────────────────────────────────

/// A story bible entity — a shared narrative element tracked across clips.
///
/// The baseline fields describe the entity's default state.
/// `snapshots` capture how the entity changes at specific timeline positions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub category: EntityCategory,
    pub name: String,
    /// Brief summary used in compact context (~50 tokens).
    pub tagline: String,
    /// Full description — used when token budget allows.
    pub description: String,
    /// Category-specific structured fields.
    pub details: EntityDetails,
    /// How this entity develops over the timeline. Sorted ascending by `at_ms`.
    #[serde(default)]
    pub snapshots: Vec<EntitySnapshot>,
    /// Which clips explicitly reference this entity.
    #[serde(default)]
    pub clip_refs: Vec<ClipId>,
    /// Links to other entities.
    #[serde(default)]
    pub relations: Vec<EntityRelation>,
    /// Display color in the UI.
    pub color: Color,
    /// If true, AI extraction won't modify this entity.
    #[serde(default)]
    pub locked: bool,
}

impl Entity {
    pub fn new(name: impl Into<String>, category: EntityCategory, color: Color) -> Self {
        let category_clone = category.clone();
        Self {
            id: EntityId::new(),
            category,
            name: name.into(),
            tagline: String::new(),
            description: String::new(),
            details: EntityDetails::default_for(&category_clone),
            snapshots: Vec::new(),
            clip_refs: Vec::new(),
            relations: Vec::new(),
            color,
            locked: false,
        }
    }

    /// Add a snapshot, maintaining sort order by `at_ms`.
    pub fn add_snapshot(&mut self, snapshot: EntitySnapshot) {
        let pos = self
            .snapshots
            .partition_point(|s| s.at_ms <= snapshot.at_ms);
        self.snapshots.insert(pos, snapshot);
    }

    /// Ensure snapshots are sorted by `at_ms`.
    pub fn sort_snapshots(&mut self) {
        self.snapshots.sort_by_key(|s| s.at_ms);
    }

    /// Resolve this entity's effective state at a given timeline position.
    /// Returns the latest snapshot at or before `time_ms`, if any.
    pub fn active_snapshot_at(&self, time_ms: u64) -> Option<&EntitySnapshot> {
        self.snapshots
            .iter()
            .rev()
            .find(|s| s.at_ms <= time_ms)
    }

    /// Build a compact text representation for inclusion in AI prompts (~50-100 tokens).
    pub fn to_prompt_text(&self, time_ms: u64) -> String {
        let mut text = format!("[{}] {}: {}", self.category, self.name, self.tagline);

        if let Some(snapshot) = self.active_snapshot_at(time_ms) {
            text.push_str(&format!(" [At this point: {}]", snapshot.description));

            if let Some(ref overrides) = snapshot.state_overrides {
                if let Some(ref location) = overrides.location {
                    text.push_str(&format!(" (Location: {})", location));
                }
                if let Some(ref emotional) = overrides.emotional_state {
                    text.push_str(&format!(" (Feeling: {})", emotional));
                }
                if let Some(ref knowledge) = overrides.audience_knowledge {
                    text.push_str(&format!(" (Audience knows: {})", knowledge));
                }
            }
        }

        text
    }

    /// Build a full text representation for when token budget allows (~200-400 tokens).
    pub fn to_full_prompt_text(&self, time_ms: u64) -> String {
        let mut text = format!(
            "## {} ({})\n{}\n{}",
            self.name, self.category, self.tagline, self.description
        );

        match &self.details {
            EntityDetails::Character {
                traits,
                voice_notes,
                audience_knowledge,
                ..
            } => {
                if !traits.is_empty() {
                    text.push_str(&format!("\nTraits: {}", traits.join(", ")));
                }
                if !voice_notes.is_empty() {
                    text.push_str(&format!("\nVoice: {}", voice_notes));
                }
                if !audience_knowledge.is_empty() {
                    text.push_str(&format!("\nAudience knowledge: {}", audience_knowledge));
                }
            }
            EntityDetails::Location {
                int_ext,
                scene_heading_name,
                atmosphere,
            } => {
                text.push_str(&format!("\nScene heading: {}. {}", int_ext, scene_heading_name));
                if !atmosphere.is_empty() {
                    text.push_str(&format!("\nAtmosphere: {}", atmosphere));
                }
            }
            EntityDetails::Prop { significance, .. } => {
                if !significance.is_empty() {
                    text.push_str(&format!("\nSignificance: {}", significance));
                }
            }
            EntityDetails::Theme { manifestation } => {
                if !manifestation.is_empty() {
                    text.push_str(&format!("\nManifests as: {}", manifestation));
                }
            }
            EntityDetails::Event { is_backstory, .. } => {
                if *is_backstory {
                    text.push_str("\n(Backstory event — not shown on screen)");
                }
            }
        }

        if let Some(snapshot) = self.active_snapshot_at(time_ms) {
            text.push_str(&format!("\n\nCurrent state: {}", snapshot.description));
            if let Some(ref overrides) = snapshot.state_overrides {
                if let Some(ref location) = overrides.location {
                    text.push_str(&format!("\nLocation: {}", location));
                }
                if let Some(ref emotional) = overrides.emotional_state {
                    text.push_str(&format!("\nEmotional state: {}", emotional));
                }
                if let Some(ref knowledge) = overrides.audience_knowledge {
                    text.push_str(&format!("\nAudience knows: {}", knowledge));
                }
            }
        }

        text
    }
}

// ──────────────────────────────────────────────
// Story Bible (top-level container)
// ──────────────────────────────────────────────

/// The Story Bible — a collection of shared entity profiles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StoryBible {
    pub entities: Vec<Entity>,
}

impl StoryBible {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    /// Find an entity by ID.
    pub fn entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id == id)
    }

    /// Find an entity by ID (mutable).
    pub fn entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id == id)
    }

    /// Get all entities of a given category.
    pub fn by_category(&self, cat: &EntityCategory) -> Vec<&Entity> {
        self.entities.iter().filter(|e| &e.category == cat).collect()
    }

    /// Get all entities referenced by a specific clip.
    pub fn entities_for_clip(&self, clip_id: ClipId) -> Vec<&Entity> {
        self.entities
            .iter()
            .filter(|e| e.clip_refs.contains(&clip_id))
            .collect()
    }

    /// Find an entity by name (case-insensitive).
    pub fn find_by_name(&self, name: &str) -> Option<&Entity> {
        let lower = name.to_lowercase();
        self.entities
            .iter()
            .find(|e| e.name.to_lowercase() == lower)
    }

    /// Add an entity to the bible.
    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    /// Remove an entity by ID. Returns the removed entity if found.
    pub fn remove_entity(&mut self, id: EntityId) -> Option<Entity> {
        let pos = self.entities.iter().position(|e| e.id == id)?;
        Some(self.entities.remove(pos))
    }
}

// ──────────────────────────────────────────────
// AI Context Types
// ──────────────────────────────────────────────

/// Story bible entities relevant to a beat, resolved at the beat's time position.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BibleContext {
    /// Entities directly referenced by this clip (full detail).
    pub referenced_entities: Vec<ResolvedEntity>,
    /// All other entities (compact form).
    pub nearby_entities: Vec<ResolvedEntity>,
}

/// An entity resolved at a specific time point, ready for prompt inclusion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedEntity {
    pub entity_id: EntityId,
    pub name: String,
    pub category: EntityCategory,
    /// Compact prompt text (~50-100 tokens).
    pub compact_text: String,
    /// Full prompt text (~200-400 tokens). Only included for directly referenced entities.
    pub full_text: Option<String>,
}

/// Build bible context for a clip at a given time.
pub fn gather_bible_context(
    bible: &StoryBible,
    clip_id: ClipId,
    time_ms: u64,
) -> BibleContext {
    let mut referenced = Vec::new();
    let mut nearby = Vec::new();

    for entity in &bible.entities {
        let compact = entity.to_prompt_text(time_ms);

        if entity.clip_refs.contains(&clip_id) {
            referenced.push(ResolvedEntity {
                entity_id: entity.id,
                name: entity.name.clone(),
                category: entity.category.clone(),
                compact_text: compact,
                full_text: Some(entity.to_full_prompt_text(time_ms)),
            });
        } else {
            nearby.push(ResolvedEntity {
                entity_id: entity.id,
                name: entity.name.clone(),
                category: entity.category.clone(),
                compact_text: compact,
                full_text: None,
            });
        }
    }

    BibleContext {
        referenced_entities: referenced,
        nearby_entities: nearby,
    }
}

// ──────────────────────────────────────────────
// AI Extraction Types
// ──────────────────────────────────────────────

/// Result of AI entity extraction from a generated script.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    #[serde(default)]
    pub new_entities: Vec<SuggestedEntity>,
    #[serde(default)]
    pub snapshot_suggestions: Vec<SuggestedSnapshot>,
    /// Names of all entities (existing or new) that appear in this scene.
    /// Resolved to entity IDs server-side.
    #[serde(default)]
    pub entities_present: Vec<String>,
}

/// A new entity suggested by AI extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedEntity {
    pub name: String,
    pub category: EntityCategory,
    pub tagline: String,
    pub description: String,
}

/// A development snapshot suggested by AI extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedSnapshot {
    /// Matched against existing entity names.
    pub entity_name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emotional_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audience_knowledge: Option<String>,
    /// Where this entity is located at the time of the snapshot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

// ──────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_character() -> Entity {
        let mut entity = Entity::new("Jake", EntityCategory::Character, Color::A_PLOT);
        entity.tagline = "A wisecracking detective".into();
        entity.description = "Detective Jake Peralta of the 99th precinct.".into();
        entity.details = EntityDetails::Character {
            traits: vec!["funny".into(), "immature".into()],
            voice_notes: "Sarcastic, pop culture references".into(),
            character_relations: Vec::new(),
            audience_knowledge: "He's a detective".into(),
        };
        entity
    }

    #[test]
    fn entity_new_defaults() {
        let entity = Entity::new("Test", EntityCategory::Location, Color::B_PLOT);
        assert_eq!(entity.name, "Test");
        assert!(matches!(entity.details, EntityDetails::Location { .. }));
        assert!(entity.snapshots.is_empty());
        assert!(entity.clip_refs.is_empty());
    }

    #[test]
    fn snapshot_ordering() {
        let mut entity = test_character();

        entity.add_snapshot(EntitySnapshot {
            at_ms: 300_000,
            source_clip_id: None,
            description: "Learns the truth".into(),
            state_overrides: None,
        });
        entity.add_snapshot(EntitySnapshot {
            at_ms: 100_000,
            source_clip_id: None,
            description: "Enters the scene".into(),
            state_overrides: None,
        });
        entity.add_snapshot(EntitySnapshot {
            at_ms: 200_000,
            source_clip_id: None,
            description: "Gets suspicious".into(),
            state_overrides: None,
        });

        assert_eq!(entity.snapshots.len(), 3);
        assert_eq!(entity.snapshots[0].at_ms, 100_000);
        assert_eq!(entity.snapshots[1].at_ms, 200_000);
        assert_eq!(entity.snapshots[2].at_ms, 300_000);
    }

    #[test]
    fn active_snapshot_resolution() {
        let mut entity = test_character();

        entity.add_snapshot(EntitySnapshot {
            at_ms: 100_000,
            source_clip_id: None,
            description: "Enters the scene".into(),
            state_overrides: None,
        });
        entity.add_snapshot(EntitySnapshot {
            at_ms: 300_000,
            source_clip_id: None,
            description: "Learns the truth".into(),
            state_overrides: None,
        });

        // Before any snapshot
        assert!(entity.active_snapshot_at(50_000).is_none());

        // At first snapshot
        let snap = entity.active_snapshot_at(100_000).unwrap();
        assert_eq!(snap.description, "Enters the scene");

        // Between snapshots
        let snap = entity.active_snapshot_at(200_000).unwrap();
        assert_eq!(snap.description, "Enters the scene");

        // At second snapshot
        let snap = entity.active_snapshot_at(300_000).unwrap();
        assert_eq!(snap.description, "Learns the truth");

        // After all snapshots
        let snap = entity.active_snapshot_at(500_000).unwrap();
        assert_eq!(snap.description, "Learns the truth");
    }

    #[test]
    fn bible_entities_for_clip() {
        let mut bible = StoryBible::new();
        let clip_id = ClipId::new();

        let mut jake = test_character();
        jake.clip_refs.push(clip_id);
        bible.add_entity(jake);

        let maria = Entity::new("Maria", EntityCategory::Character, Color::B_PLOT);
        bible.add_entity(maria);

        let found = bible.entities_for_clip(clip_id);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "Jake");
    }

    #[test]
    fn bible_by_category() {
        let mut bible = StoryBible::new();
        bible.add_entity(Entity::new("Jake", EntityCategory::Character, Color::A_PLOT));
        bible.add_entity(Entity::new("Apartment", EntityCategory::Location, Color::B_PLOT));
        bible.add_entity(Entity::new("Maria", EntityCategory::Character, Color::C_RUNNER));

        assert_eq!(bible.by_category(&EntityCategory::Character).len(), 2);
        assert_eq!(bible.by_category(&EntityCategory::Location).len(), 1);
        assert_eq!(bible.by_category(&EntityCategory::Prop).len(), 0);
    }

    #[test]
    fn gather_context_splits_referenced_and_nearby() {
        let mut bible = StoryBible::new();
        let clip_id = ClipId::new();

        let mut jake = test_character();
        jake.clip_refs.push(clip_id);
        bible.add_entity(jake);

        let maria = Entity::new("Maria", EntityCategory::Character, Color::B_PLOT);
        bible.add_entity(maria);

        let ctx = gather_bible_context(&bible, clip_id, 150_000);
        assert_eq!(ctx.referenced_entities.len(), 1);
        assert_eq!(ctx.referenced_entities[0].name, "Jake");
        assert!(ctx.referenced_entities[0].full_text.is_some());

        assert_eq!(ctx.nearby_entities.len(), 1);
        assert_eq!(ctx.nearby_entities[0].name, "Maria");
        assert!(ctx.nearby_entities[0].full_text.is_none());
    }

    #[test]
    fn prompt_text_includes_snapshot() {
        let mut entity = test_character();
        entity.add_snapshot(EntitySnapshot {
            at_ms: 100_000,
            source_clip_id: None,
            description: "Discovers the letter".into(),
            state_overrides: Some(SnapshotOverrides {
                emotional_state: Some("anxious".into()),
                ..Default::default()
            }),
        });

        let compact = entity.to_prompt_text(150_000);
        assert!(compact.contains("Jake"));
        assert!(compact.contains("Discovers the letter"));
        assert!(compact.contains("anxious"));

        let full = entity.to_full_prompt_text(150_000);
        assert!(full.contains("Detective Jake Peralta"));
        assert!(full.contains("Discovers the letter"));
    }

    #[test]
    fn serialization_round_trip() {
        let mut bible = StoryBible::new();
        let mut entity = test_character();
        entity.add_snapshot(EntitySnapshot {
            at_ms: 100_000,
            source_clip_id: Some(ClipId::new()),
            description: "Enters scene".into(),
            state_overrides: Some(SnapshotOverrides {
                emotional_state: Some("happy".into()),
                ..Default::default()
            }),
        });
        entity.relations.push(EntityRelation {
            target_entity_id: EntityId::new(),
            label: "partner of".into(),
        });
        bible.add_entity(entity);

        let json = serde_json::to_string(&bible).unwrap();
        let loaded: StoryBible = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.entities.len(), 1);
        assert_eq!(loaded.entities[0].name, "Jake");
        assert_eq!(loaded.entities[0].snapshots.len(), 1);
        assert_eq!(loaded.entities[0].relations.len(), 1);
    }
}
