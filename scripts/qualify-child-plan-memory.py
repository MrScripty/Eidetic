#!/usr/bin/env python3
"""Extend maintained real native Bible qualification with screenplay-aware plans.

All input is native AT-SPI/X11. SQLite observations are read-only. Responses are
explicitly synthetic HTTP fixtures, not evidence of real-model quality.
"""
import importlib.util
import io
import json
from pathlib import Path

spec = importlib.util.spec_from_file_location('bible_capture', Path(__file__).with_name('qualify-bible-fact-propagation.py'))
bible = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bible)
ui = bible.ui

MIDNIGHT = 'INT. CAFE - NIGHT\n\nMara takes her blue umbrella.\nThe train leaves at midnight.\n\n'
MORNING = 'INT. CAFE - NIGHT\n\nMara takes her blue umbrella.\nThe train leaves in the morning.\n\n'


class ChildProvider(bible.BibleFactProvider):
    records = []

    def do_POST(self):
        size = int(self.headers.get('Content-Length', '0'))
        if self.path != '/v1/chat/completions' or not 0 < size <= 512_000:
            self.send_error(400)
            return
        data = self.rfile.read(size)
        body = json.loads(data)
        if body.get('stream') is True:
            original = self.rfile
            self.rfile = io.BytesIO(data)
            try:
                super().do_POST()
            finally:
                self.rfile = original
            return
        user = next((m['content'] for m in body.get('messages', []) if m.get('role') == 'user'), '')
        fresh = MORNING in user
        kind = 'child_fresh' if fresh else 'child_initial'
        valid = ((MORNING if fresh else MIDNIGHT) in user and ui.PROPOSED_TEXT in user
                 and f'profile.tagline: {bible.BLUE}' in user and body.get('stream') is False)
        with self.records_lock:
            accepted = valid and not any(r['kind'] == kind and r.get('accepted') for r in self.records)
            self.records.append({'kind': kind, 'accepted': accepted,
                                 'contains_exact_expected_context': valid,
                                 'stream': False, 'real_model': False})
        if not accepted:
            self.send_error(422, 'Child plan omitted exact canonical screenplay/Bible context')
            return
        name = 'Morning departure' if fresh else 'Midnight departure'
        children = [{'name': name, 'outline': f'Mara takes her blue umbrella to the {"morning" if fresh else "midnight"} train.',
                     'weight': 1.0, 'characters': [], 'props': []}]
        response = json.dumps({'choices': [{'message': {'content': json.dumps(children)}}]}).encode()
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(response)))
        self.end_headers()
        self.wfile.write(response)


def child_flow(application, window, database, fixture, evidence, checkpoint, capture):
    a, b = fixture['a']['id'], fixture['b']['id']
    original_b = ui.blocks(database, b)[0]

    def manual_save(previous, text):
        block = ui.wait_for('saved manual source block', lambda: ui.screenplay_block(application, previous))
        ui.reveal_button(block, 'Edit', window)
        field = ui.wait_for('exact existing source draft', lambda: ui.editable(application, previous))
        ui.type_text(field, window, text)
        draft = ui.wait_for('source draft block', lambda: ui.screenplay_block(application, text))
        ui.reveal_button(draft, 'Save', window)
        return ui.wait_for('exact new saved source screenplay', lambda: next((row for row in ui.blocks(database, a) if row[1] == text), None))

    def children():
        return ui.query(database, 'SELECT id,name,content_json FROM nodes WHERE parent_id=? ORDER BY id', (b,))

    def plans():
        return ui.query(database, 'SELECT id,status FROM child_plans WHERE parent_node_id=? ORDER BY rowid', (b,))

    def plan_beats():
        control = ui.wait_for('reachable scene child planning', lambda: ui.reveal(application,
            lambda n: n.getRole() == ui.pyatspi.ROLE_PUSH_BUTTON and n.name in ('Plan Beats', 'Replan Beats')
            and n.getState().contains(ui.pyatspi.STATE_ENABLED)))
        ui.click_control(control, window)

    def select_target():
        scene = ui.wait_for('selected target scene', lambda: ui.native_timeline_clip(application, fixture['b']['name'], window))
        ui.click_control(scene[0], window, scene[1])

    checkpoint('manual screenplay save changes exact evidence for downstream timeline planning')
    first = manual_save(ui.MANUAL_TEXT, MIDNIGHT)
    select_target()
    old_children = children()
    plan_beats()
    initial = ui.wait_for('pending screenplay-aware child plan', lambda: next((row for row in plans() if row[1] == 'pending'), None))
    ui.wait_for('proposed timeline material visible', lambda: ui.reveal(application, lambda n: 'Midnight departure' in ui.text_of(n)))
    if children() != old_children or ui.blocks(database, a)[0] != first or ui.blocks(database, b)[0] != original_b:
        raise RuntimeError('Child planning changed canonical timeline or screenplay before acceptance')
    evidence['child_planning'] = {'initial_plan_id': initial[0], 'pending_preserves_children_and_screenplay': True,
                                  'first_manual_edit': MIDNIGHT, 'real_model': False}
    checkpoint('reviewable child proposal; no automatic apply')
    capture('eidetic-child-plan-pending.png')

    latest = manual_save(MIDNIGHT, MORNING)
    # Editing the source block does not retarget the selected scene's proposal.
    ui.reveal_button(application, 'Accept timeline plan', window)
    ui.wait_for('visible stale child plan refusal', lambda: ui.reveal(application,
        lambda n: 'Child plan story context changed; generate and review a fresh plan before accepting' in ui.text_of(n)))
    if plans()[0] != initial or children() != old_children or ui.blocks(database, a)[0] != latest or ui.blocks(database, b)[0] != original_b:
        raise RuntimeError('Stale child acceptance changed canonical material or plan status')
    evidence['child_planning']['stale_acceptance_refused_and_pending_retained'] = True
    checkpoint('intervening manual screenplay edit refuses the old child plan')
    capture('eidetic-child-plan-refused.png')

    plan_beats()
    fresh = ui.wait_for('fresh pending child plan', lambda: next((row for row in plans() if row[0] != initial[0] and row[1] == 'pending'), None))
    ui.wait_for('fresh proposed timeline material visible', lambda: ui.reveal(application, lambda n: 'Morning departure' in ui.text_of(n)))
    ui.reveal_button(application, 'Accept timeline plan', window)
    ui.wait_for('explicitly accepted new child plan', lambda: next((row for row in plans() if row == (fresh[0], 'applied')), None))
    accepted_children = ui.wait_for('canonical proposed child material', lambda: children() if any(row[1] == 'Morning departure' for row in children()) else None)
    if ui.blocks(database, a)[0] != latest or ui.blocks(database, b)[0] != original_b or bible.fact(database)[0] != bible.BLUE:
        raise RuntimeError('Child acceptance rewrote saved screenplay or Bible')
    evidence['child_planning'].update(status='native_child_plan_memory_passed_with_synthetic_http_fixture', fresh_plan_id=fresh[0], final_manual_edit=MORNING,
                                     accepted_children=accepted_children,
                                     explicit_timeline_acceptance_preserved_screenplay_and_bible=True)
    checkpoint('fresh screenplay-aware timeline plan accepted explicitly')
    capture('eidetic-child-plan-accepted.png')


if __name__ == '__main__':
    bible.BibleFactProvider = ChildProvider
    bible.AFTER_ACCEPTANCE = child_flow
    bible.main()
