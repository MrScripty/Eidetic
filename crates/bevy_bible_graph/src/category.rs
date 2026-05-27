use eidetic_core::contracts::BibleGraphNodeCategory;

pub(crate) fn node_fill_color(category: &BibleGraphNodeCategory) -> &'static str {
    category.fill_color()
}
