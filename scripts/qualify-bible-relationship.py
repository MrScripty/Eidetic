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


def pending_review(node, proposal_text):
    children = list(ui.walk(node))
    texts = [child for child in children if child.name == 'Proposed text']
    controls = [child.name for child in children
        if child.getRole() == ui.pyatspi.ROLE_PUSH_BUTTON
        and child.getState().contains(ui.pyatspi.STATE_ENABLED)]
    return (len(texts) == 1 and ui.text_of(texts[0]) == proposal_text
        and controls.count('Accept update') == 1 and controls.count('Reject') == 1)


def review_region(application, proposal_text):
    section = ui.reveal(application, lambda n: n.name == 'Screenplay update proposals'
        and any(child.name == 'Proposed text' and ui.text_of(child) == proposal_text for child in ui.walk(n)))
    # The section also retains accepted/rejected history. Scope the disclosure
    # and decision controls to the article owning this exact pending text.
    return ui.reveal(section, lambda n: pending_review(n, proposal_text)) if section else None


def stale_alert(application, observations=None):
    # Accessible leaves implement __len__ using child count. Never use their
    # truthiness: a real, showing zero-child error can otherwise be discarded.
    application.clear_cache()
    matches = []
    visible_exact = False
    for node in ui.walk(application):
        if ui.text_of(node) != STALE:
            continue
        component = node.queryComponent()
        component.scrollTo(ui.pyatspi.SCROLL_ANYWHERE)
        node.clear_cache()
        showing = ui.visible(node)
        matches.append({'role': node.getRoleName(), 'role_id': node.getRole(),
                        'child_count': node.childCount, 'showing': showing,
                        'bounds': list(component.getExtents(ui.pyatspi.XY_SCREEN)),
                        'exact_text': ui.text_of(node)})
        visible_exact = visible_exact or showing
    if observations is not None and matches and (not observations or observations[-1] != matches):
        observations.append(matches)
        print('Exact stale accessibility: ' + json.dumps(matches), flush=True)
    # wait_for also tests truthiness, so return a boolean observation, not a leaf.
    return visible_exact


def begin_manual_draft(application, window):
    block = ui.wait_for('manual source block', lambda: ui.screenplay_block(application, 'red umbrella'))
    # The accepted target can scroll the manual card horizontally out of view.
    # Reveal this exact card's button through native AT-SPI before pointer input.
    ui.reveal_button(block, 'Edit', window)
    draft = ui.wait_for('manual source editor', lambda: ui.editable(application, ui.MANUAL_TEXT))
    ui.type_text(draft, window, bible.DRAFT)


def cancel_manual_draft(application, window):
    ui.wait_for('exact retained manual draft before explicit cancellation',
                lambda: ui.reveal(application, lambda n:
                    n.getState().contains(ui.pyatspi.STATE_EDITABLE)
                    and ui.text_of(n) == bible.DRAFT))
    application.clear_cache()
    controls = [n for n in ui.walk(application) if n.getRole() == ui.pyatspi.ROLE_PUSH_BUTTON
                and n.name == 'Cancel' and n.getState().contains(ui.pyatspi.STATE_ENABLED)]
    if len(controls) != 1:
        raise RuntimeError('Expected one unambiguous healthy manual draft Cancel control')
    ui.reveal_button(application, 'Cancel', window)


def inspect_saved_label(application, window, database, saved):
    ui.reveal_button(application, 'Edit relationship label ' + saved[4], window)
    field = ui.wait_for('exact saved label in native editor', lambda: ui.reveal(application,
        lambda n: n.getState().contains(ui.pyatspi.STATE_EDITABLE)
        and n.name == 'Relationship label ' + EDGE_ID and ui.text_of(n) == saved[4]))
    if edge(database) != saved:
        raise RuntimeError('Opening the saved relationship label changed canon')
    return field


