#!/usr/bin/env python3
"""Source-bound native name edit and targeted review; synthetic HTTP/SSE only."""
import importlib.util
import io
import json
import os
from pathlib import Path

spec = importlib.util.spec_from_file_location('name_relationship_helpers', Path(__file__).with_name('qualify-bible-relationship.py'))
relationship = importlib.util.module_from_spec(spec)
spec.loader.exec_module(relationship)
bible = relationship.bible
ui = relationship.ui
NODE_ID = 'qualification.mara'
BEFORE = 'Mara'
AFTER = 'Marisol'
REPLACEMENT = 'EXT. STATION - NIGHT\n\nEli spots Marisol and her blue umbrella.'


def name_prompt(user, system):
    return ('targeted screenplay update' in system and '"kind":"bible_node"' in user
            and '"input_excerpt":"Mara"' in user
            and '- Marisol [character] (qualification.mara)' in user
            and f'profile.tagline: {bible.BLUE}' in user
            and ui.MANUAL_TEXT in user and ui.PROPOSED_TEXT in user)


class NameProvider(bible.BibleFactProvider):
    records = []

    def do_POST(self):
        size = int(self.headers.get('Content-Length', '0'))
        if self.path != '/v1/chat/completions' or not 0 < size <= 512_000:
            self.send_error(400)
            return
        data = self.rfile.read(size)
        body = json.loads(data)
        user = next((m['content'] for m in body.get('messages', []) if m.get('role') == 'user'), '')
        system = next((m['content'] for m in body.get('messages', []) if m.get('role') == 'system'), '')
        if '"kind":"bible_node"' not in user:
            original = self.rfile
            self.rfile = io.BytesIO(data)
            try:
                super().do_POST()
            finally:
                self.rfile = original
            return
        valid = body.get('stream') is True and name_prompt(user, system)
        with self.records_lock:
            count = sum(r.get('kind', '').startswith('preview_name_') and r.get('accepted') for r in self.records)
            accepted = valid and count < 2
            self.records.append({'kind': f'preview_name_{count + 1}', 'accepted': accepted,
                'contains_exact_expected_context': valid, 'stream': True, 'real_model': False})
        if not accepted:
            self.send_error(422, 'Exact canonical name and screenplay evidence missing')
            return
        data = ('data: ' + json.dumps({'choices': [{'delta': {'content': REPLACEMENT}}]})
                + '\n\ndata: [DONE]\n\n').encode()
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)


def name(database):
    rows = ui.query(database, 'SELECT name FROM bible_graph_nodes WHERE id=? AND deleted_event_id IS NULL', (NODE_ID,))
    if len(rows) != 1:
        raise RuntimeError('Expected exactly one fixture Bible node')
    revisions = ui.query(database, "SELECT r.change_event_id FROM object_revisions r JOIN object_revision_fields f ON f.revision_id=r.id JOIN change_events e ON e.id=r.change_event_id WHERE r.object_kind='bible_node' AND r.object_id=? AND f.field_key='name' AND f.new_type='text' ORDER BY e.rowid DESC,r.rowid DESC LIMIT 1", (NODE_ID,))
    if len(revisions) != 1:
        raise RuntimeError('Expected exact owned name revision')
    return rows[0][0], revisions[0][0]


def edit_name(application, window, database, before, value):
    ui.reveal_button(application, before[0], window)
    field = ui.wait_for('native node name editor', lambda: ui.reveal(application,
        lambda n: n.name == 'Node name' and n.getState().contains(ui.pyatspi.STATE_EDITABLE)))
    ui.type_text(field, window, value)
    ui.reveal_button(application, 'Save', window)
    return ui.wait_for('exact authored name and new revision', lambda:
        name(database) if name(database)[0] == value and name(database)[1] != before[1] else None)


def receipt(database, proposal_id):
    rows = ui.query(database, 'SELECT binding_json FROM script_impact_proposal_bindings WHERE proposal_id=?', (proposal_id,))
    if len(rows) != 1:
        raise RuntimeError('Expected one durable name preview')
    binding = json.loads(rows[0][0])
    inputs = binding.get('bible_node_name_inputs')
    mara = [item for item in inputs or [] if item['node_id'] == NODE_ID]
    if len(mara) != 1 or mara[0]['name'] != AFTER:
        raise RuntimeError('Preview lacks exact name custody')
    return binding, mara[0]


