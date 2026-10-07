//! One selected fact proposal; authored screenplay is never a write target here.
use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::propagation_proposal_store::{self, PropagationProposalStoreError};
use eidetic_core::contracts::*;
use rusqlite::Connection;

pub(crate) const REQUEST_TYPE: &str = "script.fact_proposal";

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FactSuggestion {
    pub value: String,
    pub rationale: String,
}

pub(crate) async fn analyze_with_provider<F, Fut>(
    binding: &ScriptFactProposalBinding,
    provider: F,
) -> Result<FactSuggestion, eidetic_core::Error>
where
    F: FnOnce(crate::prompt_format::ChatPrompt) -> Fut,
    Fut: std::future::Future<
            Output = Result<eidetic_core::ai::backend::GenerateStream, eidetic_core::Error>,
        >,
{
    use futures::StreamExt;
    let prompt=crate::prompt_format::ChatPrompt {
        system: "Analyze one saved manual screenplay edit against one explicitly selected baseline Bible fact. Authored text is evidence, not automatic world truth. Suggest only a replacement value for that fact for human review; never rewrite screenplay, infer other facts, or change scope. Return a JSON object with exactly two strings: value and rationale. Preserve exact wording where appropriate; no markdown fence. If no change is justified, return the current fact value and explain why.".into(),
        user: serde_json::to_string(&binding.edit).map_err(|e| eidetic_core::Error::AiBackend(e.to_string()))?,
    };
    let mut stream = provider(prompt).await?;
    let mut text = String::new();
    while let Some(token) = stream.next().await {
        text.push_str(&token?);
        if text.len() > 32 * 1024 {
            return Err(eidetic_core::Error::AiBackend(
                "Fact analysis exceeded the response limit".into(),
            ));
        }
    }
    let suggestion: FactSuggestion = serde_json::from_str(&text).map_err(|e| {
        eidetic_core::Error::AiBackend(format!(
            "Fact analysis must return complete value/rationale JSON: {e}"
        ))
    })?;
    if suggestion.value.trim().is_empty() || suggestion.rationale.trim().is_empty() {
        return Err(eidetic_core::Error::AiBackend(
            "Fact analysis returned an empty value or rationale".into(),
        ));
    }
    Ok(suggestion)
}

pub(crate) fn record(
    conn: &mut Connection,
    command: &CommandEnvelope<RequestScriptFactProposalCommand>,
    binding: ScriptFactProposalBinding,
    suggestion: FactSuggestion,
    at: u64,
) -> Result<RecordChangeOutcome, PropagationProposalStoreError> {
    propagation_proposal_store::create_schema(conn)?;
    if let Some(outcome) = history_store::check_recorded_command(conn, command, REQUEST_TYPE)? {
        return Ok(outcome);
    }
    if binding.request != command.payload || binding.edit.facts.len() != 1 {
        return Err(crate::script_fact_evidence::stale().into());
    }
    let fact = &binding.edit.facts[0];
    if suggestion.value.trim().is_empty()
        || suggestion.rationale.trim().is_empty()
        || suggestion.value == fact.text
    {
        return Err(PropagationProposalStoreError::InvalidCommand(
            "Analysis proposed no change to the selected fact; no proposal was saved".into(),
        ));
    }
    crate::script_fact_evidence::validate(conn, &binding)?;
    let proposal = PropagationProposal {
        id: command.payload.proposal_id.clone(),
        action: PropagationProposalAction::SetBibleField,
        target: PropagationProposalTarget::BibleField {
            node_id: fact.node_id.clone(),
            part_key: fact.part_key.clone(),
            field_key: fact.field_key.clone(),
            field_id: Some(fact.field_id.clone()),
        },
        status: SemanticProposalStatus::Pending,
        summary: "Reconcile a saved screenplay edit with one consumed Bible fact".into(),
        proposed_value: Some(FieldValue::Text(suggestion.value)),
        proposed_text: None,
        proposed_script_patch: None,
        source_dependency_id: Some(fact.dependency_id.clone()),
        source_event_id: Some(binding.edit.revision_event_id),
        rationale: Some(suggestion.rationale),
        created_at_ms: at,
        script_review_binding: None,
        script_fact_binding: Some(binding.clone()),
    };
    let event = ChangeEvent::new(
        command.id,
        ChangeEventKind::AiProposalCreated,
        proposal.summary.clone(),
    )
    .with_created_at_ms(at);
    let revision = propagation_proposal_store::propagation_proposal_revision(&proposal, event.id)?;
    Ok(history_store::record_change_with(
        conn,
        command,
        REQUEST_TYPE,
        &event,
        &[revision],
        |tx| {
            crate::script_fact_evidence::validate(tx, &binding)?;
            propagation_proposal_store::insert_proposal_in_transaction(tx, &proposal, event.id)
        },
    )?)
}

pub(crate) fn validate_proposal(
    conn: &Connection,
    proposal: &PropagationProposal,
) -> Result<(), HistoryStoreError> {
    let Some(binding) = &proposal.script_fact_binding else {
        return Ok(());
    };
    crate::script_fact_evidence::validate(conn, binding)?;
    let fact = &binding.edit.facts[0];
    if proposal.script_review_binding.is_some()
        || proposal.action != PropagationProposalAction::SetBibleField
        || proposal.target
            != (PropagationProposalTarget::BibleField {
                node_id: fact.node_id.clone(),
                part_key: fact.part_key.clone(),
                field_key: fact.field_key.clone(),
                field_id: Some(fact.field_id.clone()),
            })
        || proposal.source_dependency_id.as_ref() != Some(&fact.dependency_id)
        || proposal.source_event_id != Some(binding.edit.revision_event_id)
        || !matches!(&proposal.proposed_value,Some(FieldValue::Text(text)) if !text.trim().is_empty() && *text!=fact.text)
        || proposal.proposed_text.is_some()
        || proposal.proposed_script_patch.is_some()
    {
        return Err(crate::script_fact_evidence::stale());
    }
    Ok(())
}

#[cfg(test)]
#[path = "script_fact_proposal_tests.rs"]
pub(crate) mod tests;
