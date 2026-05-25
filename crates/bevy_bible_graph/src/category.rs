#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BibleGraphVisualCategory {
    Character,
    Location,
    Prop,
    Culture,
    Event,
    Theme,
    Rule,
    Reference,
    Canonical,
    Other,
}

impl BibleGraphVisualCategory {
    pub(crate) fn from_schema_key(schema_key: &str) -> Self {
        if schema_key.contains("character") {
            Self::Character
        } else if schema_key.contains("place") || schema_key.contains("location") {
            Self::Location
        } else if schema_key.contains("object") || schema_key.contains("prop") {
            Self::Prop
        } else if schema_key.contains("culture") {
            Self::Culture
        } else if schema_key.contains("event") {
            Self::Event
        } else if schema_key.contains("theme") {
            Self::Theme
        } else if schema_key.contains("rule") {
            Self::Rule
        } else if schema_key.contains("reference") {
            Self::Reference
        } else if schema_key.contains("canonical") {
            Self::Canonical
        } else {
            Self::Other
        }
    }

    pub(crate) fn fill_color(self) -> &'static str {
        match self {
            Self::Character => "#2f7a6e",
            Self::Location => "#3f668f",
            Self::Prop => "#7a5c8f",
            Self::Culture => "#6f7a2f",
            Self::Event => "#8f4f5c",
            Self::Theme => "#8a6f3d",
            Self::Rule => "#8f5c3f",
            Self::Reference => "#4f7f8f",
            Self::Canonical => "#536f88",
            Self::Other => "#34495e",
        }
    }
}

pub(crate) fn node_fill_color(schema_key: &str) -> &'static str {
    BibleGraphVisualCategory::from_schema_key(schema_key).fill_color()
}