def name_flow(application, window, database, fixture, evidence, checkpoint, capture):
    a_id, b_id = fixture['a']['id'], fixture['b']['id']
    a, b = ui.blocks(database, a_id)[0], ui.blocks(database, b_id)[0]
    original = name(database)
    if original[0] != BEFORE or b[1] != ui.PROPOSED_TEXT:
        raise RuntimeError('Unexpected name-flow starting canon')
    dependencies = ui.query(database, "SELECT d.id,r.target_revision_event_id FROM semantic_dependencies d JOIN semantic_dependency_revisions r ON r.dependency_id=d.id WHERE d.target_kind='bible_node' AND d.target_id=? AND r.source_revision_event_id=?", (NODE_ID, b[2]))
    if len(dependencies) != 1 or dependencies[0][1] != original[1]:
        raise RuntimeError('Generation did not bind its supplied Bible name')
    observed = evidence['bible_node_name'] = {'original_name': list(original), 'consumed_dependency': dependencies[0][0], 'real_model': False}
    relationship.begin_manual_draft(application, window)
    changed = edit_name(application, window, database, original, AFTER)
    ui.wait_for('manual draft survives name refresh', lambda: ui.editable(application, bible.DRAFT))
    ui.wait_for('name-specific downstream review', lambda: ui.reveal(application,
        lambda n: 'Bible name changed.' in ui.text_of(n)))
    if ui.blocks(database, a_id)[0] != a or ui.blocks(database, b_id)[0] != b:
        raise RuntimeError('Name edit changed saved screenplay')
    observed.update(edited_name=list(changed), saved_scripts_preserved=True, exact_manual_draft_preserved=True)
    checkpoint('exact authored Mara to Marisol edit; saved screenplay needs review')
    capture('eidetic-name-review.png')
    relationship.cancel_manual_draft(application, window)
    relationship.expand_writing_area(application, window, {'bible_relationship': observed})
    ui.reveal_button(application, 'Preview update', window)
    pending = ui.wait_for('name preview persisted', lambda: next((r for r in relationship.proposals(database, b[0]) if r[1] == 'pending' and r[2] == REPLACEMENT), None))
    first, input_name = receipt(database, pending[0])
    if input_name['revision_event_id'] != changed[1] or ui.blocks(database, b_id)[0] != b:
        raise RuntimeError('Preview rebound name or changed saved text')
    region = ui.wait_for('exact native name proposal', lambda: relationship.review_region(application, REPLACEMENT))
    disclosure = ui.wait_for('name receipt disclosure', lambda: ui.reveal(region, lambda n: n.name == 'Names used for this preview'))
    disclosure.queryComponent().scrollTo(ui.pyatspi.SCROLL_TOP_LEFT)
    ui.click_control(disclosure, window)
    ui.wait_for('original name revision visible', lambda: ui.reveal(region, lambda n: changed[1] in ui.text_of(n)))
    observed['original_preview_binding'] = first
    checkpoint('targeted name preview with exact receipts; explicit acceptance required')
    capture('eidetic-name-preview.png')
    restored = edit_name(application, window, database, changed, BEFORE)
    again = edit_name(application, window, database, restored, AFTER)
    before = relationship.base.recovery.database_snapshot(database)
    region = ui.wait_for('original preview after name ABA', lambda: relationship.review_region(application, REPLACEMENT))
    ui.reveal_button(region, 'Accept update', window)
    ui.wait_for('exact stale name acceptance refusal', lambda: relationship.stale_alert(application))
    after = relationship.base.recovery.database_snapshot(database)
    relationship.base.require_refusal_unchanged(before, after)
    observed.update(aba_name=list(again), stale_refusal=relationship.STALE, stale_aba_refused_without_writes=True, database_before_refusal=before, database_after_refusal=after)
    capture('eidetic-name-stale.png')
    region = ui.wait_for('stale pending proposal', lambda: relationship.review_region(application, REPLACEMENT))
    ui.reveal_button(region, 'Reject', window)
    ui.wait_for('old preview explicitly rejected', lambda: (pending[0], 'rejected', REPLACEMENT) in relationship.proposals(database, b[0]))
    ui.reveal_button(application, 'Preview update', window)
    fresh = ui.wait_for('fresh name preview', lambda: next((r for r in relationship.proposals(database, b[0]) if r[0] != pending[0] and r[1] == 'pending' and r[2] == REPLACEMENT), None))
    latest, current = receipt(database, fresh[0])
    if current['revision_event_id'] != again[1] or ui.blocks(database, b_id)[0] != b:
        raise RuntimeError('Fresh name preview changed canon or lacks exact read')
    observed['fresh_preview_binding'] = latest
    capture('eidetic-name-fresh-preview.png')
    region = ui.wait_for('fresh pending name proposal', lambda: relationship.review_region(application, REPLACEMENT))
    ui.reveal_button(region, 'Accept update', window)
    saved = ui.wait_for('explicit accepted name screenplay', lambda: next((r for r in ui.blocks(database, b_id) if r[1] == REPLACEMENT and r[2] != b[2]), None))
    if ui.blocks(database, a_id)[0] != a or name(database) != again:
        raise RuntimeError('Acceptance changed manual source or Bible name')
    bindings = ui.query(database, "SELECT r.target_revision_event_id FROM semantic_dependencies d JOIN semantic_dependency_revisions r ON r.dependency_id=d.id WHERE d.target_kind='bible_node' AND d.target_id=? AND r.source_revision_event_id=?", (NODE_ID, saved[2]))
    if bindings != [(again[1],)]:
        raise RuntimeError('Acceptance did not refresh exact name consumption')
    def cleared():
        application.clear_cache()
        return not any(n.name == 'Screenplay needs review' for n in ui.walk(application))
    ui.wait_for('name review cleared after explicit acceptance', cleared)
    ui.wait_for('accepted exact name screenplay visible', lambda: ui.screenplay_block(application, REPLACEMENT.splitlines()[-1]))
    observed.update(status='native_name_edit_review_aba_acceptance_passed', accepted_revision=saved[2], refreshed_consumed_name_revision=again[1], manual_source_unchanged=True)
    checkpoint('explicit acceptance replaces only targeted screenplay and refreshes name receipts')
    capture('eidetic-name-accepted.png')


if __name__ == '__main__':
    if not os.environ.get('EIDETIC_CAPTURE_SOURCE'):
        raise SystemExit('An exact frozen EIDETIC_CAPTURE_SOURCE is required')
    bible.BibleFactProvider = NameProvider
    bible.AFTER_ACCEPTANCE = name_flow
    bible.main()