def expand_writing_area(application, window, evidence):
    geometry = {key: int(value) for key, value in (line.split('=', 1) for line in
        ui.command('xdotool', 'getwindowgeometry', '--shell', window).splitlines())}
    def divider():
        return ui.find(application,
        lambda n: n.getRole() == ui.pyatspi.ROLE_PUSH_BUTTON and n.name == 'Resize panels'
        and base.recovery.memory.is_editor_divider(tuple(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)),
            geometry['WIDTH'], geometry['HEIGHT']))
    control = ui.wait_for('existing panel divider', divider)
    before = tuple(control.queryComponent().getExtents(ui.pyatspi.XY_SCREEN))
    ui.click_control(control, window)
    ui.command('xdotool', 'key', '--clearmodifiers', *(['Up'] * 7))
    def expanded():
        current = divider()
        if current is None:
            return None
        bounds = tuple(current.queryComponent().getExtents(ui.pyatspi.XY_SCREEN))
        return bounds if bounds[1] < before[1] else None
    after = ui.wait_for('supported expanded writing area', expanded)
    evidence['bible_relationship']['writing_area_resize'] = {'before': list(before), 'after': list(after),
        'supported_action': 'existing Resize panels keyboard Up'}


def open_relationship_evidence(region, window):
    disclosure = ui.wait_for('original preview relationship disclosure', lambda: ui.reveal(region,
        lambda n: n.name == 'Relationships used for this preview'))
    # ANYWHERE may leave a fragmented summary partly outside the window.
    # Align the exact pending control before the unchanged strict pointer guard.
    disclosure.queryComponent().scrollTo(ui.pyatspi.SCROLL_TOP_LEFT)
    geometry = {key: int(value) for key, value in (line.split('=', 1) for line in
        ui.command('xdotool', 'getwindowgeometry', '--shell', window).splitlines())}
    def aligned():
        disclosure.clear_cache()
        bounds = tuple(disclosure.queryComponent().getExtents(ui.pyatspi.XY_SCREEN))
        try:
            ui.control_click_point(bounds, geometry)
        except RuntimeError as error:
            if str(error) != 'Control bounds are outside the verified native window':
                raise
            return None
        return disclosure
    ui.wait_for('fully bounded pending disclosure after native scroll', aligned)
    ui.click_control(disclosure, window)


def relationship_accessibility(application):
    application.clear_cache()
    matches = []
    for node in ui.walk(application):
        value = ui.text_of(node)
        if DOUBT not in value and DOUBT not in (node.name or ''):
            continue
        matches.append({'role': node.getRoleName(), 'child_count': node.childCount,
            'showing': ui.visible(node), 'name': (node.name or '')[:120],
            'text': value[:160], 'bounds': list(node.queryComponent().getExtents(ui.pyatspi.XY_SCREEN))})
    return matches


def inspect_bible_label(application, width, timeline_y):
    application.clear_cache()
    for node in ui.walk(application):
        # The rendered editor's exact ID/value is the qualified UI contract.
        # Do not assume a plain span has its own accessible text/glyph object.
        if node.name != 'Relationship label ' + EDGE_ID or ui.text_of(node) != DOUBT:
            continue
        node.queryComponent().scrollTo(ui.pyatspi.SCROLL_TOP_LEFT)
        node.clear_cache()
        x, y, w, h = node.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)
        if ui.visible(node) and w > 0 and h > 0 and 0 <= x and x + w <= width and 60 <= y and y + h <= timeline_y and (x + w <= 280 or x >= width - 320):
            return {'exact_label': DOUBT, 'editor_bounds': [x, y, w, h],
                    'role': node.getRoleName(), 'child_count': node.childCount}
    return None


