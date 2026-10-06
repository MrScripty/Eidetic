#!/usr/bin/env python3
"""Actual Tauri relationship label edit, affected review and explicit acceptance.

Reuse the maintained Bible/save walkthrough and native AT-SPI/X11 controls.
SQLite observations are read-only; loopback model responses are synthetic.
"""
import importlib.util
import io
import json
import os
from pathlib import Path

spec = importlib.util.spec_from_file_location(
    'relationship_base', Path(__file__).with_name('qualify-child-plan-bible.py'))
base = importlib.util.module_from_spec(spec)
spec.loader.exec_module(base)
bible = base.bible
ui = bible.ui
EDGE_ID = 'qualification.mara.eli'
TRUST = 'Mara trusts Eli'
DOUBT = 'Mara doubts Eli'
REPLACEMENT = 'EXT. STATION - NIGHT\n\nMara doubts Eli as they board the train.'
STALE = 'screenplay proposal is stale; request a fresh preview'


def relationship_prompt(user, system):
    return ('targeted screenplay update' in system and '"kind":"bible_edge"' in user
            and TRUST in user and DOUBT in user and EDGE_ID in user
            and f'profile.tagline: {bible.BLUE}' in user
            and ui.MANUAL_TEXT in user and ui.PROPOSED_TEXT in user)


class RelationshipProvider(bible.BibleFactProvider):
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
        if '"kind":"bible_edge"' not in user:
            original = self.rfile
            self.rfile = io.BytesIO(data)
            try:
                super().do_POST()
            finally:
                self.rfile = original
            return
        valid = body.get('stream') is True and relationship_prompt(user, system)
        with self.records_lock:
            count = sum(r.get('kind', '').startswith('preview_relationship_') and r.get('accepted') for r in self.records)
            accepted = valid and count < 2
            self.records.append({'kind': f'preview_relationship_{count + 1}', 'accepted': accepted,
                'contains_exact_expected_context': valid, 'stream': True, 'real_model': False})
        if not accepted:
            self.send_error(422, 'Exact canonical relationship and screenplay evidence missing')
            return
        data = ('data: ' + json.dumps({'choices': [{'delta': {'content': REPLACEMENT}}]})
                + '\n\ndata: [DONE]\n\n').encode()
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)


def edge(database):
    rows = ui.query(database, 'SELECT id,from_node_id,to_node_id,edge_kind,label,directed,sort_order,updated_event_id '
                    'FROM bible_graph_edges WHERE id=? AND deleted_event_id IS NULL', (EDGE_ID,))
    if len(rows) != 1:
        raise RuntimeError('Expected exactly one fixture relationship')
    return rows[0]


def edit_label(application, window, database, before, label):
    ui.reveal_button(application, 'Edit relationship label ' + before[4], window)
    field = ui.wait_for('native relationship label editor', lambda: ui.reveal(application,
        lambda n: n.getState().contains(ui.pyatspi.STATE_EDITABLE) and n.name == 'Relationship label ' + EDGE_ID))
    ui.type_text(field, window, label)
    ui.reveal_button(application, 'Save relationship label', window)
    saved = ui.wait_for('exact saved relationship label and new revision', lambda:
        edge(database) if edge(database)[4] == label and edge(database)[7] != before[7] else None)
    if saved[:4] != before[:4] or saved[5:7] != before[5:7]:
        raise RuntimeError('Label edit changed endpoints, kind, direction or ordering')
    return saved


def proposals(database, block_id):
    return ui.query(database, 'SELECT id,status,proposed_text FROM propagation_proposals WHERE target_id=? ORDER BY rowid', (block_id,))


def receipt(database, proposal_id):
    rows = ui.query(database, 'SELECT binding_json FROM script_impact_proposal_bindings WHERE proposal_id=?', (proposal_id,))
    if len(rows) != 1:
        raise RuntimeError('Expected one durable relationship preview')
    value = json.loads(rows[0][0])
    inputs = value.get('bible_relationship_inputs')
    if not isinstance(inputs, list) or len(inputs) != 1 or inputs[0]['edge']['edge_id'] != EDGE_ID:
        raise RuntimeError('Preview lacks exact relationship custody')
    return value


def review_region(application, proposal_text):
    return ui.reveal(application, lambda n: n.name == 'Screenplay update proposals'
        and any(child.name == 'Proposed text' and ui.text_of(child) == proposal_text for child in ui.walk(n)))


def begin_manual_draft(application, window):
    block = ui.wait_for('manual source block', lambda: ui.screenplay_block(application, 'red umbrella'))
    # The accepted target can scroll the manual card horizontally out of view.
    # Reveal this exact card's button through native AT-SPI before pointer input.
    ui.reveal_button(block, 'Edit', window)
    draft = ui.wait_for('manual source editor', lambda: ui.editable(application, ui.MANUAL_TEXT))
    ui.type_text(draft, window, bible.DRAFT)


