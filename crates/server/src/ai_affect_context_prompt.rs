use eidetic_core::contracts::{
    AffectProjection, AffectProvenance, AffectValue, ProjectionEnvelope,
};

pub(crate) fn append_affect_context(
    user: &mut String,
    context: &ProjectionEnvelope<AffectProjection>,
) {
    if context.payload.values.is_empty() {
        return;
    }

    user.push_str(
        "AFFECT CONSTRAINTS:\n\
         Use these backend-owned emotional constraints to guide tone and intensity. \
         Do not override locked/user-authored text or established bible facts.\n",
    );
    for value in &context.payload.values {
        user.push_str(&format_affect_value(value));
    }
    user.push('\n');
}

fn format_affect_value(value: &AffectValue) -> String {
    let labels = value
        .mood_labels
        .iter()
        .map(|label| label.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let rationale = value
        .rationale
        .as_deref()
        .filter(|rationale| !rationale.trim().is_empty())
        .unwrap_or("no rationale provided");
    format!(
        "- mood: {}; valence: {}; arousal: {}; intensity: {}; confidence: {}; provenance: {}; rationale: {}\n",
        labels,
        value.valence.basis_points(),
        value.arousal.basis_points(),
        value.intensity.basis_points(),
        value.confidence.basis_points(),
        provenance_label(&value.provenance),
        rationale,
    )
}

fn provenance_label(provenance: &AffectProvenance) -> &'static str {
    match provenance {
        AffectProvenance::UserAuthored => "user_authored",
        AffectProvenance::AgentProposed => "agent_proposed",
        AffectProvenance::ScriptEditDetected => "script_edit_detected",
        AffectProvenance::Imported => "imported",
    }
}