def readable_saved(application, window, evidence):
    block = ui.wait_for('saved relationship screenplay', lambda: ui.screenplay_block(application, REPLACEMENT.splitlines()[-1]))
    block.queryComponent().scrollTo(ui.pyatspi.SCROLL_TOP_LEFT)
    geometry = {key: int(value) for key, value in (line.split('=', 1) for line in
        ui.command('xdotool', 'getwindowgeometry', '--shell', window).splitlines())}
    def exact_ranges():
        application.clear_cache()
        divider = ui.find(application, lambda n: n.getRole() == ui.pyatspi.ROLE_PUSH_BUTTON
            and n.name == 'Resize panels' and base.recovery.memory.is_editor_divider(
                tuple(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)), geometry['WIDTH'], geometry['HEIGHT']))
        timeline = ui.find(application, lambda n: n.getRole() == ui.pyatspi.ROLE_PUSH_BUTTON
            and n.name == 'Resize panels' and n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[2] >= geometry['WIDTH'] - 10)
        if divider is None or timeline is None:
            return None
        top = tuple(divider.queryComponent().getExtents(ui.pyatspi.XY_SCREEN))
        frame = (top[0], top[1] + top[3], top[0] + top[2],
            timeline.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[1])
        saved = next((n for n in ui.walk(application) if n.name == 'Screenplay block'
            and any(REPLACEMENT.splitlines()[-1] in ui.text_of(child) for child in ui.walk(n))), None)
        if saved is None:
            return None
        lines = [line for line in REPLACEMENT.splitlines() if line]
        ranges = {}
        for node in ui.walk(saved):
            value = ui.text_of(node).strip()
            if value not in lines or not ui.visible(node):
                continue
            try:
                bounds = tuple(node.queryText().getRangeExtents(0, len(value), ui.pyatspi.XY_SCREEN))
            except NotImplementedError:
                continue
            if base.recovery.memory.inside_writing_area(bounds, frame):
                ranges[value] = list(bounds)
        return {'bounds': list(frame), 'saved_text_ranges': ranges} if len(ranges) == len(lines) else None
    observed = ui.wait_for('both exact saved screenplay lines inside writing area', exact_ranges)
    evidence['bible_relationship']['label_accessibility_before_inspection'] = relationship_accessibility(application)
    ui.reveal_button(application, 'Edit relationship label ' + DOUBT, window)
    label = ui.wait_for('exact saved Bible label in bounded native editor',
        lambda: inspect_bible_label(application, geometry['WIDTH'], observed['bounds'][3]))
    # Sidebar inspection must also leave both saved screenplay glyph ranges visible.
    observed = ui.wait_for('saved screenplay still readable during Bible inspection', exact_ranges)
    evidence['bible_relationship']['label_accessibility_after_inspection'] = relationship_accessibility(application)
    evidence['bible_relationship']['readable_canonical_bible_label'] = label
    evidence['bible_relationship']['readable_final_actions'] = ['Resize panels keyboard Up', 'AT-SPI SCROLL_TOP_LEFT', 'native saved-label editor inspection without Save']
    evidence['bible_relationship']['readable_final_screenplay'] = {'expected_saved_text': REPLACEMENT, **observed}


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
    inspect_saved_label(application, window, database, changed)
    capture('eidetic-relationship-review.png')
    ui.reveal_button(application, 'Cancel label edit', window)
    # Explicitly cancel the healthy manual draft after proving refresh preservation.
    cancel_manual_draft(application, window)
    if ui.blocks(database, a_id)[0] != a:
        raise RuntimeError('Explicit draft cancellation changed the saved manual source')
    expand_writing_area(application, window, evidence)
    ui.reveal_button(application, 'Preview update', window)
    pending = ui.wait_for('relationship preview persisted', lambda: next((r for r in proposals(database, b[0])
        if r[1] == 'pending' and r[2] == REPLACEMENT), None))
    first = receipt(database, pending[0])
    if first['bible_relationship_inputs'][0]['revision_event_id'] != changed[7] or ui.blocks(database, b_id)[0] != b:
        raise RuntimeError('Preview changed canon or rebound relationship custody')
    region = ui.wait_for('relationship preview native review', lambda: review_region(application, REPLACEMENT))
    open_relationship_evidence(region, window)
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
    observations = evidence['bible_relationship'].setdefault('stale_accessibility', [])
    ui.wait_for('exact stale relationship acceptance refusal', lambda: stale_alert(application, observations))
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
    region = ui.wait_for('fresh exact pending screenplay review', lambda: review_region(application, REPLACEMENT))
    ui.reveal_button(region, 'Accept update', window)
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