def readable_saved(application, window, evidence):
    geometry = {key: int(value) for key, value in (line.split('=', 1) for line in
        ui.command('xdotool', 'getwindowgeometry', '--shell', window).splitlines())}
    control = ui.wait_for('existing panel divider', lambda: ui.find(application,
        lambda n: n.getRole() == ui.pyatspi.ROLE_PUSH_BUTTON and n.name == 'Resize panels'
        and base.recovery.memory.is_editor_divider(tuple(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)),
            geometry['WIDTH'], geometry['HEIGHT'])))
    before = tuple(control.queryComponent().getExtents(ui.pyatspi.XY_SCREEN))
    ui.click_control(control, window)
    ui.command('xdotool', 'key', '--clearmodifiers', *(['Up'] * 7))
    block = ui.wait_for('saved relationship screenplay', lambda: ui.screenplay_block(application, REPLACEMENT.splitlines()[-1]))
    block.queryComponent().scrollTo(ui.pyatspi.SCROLL_TOP_LEFT)
    ui.wait_for('saved relationship still in Bible detail', lambda: ui.reveal(application,
        lambda n: n.getRole() == ui.pyatspi.ROLE_PUSH_BUTTON and n.name == 'Edit relationship label ' + DOUBT))
    evidence['bible_relationship']['readable_final_actions'] = ['Resize panels keyboard Up', 'AT-SPI SCROLL_TOP_LEFT']
    evidence['bible_relationship']['divider_before'] = list(before)


