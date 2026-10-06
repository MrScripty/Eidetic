#!/usr/bin/env python3
"""Actual native Bible-only plan drift, historical evidence and explicit review.

Reuse normal public-service setup and native Bible/save/targeted screenplay flow.
All subsequent input is AT-SPI/X11; all database observations are read-only.
The loopback HTTP responses are synthetic and do not qualify model quality.
"""
import importlib.util
import io
import json
import os
from pathlib import Path

spec = importlib.util.spec_from_file_location(
    'recovery_capture', Path(__file__).with_name('qualify-child-plan-recovery.py'))
recovery = importlib.util.module_from_spec(spec)
spec.loader.exec_module(recovery)
bible = recovery.memory.bible
ui = bible.ui
GOLD = "Mara's umbrella is gold."
GOLD_SCREENPLAY = 'EXT. STATION - NIGHT\n\nEli spots Mara and her gold umbrella.'
STALE = 'Child plan story context changed; generate and review a fresh plan before accepting'
EDGE = 'Mara trusts Eli'


def expected_context(user, fact):
    return (f'profile.tagline: {fact}' in user and ui.MANUAL_TEXT in user
            and ui.PROPOSED_TEXT in user and EDGE in user
            and 'qualification.mara' in user and 'qualification.eli' in user)


class BiblePlanProvider(bible.BibleFactProvider):
    records = []

    def do_POST(self):
        size = int(self.headers.get('Content-Length', '0'))
        if self.path != '/v1/chat/completions' or not 0 < size <= 512_000:
            self.send_error(400)
            return
        data = self.rfile.read(size)
        body = json.loads(data)
        user = next((m['content'] for m in body.get('messages', []) if m.get('role') == 'user'), '')
        gold = f'profile.tagline: {GOLD}' in user
        stream = body.get('stream') is True
        if stream and not gold:
            original = self.rfile
            self.rfile = io.BytesIO(data)
            try:
                super().do_POST()
            finally:
                self.rfile = original
            return
        kind = 'preview_gold' if stream else 'child_gold' if gold else 'child_blue'
        system = next((m['content'] for m in body.get('messages', []) if m.get('role') == 'system'), '')
        valid = expected_context(user, GOLD if gold else bible.BLUE)
        if stream:
            valid = valid and 'targeted screenplay update' in system
        with self.records_lock:
            accepted = valid and not any(r['kind'] == kind and r.get('accepted') for r in self.records)
            self.records.append({'kind': kind, 'accepted': accepted,
                'contains_exact_expected_context': valid, 'stream': stream, 'real_model': False})
        if not accepted:
            self.send_error(422, 'Exact canonical screenplay, Bible fact or relationship missing')
            return
        if stream:
            text = GOLD_SCREENPLAY
            data = ''.join('data: ' + json.dumps({'choices': [{'delta': {'content': part}}]}) + '\n\n'
                for part in (text[:len(text)//2], text[len(text)//2:])).encode() + b'data: [DONE]\n\n'
            content_type = 'text/event-stream'
        else:
            color = 'Gold' if gold else 'Blue'
            children = [{'name': f' {color} departure ',
                'outline': f'Mara takes her {color.lower()} umbrella to Eli.\n',
                'weight': 1.0, 'location': '', 'characters': [' Mara ', ' Eli ', ' '],
                'props': ['', ' Umbrella ', ' ']}]
            data = json.dumps({'choices': [{'message': {'content': json.dumps(children)}}]}).encode()
            content_type = 'application/json'
        self.send_response(200)
        self.send_header('Content-Type', content_type)
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)


def stored_memory(database, plan_id):
    rows = ui.query(database,
        "SELECT c.payload_json FROM child_plans p JOIN change_events e ON e.id=p.created_event_id "
        "JOIN commands c ON c.id=e.command_id WHERE p.id=? AND c.payload_type='ai.child_plan_create'",
        (plan_id,))
    if len(rows) != 1:
        raise RuntimeError('Expected one original pending-plan receipt')
    memory = json.loads(rows[0][0]).get('memory')
    if not isinstance(memory, dict) or not isinstance(memory.get('bible'), dict):
        raise RuntimeError('Missing original Bible custody')
    return memory


def require_refusal_unchanged(before, after):
    if before != after:
        raise RuntimeError('Refused Bible-stale plan changed canonical state or history')


def saved_script_and_placement(database):
    return {'blocks': ui.query(database, 'SELECT * FROM script_blocks ORDER BY id'),
        'segments': ui.query(database, 'SELECT * FROM script_segments ORDER BY id'),
        'nodes': ui.query(database, 'SELECT * FROM nodes ORDER BY id')}


def native_receipt(application, window, value, revision):
    review = ui.wait_for('original child review', lambda: ui.reveal(application,
        lambda n: n.name == 'Review proposed timeline children'))
    # Replacing the pending plan can preserve the existing native details state.
    # Open only when its recorded region is absent, never blindly toggle closed.
    if ui.reveal(review, lambda n: n.name == 'Recorded Bible evidence') is None:
        summary = ui.wait_for('Bible source disclosure', lambda: ui.reveal(review,
            lambda n: n.name == 'Bible facts used for this plan'))
        ui.click_control(summary, window)
    region = ui.wait_for('recorded Bible evidence region', lambda: ui.reveal(review,
        lambda n: n.name == 'Recorded Bible evidence'))
    for expected in (value, revision, EDGE, 'Mara', 'Eli'):
        ui.wait_for('visible original Bible evidence ' + expected, lambda expected=expected: ui.reveal(
            region, lambda n: expected in ui.text_of(n)))
    return {'region': region.name, 'exact_value': value, 'field_revision_event_id': revision,
        'relationship_label': EDGE, 'fictional_time': 'unspecified',
        'source': 'original saved receipt; no current Bible hydration'}


def readable_gold_screenplay(application, window, evidence):
    geometry = {key: int(value) for key, value in (line.split('=', 1) for line in
        ui.command('xdotool', 'getwindowgeometry', '--shell', window).splitlines())}
    def divider():
        return ui.find(application, lambda n: n.getRole() == ui.pyatspi.ROLE_PUSH_BUTTON
            and n.name == 'Resize panels' and recovery.memory.is_editor_divider(
                tuple(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)), geometry['WIDTH'], geometry['HEIGHT']))
    control = ui.wait_for('existing editor panel resize control', divider)
    before = tuple(control.queryComponent().getExtents(ui.pyatspi.XY_SCREEN))
    ui.click_control(control, window)
    ui.command('xdotool', 'key', '--clearmodifiers', *(['Up'] * 7))
    after = ui.wait_for('supported expanded writing area', lambda: tuple(divider().queryComponent().getExtents(
        ui.pyatspi.XY_SCREEN)) if divider() and divider().queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[1] < before[1] else None)
    saved = ui.wait_for('saved Gold block', lambda: ui.screenplay_block(application,
        'Eli spots Mara and her gold umbrella.'))
    saved.queryComponent().scrollTo(ui.pyatspi.SCROLL_TOP_LEFT)
    def exact_ranges():
        application.clear_cache()
        bottom_control = ui.find(application, lambda n: n.getRole() == ui.pyatspi.ROLE_PUSH_BUTTON
            and n.name == 'Resize panels' and n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[2] >= geometry['WIDTH'] - 10)
        if bottom_control is None:
            return None
        frame = (after[0], after[1] + after[3], after[0] + after[2],
            bottom_control.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[1])
        block = next((n for n in ui.walk(application) if n.name == 'Screenplay block'
            and any('Eli spots Mara and her gold umbrella.' in ui.text_of(child) for child in ui.walk(n))), None)
        if block is None:
            return None
        ranges = {}
        for node in ui.walk(block):
            value = ui.text_of(node).strip()
            if value not in GOLD_SCREENPLAY.splitlines() or not value or not ui.visible(node):
                continue
            try:
                bounds = tuple(node.queryText().getRangeExtents(0, len(value), ui.pyatspi.XY_SCREEN))
            except NotImplementedError:
                continue
            if recovery.memory.inside_writing_area(bounds, frame):
                ranges[value] = list(bounds)
        return {'bounds': list(frame), 'saved_text_ranges': ranges} if len(ranges) == 2 else None
    observed = ui.wait_for('both exact Gold screenplay lines inside native writing area', exact_ranges)
    evidence['bible_child_plan']['readable_final_screenplay'] = {
        'supported_actions': ['Resize panels keyboard Up', 'AT-SPI SCROLL_TOP_LEFT'],
        'editor_divider_before': list(before), 'editor_divider_after': list(after), **observed}


def child_flow(application, window, database, fixture, evidence, checkpoint, capture):
    parent = fixture['b']['id']
    original_fact = bible.fact(database)
    baseline = saved_script_and_placement(database)
    if original_fact[0] != bible.BLUE:
        raise RuntimeError('Expected already accepted Blue screenplay/Bible flow')

    def plans():
        return ui.query(database, 'SELECT id,status FROM child_plans WHERE parent_node_id=? ORDER BY rowid', (parent,))

    def generate():
        control = ui.wait_for('child planning control', lambda: ui.reveal(application,
            lambda n: n.getRole() == ui.pyatspi.ROLE_PUSH_BUTTON and n.name in ('Plan Beats', 'Replan Beats')
            and n.getState().contains(ui.pyatspi.STATE_ENABLED)))
        ui.click_control(control, window)

    def review_child(name):
        return ui.reveal(application, lambda n: n.name == 'Review proposed timeline children'
            and any(name in ui.text_of(child) for child in ui.walk(n)))

    # Inspect the seeded relationship in the actual existing Bible detail UI.
    # Setup is explicitly public-service authored; this is native inspection.
    ui.wait_for('Bible relationship label', lambda: ui.reveal(application,
        lambda n: ui.text_of(n).strip() == EDGE))
    evidence['bible_graph_inspection'] = {**fixture['relationship_setup'],
        'native_label_inspected': True, 'native_edge_authoring': False}
    checkpoint('existing graph relationship inspected in native Bible detail')
    capture('eidetic-bible-relationship-inspected.png')

    generate()
    initial = ui.wait_for('Blue pending plan', lambda: next((r for r in plans() if r[1] == 'pending'), None))
    ui.wait_for('Blue child proposal visible', lambda: review_child('Blue departure'))
    receipt = stored_memory(database, initial[0])
    original_evidence = native_receipt(application, window, bible.BLUE, original_fact[1])
    if saved_script_and_placement(database) != baseline:
        raise RuntimeError('Pending plan changed saved screenplay or placement')
    evidence['bible_child_plan'] = {'initial_plan_id': initial[0], 'original_receipt': receipt,
        'original_evidence_visible': original_evidence, 'pending_preserved_script_and_placement': True,
        'real_model': False}
    checkpoint('review original Blue facts and graph relationship before acceptance')
    capture('eidetic-bible-plan-blue-evidence.png')

    # Close and recover the same saved proposal, qualifying historical custody.
    ui.reveal_button(application, 'Close preview', window)
    ui.wait_for('child preview closed', lambda: not any(n.name == 'Review proposed timeline children' for n in ui.walk(application)))
    before_read = recovery.database_snapshot(database)
    requests_before = list(BiblePlanProvider.records)
    ui.reveal_button(application, 'Review saved timeline plans', window)
    ui.reveal_button(application, 'Review plan 1', window)
    ui.wait_for('recovered Blue proposal visible', lambda: review_child('Blue departure'))
    native_receipt(application, window, bible.BLUE, original_fact[1])
    recovery.require_read_only_recovery(before_read, recovery.database_snapshot(database),
        requests_before, list(BiblePlanProvider.records))
    evidence['bible_child_plan']['recovery_retained_original_evidence_without_writes_or_provider_call'] = True

    field = ui.wait_for('current Blue Bible field', lambda: ui.reveal(application,
        lambda n: n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n) == bible.BLUE))
    ui.type_text(field, window, GOLD)
    save = ui.wait_for('associated Bible fact Save', lambda: bible.save_fact_control(application, field))
    ui.click_control(save, window)
    edited = ui.wait_for('exact canonical Gold fact', lambda: bible.fact(database)
        if bible.fact(database)[0] == GOLD and bible.fact(database)[1] != original_fact[1] else None)
    if saved_script_and_placement(database) != baseline:
        raise RuntimeError('Bible-only edit changed screenplay, placement or their revisions')
    ui.wait_for('downstream screenplay review after Gold edit', lambda: ui.reveal(application,
        lambda n: 'Bible fact profile.tagline changed.' in ui.text_of(n)))
    # Already-open disclosure is historical: reveal its original value without toggling.
    ui.wait_for('original Blue fact remains in pending review', lambda: ui.reveal(application,
        lambda n: n.name == 'Recorded Bible evidence'
        and any(bible.BLUE in ui.text_of(child) for child in ui.walk(n))))
    evidence['bible_child_plan']['manual_bible_only_edit'] = {'before': bible.BLUE, 'after': GOLD,
        'revision_event_id': edited[1], 'all_saved_scripts_segments_and_nodes_unchanged': True}
    checkpoint('manual Gold fact; original Blue plan evidence and downstream review retained')
    capture('eidetic-bible-plan-gold-review.png')

    before_refusal = recovery.database_snapshot(database)
    ui.reveal_button(application, 'Accept timeline plan', window)
    ui.wait_for('exact Bible-stale acceptance refusal', lambda: ui.reveal(application,
        lambda n: ui.text_of(n).strip() == STALE))
    after_refusal = recovery.database_snapshot(database)
    require_refusal_unchanged(before_refusal, after_refusal)
    if plans()[0] != initial:
        raise RuntimeError('Stale plan did not stay pending')
    evidence['bible_child_plan'].update(stale_refusal=STALE,
        database_before_refusal=before_refusal, database_after_refusal=after_refusal,
        bible_only_stale_acceptance_refused_without_writes=True)
    checkpoint('Bible-only drift refuses old timeline acceptance with no SQLite writes')
    capture('eidetic-bible-plan-refused.png')

    generate()
    fresh = ui.wait_for('fresh Gold pending plan', lambda: next((r for r in plans()
        if r[0] != initial[0] and r[1] == 'pending'), None))
    ui.wait_for('Gold child proposal visible', lambda: review_child('Gold departure'))
    fresh_receipt = stored_memory(database, fresh[0])
    native_receipt(application, window, GOLD, edited[1])
    evidence['bible_child_plan']['fresh_original_receipt'] = fresh_receipt
    checkpoint('fresh Gold facts and relationships in reviewable targeted timeline proposal')
    capture('eidetic-bible-plan-gold-evidence.png')
    before_accept = {key: baseline[key] for key in ('blocks', 'segments')}
    ui.reveal_button(application, 'Accept timeline plan', window)
    ui.wait_for('explicitly accepted Gold timeline plan', lambda: (fresh[0], 'applied') in plans())
    now = saved_script_and_placement(database)
    if {key: now[key] for key in ('blocks', 'segments')} != before_accept or bible.fact(database) != edited:
        raise RuntimeError('Timeline acceptance rewrote screenplay or Bible canon')
    rendered = ui.wait_for('accepted Gold departure timeline clip', lambda:
        ui.native_timeline_clip(application, 'Gold departure', window))
    ui.wait_for('separate screenplay review remains after timeline acceptance', lambda: ui.reveal(application,
        lambda n: n.name == 'Screenplay needs review'))
    evidence['bible_child_plan'].update(fresh_plan_id=fresh[0], fresh_explicit_timeline_acceptance=True,
        canonical_screenplay_and_bible_preserved=True, screenplay_review_remained=True,
        accepted_child_native_bounds=list(rendered[1]))
    checkpoint('explicit Gold timeline acceptance preserves saved Blue screenplay and its review')
    capture('eidetic-bible-plan-accepted-review.png')

    # Complete the independent existing screenplay review with its own acceptance.
    ui.reveal_button(application, 'Preview update', window)
    proposal = ui.wait_for('Gold targeted screenplay preview', lambda: next((r for r in ui.query(database,
        'SELECT id,status,proposed_text FROM propagation_proposals WHERE proposed_text=?', (GOLD_SCREENPLAY,))
        if r[1] == 'pending'), None))
    if ui.blocks(database, parent)[0][1] != ui.PROPOSED_TEXT:
        raise RuntimeError('Gold preview replaced saved Blue screenplay')
    ui.wait_for('exact Gold proposed screenplay visible', lambda: ui.reveal(application,
        lambda n: n.name == 'Proposed text' and ui.text_of(n) == GOLD_SCREENPLAY))
    checkpoint('independent Gold screenplay preview; saved Blue text retained')
    capture('eidetic-bible-gold-screenplay-preview.png')
    ui.reveal_button(application, 'Accept update', window)
    accepted = ui.wait_for('explicitly accepted Gold screenplay', lambda: next((r for r in ui.blocks(database, parent)
        if r[1] == GOLD_SCREENPLAY), None))
    if ui.blocks(database, fixture['a']['id'])[0][1] != ui.MANUAL_TEXT or bible.fact(database) != edited:
        raise RuntimeError('Targeted Gold acceptance changed manual source or Bible')
    binding = ui.query(database, 'SELECT r.target_revision_event_id FROM semantic_dependencies d '
        'JOIN semantic_dependency_revisions r ON r.dependency_id=d.id '
        "WHERE d.target_field_id='qualification.mara.tagline' AND r.source_revision_event_id=?", (accepted[2],))
    if binding != [(edited[1],)]:
        raise RuntimeError('Gold screenplay acceptance omitted fresh fact provenance')
    ui.wait_for('Gold saved screenplay visible', lambda: ui.screenplay_block(application, 'Eli spots Mara and her gold umbrella.'))
    ui.wait_for('Gold review cleared after explicit screenplay acceptance', lambda:
        not any(n.name == 'Screenplay needs review' for n in ui.walk(application)))
    evidence['bible_child_plan'].update(status='native_bible_child_plan_passed_with_synthetic_http_fixture',
        separate_gold_screenplay_preview_id=proposal[0], gold_screenplay_revision=accepted[2],
        independent_explicit_screenplay_acceptance=True, exact_manual_source_preserved=True,
        gold_screenplay_fact_binding=edited[1])
    checkpoint('separate explicit Gold screenplay acceptance refreshes provenance')
    capture('eidetic-bible-gold-screenplay-accepted.png')
    readable_gold_screenplay(application, window, evidence)
    ui.wait_for('Gold Bible field remains visible', lambda: ui.reveal(application,
        lambda n: n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n) == GOLD))
    ui.wait_for('accepted Gold departure still visible', lambda: ui.native_timeline_clip(application,
        'Gold departure', window))
    checkpoint('readable saved Gold screenplay with authored Bible and accepted timeline')
    capture('eidetic-bible-story-memory-readable.png')


if __name__ == '__main__':
    if not os.environ.get('EIDETIC_CAPTURE_SOURCE'):
        raise SystemExit('An exact frozen EIDETIC_CAPTURE_SOURCE is required')
    bible.BibleFactProvider = BiblePlanProvider
    bible.AFTER_ACCEPTANCE = child_flow
    bible.main()
