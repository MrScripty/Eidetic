use eidetic_core::contracts::BibleGraphNodeCategory;

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
    pub(crate) fn from_category(category: &BibleGraphNodeCategory) -> Self {
        match category {
            BibleGraphNodeCategory::Character => Self::Character,
            BibleGraphNodeCategory::Location => Self::Location,
            BibleGraphNodeCategory::Prop => Self::Prop,
            BibleGraphNodeCategory::Culture => Self::Culture,
            BibleGraphNodeCategory::Event => Self::Event,
            BibleGraphNodeCategory::Theme => Self::Theme,
            BibleGraphNodeCategory::Rule => Self::Rule,
            BibleGraphNodeCategory::Reference => Self::Reference,
            BibleGraphNodeCategory::Canonical => Self::Canonical,
            BibleGraphNodeCategory::Other => Self::Other,
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

pub(crate) fn node_fill_color(category: &BibleGraphNodeCategory) -> &'static str {
    BibleGraphVisualCategory::from_category(category).fill_color()
}