def relationship_flow(application, window, database, fixture, evidence, checkpoint, capture):
    a_id, b_id = fixture['a']['id'], fixture['b']['id']
    a, b = ui.blocks(database, a_id)[0], ui.blocks(database, b_id)[0]
    original = edge(database)
    if original[4] != TRUST or b[1] != ui.PROPOSED_TEXT:
        raise RuntimeError('Unexpected relationship-flow starting canon')
    dependencies = ui.query(database, 'SELECT d.id,r.target_revision_event_id FROM semantic_dependencies d '
        'JOIN semantic_dependency_revisions r ON r.dependency_id=d.id WHERE d.target_kind=\'bible_edge\' '
        'AND d.target_id=? AND r.source_revision_event_id=?', (EDGE_ID, b[2]))
    if len(dependencies) != 1 or dependencies[0][1] != original[7]:
        raise RuntimeError('Accepted screenplay did not bind its actually supplied relationship')
    evidence['bible_relationship'] = {'original_edge': list(original), 'consumed_dependency': dependencies[0][0],
        'consumed_revision': dependencies[0][1], 'real_model': False}
    begin_manual_draft(application, window)
    baseline = base.saved_script_and_placement(database)
    checkpoint('native exact relationship label edit; saved screenplay and draft retained')
    changed = edit_label(application, window, database, original, DOUBT)
    if base.saved_script_and_placement(database) != baseline:
        raise RuntimeError('Relationship edit changed saved screenplay or placement')
    ui.wait_for('manual draft preserved after relationship refresh', lambda: ui.editable(application, bible.DRAFT))
    ui.wait_for('relationship-specific downstream review', lambda: ui.reveal(application,
        lambda n: 'Bible relationship changed.' in ui.text_of(n)))
    evidence['bible_relationship'].update(edited_edge=list(changed), saved_scripts_and_placement_preserved=True,
        exact_manual_draft_preserved=True)
    capture('eidetic-relationship-review.png')
    # Explicitly cancel the healthy manual draft after proving refresh preservation.
    block = ui.wait_for('retained draft block', lambda: ui.screenplay_block(application, 'Retained manual draft'))
    ui.reveal_button(block, 'Cancel', window)
    ui.reveal_button(application, 'Preview update', window)
    pending = ui.wait_for('relationship preview persisted', lambda: next((r for r in proposals(database, b[0])
        if r[1] == 'pending' and r[2] == REPLACEMENT), None))
    first = receipt(database, pending[0])
    if first['bible_relationship_inputs'][0]['revision_event_id'] != changed[7] or ui.blocks(database, b_id)[0] != b:
        raise RuntimeError('Preview changed canon or rebound relationship custody')
    region = ui.wait_for('relationship preview native review', lambda: review_region(application, REPLACEMENT))
    disclosure = ui.wait_for('original preview relationship disclosure', lambda: ui.reveal(region,
        lambda n: n.name == 'Relationships used for this preview'))
    ui.click_control(disclosure, window)
    ui.wait_for('exact original preview receipt visible', lambda: ui.reveal(region,
        lambda n: changed[7] in ui.text_of(n)))
    evidence['bible_relationship']['original_preview_binding'] = first
    checkpoint('pending targeted relationship preview; explicit acceptance required')
    capture('eidetic-relationship-preview.png')
    restored = edit_label(application, window, database, changed, TRUST)
    again = edit_label(application, window, database, restored, DOUBT)
    if ui.blocks(database, a_id)[0] != a or ui.blocks(database, b_id)[0] != b:
        raise RuntimeError('Relationship ABA changed saved screenplay')
    before = base.recovery.database_snapshot(database)
    # This old preview is the sole pending proposal; explicit native acceptance must refuse it.
    region = ui.wait_for('original preview after relationship ABA', lambda: review_region(application, REPLACEMENT))
    ui.reveal_button(region, 'Accept update', window)
    ui.wait_for('exact stale relationship acceptance refusal', lambda: ui.reveal(application,
        lambda n: STALE in ui.text_of(n)))
    after = base.recovery.database_snapshot(database)
    base.require_refusal_unchanged(before, after)
    if (pending[0], 'pending', REPLACEMENT) not in proposals(database, b[0]):
        raise RuntimeError('Stale relationship preview lost pending custody')
    evidence['bible_relationship'].update(aba_edge=list(again), stale_refusal=STALE,
        database_before_refusal=before, database_after_refusal=after, stale_aba_refused_without_writes=True)
    checkpoint('native relationship edit-and-restore refuses old preview with no writes')
    capture('eidetic-relationship-stale.png')
    # Reject only the obsolete preview, then request a fresh explicit review.
    region = ui.wait_for('original refused preview', lambda: review_region(application, REPLACEMENT))
    ui.reveal_button(region, 'Reject', window)
    ui.wait_for('obsolete preview explicitly rejected', lambda: (pending[0], 'rejected', REPLACEMENT) in proposals(database, b[0]))
    ui.reveal_button(application, 'Preview update', window)
    fresh = ui.wait_for('fresh relationship preview', lambda: next((r for r in proposals(database, b[0])
        if r[0] != pending[0] and r[1] == 'pending' and r[2] == REPLACEMENT), None))
    latest = receipt(database, fresh[0])
    if latest['bible_relationship_inputs'][0]['revision_event_id'] != again[7] or ui.blocks(database, b_id)[0] != b:
        raise RuntimeError('Fresh relationship preview lacks original current read or replaced canon')
    evidence['bible_relationship']['fresh_preview_binding'] = latest
    checkpoint('fresh relationship revision in review; saved screenplay unchanged')
    capture('eidetic-relationship-fresh-preview.png')
    ui.reveal_button(application, 'Accept update', window)
    saved = ui.wait_for('explicit accepted relationship screenplay', lambda: next((r for r in ui.blocks(database, b_id)
        if r[1] == REPLACEMENT and r[2] != b[2]), None))
    if ui.blocks(database, a_id)[0] != a or edge(database) != again:
        raise RuntimeError('Acceptance altered manual screenplay or relationship canon')
    bindings = ui.query(database, 'SELECT r.target_revision_event_id FROM semantic_dependencies d '
        'JOIN semantic_dependency_revisions r ON r.dependency_id=d.id WHERE d.target_kind=\'bible_edge\' '
        'AND d.target_id=? AND r.source_revision_event_id=?', (EDGE_ID, saved[2]))
    if bindings != [(again[7],)]:
        raise RuntimeError('Acceptance did not refresh exact relationship consumption')
    def review_cleared():
        application.clear_cache()
        return not any(n.name == 'Screenplay needs review' for n in ui.walk(application))
    ui.wait_for('relationship needs-review cleared', review_cleared)
    evidence['bible_relationship'].update(status='native_relationship_review_passed_with_synthetic_provider',
        saved_screenplay_revision=saved[2], refreshed_relationship_revision=again[7],
        explicit_acceptance=True, exact_manual_source_preserved=True)
    checkpoint('explicit relationship acceptance replaces only reviewed screenplay')
    capture('eidetic-relationship-accepted.png')
    readable_saved(application, window, evidence)
    checkpoint('native authored relationship, timeline and saved screenplay visible')
    capture('eidetic-relationship-readable.png')


if __name__ == '__main__':
    if not os.environ.get('EIDETIC_CAPTURE_SOURCE'):
        raise SystemExit('An exact EIDETIC_CAPTURE_SOURCE is required')
    bible.BibleFactProvider = RelationshipProvider
    bible.AFTER_ACCEPTANCE = relationship_flow
    bible.main()
