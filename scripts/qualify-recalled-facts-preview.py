#!/usr/bin/env python3
"""Frozen native app; normal selection/review controls, labelled public-command QA replay.

No application source edits, DOM/IPC injection, direct database writes, models or
real-model quality claims. Exact prompts come from the production HTTP client.
"""
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import time
from urllib.request import urlopen

spec = importlib.util.spec_from_file_location('recall_capture', Path(__file__).with_name('qualify-bible-recall.py'))
driver = importlib.util.module_from_spec(spec)
spec.loader.exec_module(driver)
ui = driver.ui
SOURCE = 'e140290ed4414585c0c507874fab5b0e04c9dc22'
SELECTED = 'SELECTED dry roof key — 雨.'
FRESH = 'SELECTED copper roof key — 雨.'
FINAL = 'SELECTED final roof key — 雨.'
UNSELECTED = 'UNSELECTED brass key in cellar.'
FIRST_PREVIEW = 'Synthetic selected-fact preview: Mara protects the witness and the dry roof key.\n\n'
FRESH_PREVIEW = 'Synthetic fresh selected-fact preview: Mara protects the witness and the copper roof key.\n\n'


def prompt_checks(user, kind, phase):
    absent = UNSELECTED not in user and driver.DRAFT not in user and 'environment.weather: Rain' not in user and 'Flooded road' not in user
    if kind == 'recap':
        return absent and driver.GENERATED in user
    common = absent and driver.A_TEXT in user and driver.F_TEXT in user
    if kind == 'generation':
        return common and driver.BLUE in user and SELECTED not in user and FRESH not in user
    common = common and driver.MANUAL in user and driver.AMBER in user and f'profile.tagline: {driver.BLUE}' not in user
    if phase == 2:
        return common and SELECTED not in user and FRESH not in user
    chosen = SELECTED if phase == 3 else FRESH
    excluded = FRESH if phase == 3 else SELECTED
    return common and f'profile.tagline: {chosen}' in user and excluded not in user and 'Mara trusts the Keeper' in user


class SelectedProvider(driver.RecallProvider):
    records = []

    def do_POST(self):
        size = int(self.headers.get('Content-Length', '0'))
        if self.path != '/v1/chat/completions' or not 0 < size <= 512_000:
            self.send_error(400)
            return
        body = json.loads(self.rfile.read(size))
        messages = body.get('messages', [])
        system = next((m['content'] for m in messages if m.get('role') == 'system'), '')
        user = next((m['content'] for m in messages if m.get('role') == 'user'), '')
        kind = 'preview' if 'targeted screenplay update' in system else 'recap' if user.startswith('Generate a scene recap for this screenplay beat:') else 'generation'
        with self.records_lock:
            phase = len([r for r in self.records if r.get('accepted')])
            sequence = ['generation', 'recap', 'preview', 'preview', 'preview']
            valid = phase < len(sequence) and kind == sequence[phase] and body.get('stream') is True and prompt_checks(user, kind, phase)
            record = {'kind': kind, 'phase': phase, 'accepted': valid, 'real_model': False, 'stream': body.get('stream') is True,
                      'exact_user_prompt': user, 'exact_system_prompt': system,
                      'selected_value_expected': SELECTED if phase == 3 else FRESH if phase == 4 else None,
                      'unselected_value_absent': UNSELECTED not in user, 'draft_absent': driver.DRAFT not in user}
            self.records.append(record)
        if not valid:
            self.send_error(422, 'Labelled qualification prompt or phase mismatch')
            return
        text = driver.GENERATED if phase == 0 else 'Mara waits for Eli.' if phase == 1 else driver.PROPOSED if phase == 2 else FIRST_PREVIEW if phase == 3 else FRESH_PREVIEW
        record['synthetic_response'] = text
        parts = [text[:len(text)//2], text[len(text)//2:]]
        data = ''.join('data: ' + json.dumps({'choices': [{'delta': {'content': part}}]}) + '\n\n' for part in parts).encode() + b'data: [DONE]\n\n'
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)


def receipt(application):
    node = ui.find(application, lambda n: n.name == 'QA selected-fact receipt' and n.getRole() in (ui.pyatspi.ROLE_TEXT, ui.pyatspi.ROLE_ENTRY))
    if node is None:
        return None
    result = json.loads(ui.text_of(node))
    if not result['fixture'].startswith('qualifier-only public-command mutation'):
        raise RuntimeError('Missing labelled compiled QA receipt')
    return result


def wait_receipt(application, label, predicate):
    def read():
        value = receipt(application)
        if value and value['error']:
            raise RuntimeError('Public-command QA failure: ' + value['error'])
        return value if value and predicate(value) else None
    return ui.wait_for(label, read)


def pending(database, text):
    rows = ui.query(database, 'SELECT id,proposed_text,status FROM propagation_proposals WHERE status=\'pending\' AND proposed_text=?', (text,))
    return rows[0] if len(rows) == 1 else None


def proposal_rows(database, proposal_id):
    return {'proposal': ui.query(database, 'SELECT * FROM propagation_proposals WHERE id=?', (proposal_id,)),
            'binding': ui.query(database, 'SELECT * FROM script_impact_proposal_bindings WHERE proposal_id=?', (proposal_id,))}


def binding(database, proposal_id):
    rows = ui.query(database, 'SELECT binding_json FROM script_impact_proposal_bindings WHERE proposal_id=?', (proposal_id,))
    if len(rows) != 1:
        raise RuntimeError('Missing canonical proposal binding')
    return json.loads(rows[0][0])


def fact(database):
    rows = ui.query(database, "SELECT text_value,updated_event_id FROM bible_graph_fields WHERE id='qualification.keeper.tagline'")
    return rows[0] if len(rows) == 1 else None


def require_selection_binding(value, expected_fact):
    selection = value['request']['recall_selection']
    if selection['query']['story_time_ms'] is not None or len(selection['facts']) != 1:
        raise RuntimeError('Selection is not exactly one unspecified-time baseline')
    chosen = selection['facts'][0]
    if chosen['field_id'] != 'qualification.keeper.tagline' or chosen['revision_event_id'] != expected_fact[1] or 'value' in chosen:
        raise RuntimeError('Selected field identity/revision custody mismatch')
    fields = value['bible_inputs']
    canonical = next(item for item in fields if item['field_id'] == chosen['field_id'])
    if canonical['value']['value'] != expected_fact[0] or canonical['revision_event_id'] != expected_fact[1]:
        raise RuntimeError('Consumed canonical selected value/revision mismatch')
    if any(item['field_id'] == 'qualification.keeper.motivation' for item in fields):
        raise RuntimeError('Unselected sibling was consumed')
    if not any(path['relationship']['edge']['edge_id'] == 'qualification.mara.keeper' for path in selection['paths']):
        raise RuntimeError('Selected connecting path missing')
    return {'selection': selection, 'canonical_consumed_field': canonical}


def recall_unspecified(application, window):
    field = ui.wait_for('right inspector story-time field', lambda: ui.reveal(application,
        lambda n: n.name == 'Recall story time (ms)' and n.getState().contains(ui.pyatspi.STATE_EDITABLE)
        and n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0] > 1000))
    ui.click_control(field, window)
    def focused():
        field.clear_cache()
        return field.getState().contains(ui.pyatspi.STATE_FOCUSED)
    ui.wait_for('native story-time focus', focused)
    ui.command('xdotool', 'key', '--clearmodifiers', 'ctrl+a')
    ui.command('xdotool', 'key', '--clearmodifiers', 'BackSpace')
    def empty():
        field.clear_cache()
        return ui.text_of(field) == ''
    ui.wait_for('actual empty native story-time value', empty)
    ui.TYPED_TEXT_OBSERVATIONS.append({'field': field.name, 'expected_length': 0, 'actual_text': '', 'exact': True, 'route': 'native ctrl+a and BackSpace'})
    ui.reveal_button(application, 'Recall related story facts', window)


def click_disclosure(application, window, label):
    node = ui.wait_for('actual native disclosure '+label, lambda: ui.reveal(application,
        lambda n: n.name == label and n.getRole() in (ui.pyatspi.ROLE_UNKNOWN, ui.pyatspi.ROLE_PUSH_BUTTON, ui.pyatspi.ROLE_TOGGLE_BUTTON)))
    ui.click_control(node, window)
    return node


def selector_summary(application):
    return ui.reveal(application, lambda n: n.name.startswith('Use recalled facts for this preview (')
                     and n.getRole() in (ui.pyatspi.ROLE_UNKNOWN, ui.pyatspi.ROLE_PUSH_BUTTON, ui.pyatspi.ROLE_TOGGLE_BUTTON))


def select_fact(application, window, value, first=False):
    summary = ui.wait_for('existing recalled-fact selector disclosure', lambda: selector_summary(application))
    if first:
        ui.click_control(summary, window)
    checkbox = ui.wait_for('exact far-peer baseline checkbox', lambda: ui.reveal(application,
        lambda n: n.getRole() == ui.pyatspi.ROLE_CHECK_BOX and value in n.name))
    if not checkbox.getState().contains(ui.pyatspi.STATE_ENABLED):
        raise RuntimeError('Resolved baseline checkbox unexpectedly disabled')
    ui.click_control(checkbox, window)
    def checked():
        checkbox.clear_cache()
        return checkbox.getState().contains(ui.pyatspi.STATE_CHECKED)
    ui.wait_for('native checked selected fact', checked)
    def selected_counter():
        node = selector_summary(application)
        return node if node and '(1/8 selected)' in node.name else None
    summary = ui.wait_for('one selected baseline in counter', selected_counter)
    return {'name': checkbox.name, 'checked': True, 'summary': summary.name}


def require_draft(application):
    return ui.wait_for('exact unrelated unsaved F draft', lambda: ui.editable(application, driver.DRAFT))


def qualify(application, window, database, original, capture, checkpoint, evidence):
    if evidence['application_source_sha'] != SOURCE:
        raise RuntimeError('Frozen source admission mismatch')
    evidence['selected_fact_test_build'] = {'label': 'qualifier-only public-command mutation and saved-selector replay controls',
        'production_files_changed': False, 'DOM_or_IPC_injection': False, 'direct_database_writes': False, 'real_model': False,
        'ordinary_native_controls': ['fact checkboxes', 'Preview update', 'Reject', 'Accept update'],
        'QA_controls': ['mutate far-peer fact through unchanged public command', 'replay exact old selector through unchanged public preview command']}
    evidence['selected_fact_checks'] = checks = []
    previous = pending(database, driver.PROPOSED)
    previous_binding = binding(database, previous[0])
    if any(n['node_id'] == 'qualification.keeper' for n in previous_binding['bible_context']['payload']['nodes']):
        raise RuntimeError('Far peer was already present in automatic scene context')
    ui.reveal_button(application, 'Reject', window)
    ui.wait_for('explicitly rejected initial nonselected proposal', lambda: ui.query(database, "SELECT id FROM propagation_proposals WHERE id=? AND status='rejected'", (previous[0],)))
    retired = proposal_rows(database, previous[0])
    driver.require_preserved(database, original)
    recall_unspecified(application, window)
    ui.wait_for('real far-peer resolved baseline recall', lambda: driver.visible_text(application, 'profile.tagline: ' + SELECTED))
    history = driver.canonical_state(database)
    checks.append({'label': 'normal native baseline selection', 'control': select_fact(application, window, SELECTED, first=True)})
    capture('eidetic-recalled-facts-baseline-selected.png')
    ui.click_control(selector_summary(application), window)
    driver.require_preserved(database, original, history)
    old_fact = fact(database)
    ui.reveal_button(application, 'Preview update', window)
    selected = ui.wait_for('selected pending targeted proposal', lambda: pending(database, FIRST_PREVIEW))
    selected_binding = binding(database, selected[0])
    checks.append({'label': 'pending selected preview canonical custody', 'proposal_id': selected[0], **require_selection_binding(selected_binding, old_fact)})
    wait_receipt(application, 'actual selected request captured by labelled wrapper', lambda r: bool(r['capturedRequest']))
    driver.PROPOSED = FIRST_PREVIEW
    checks[-1]['visible_proposal'] = driver.reveal_visible_proposal(application, window)
    driver.require_preserved(database, original)
    require_draft(application)
    click_disclosure(application, window, 'Author-selected recalled facts used for this preview')
    checks[-1]['visible_proposal_after_receipt_open'] = driver.reveal_visible_proposal(application, window)
    checkpoint('selected far baseline reaches pending proposal; unselected sibling absent from exact HTTP prompt')
    capture('eidetic-recalled-facts-selected-pending.png')

    frozen = proposal_rows(database, selected[0])
    ui.click_button(application, 'QA mutate selected Keeper fact', window)
    changed = ui.wait_for('new canonical selected fact revision', lambda: fact(database) if fact(database)[0] == FRESH and fact(database)[1] != old_fact[1] else None)
    wait_receipt(application, 'ordinary recall invalidation after canonical write', lambda r: r['mutations'] == 1 and r['recallInvalidated'] and r['recallVersion'] is None)
    def retired_counter():
        node = selector_summary(application)
        return node if node and '(0/8 selected)' in node.name else None
    counter = ui.wait_for('selection retired after mutation', retired_counter)
    before_history = driver.canonical_state(database)
    count = len(SelectedProvider.records)
    ui.click_button(application, 'QA replay stale selection', window)
    refused = wait_receipt(application, 'actual backend stale-selector refusal', lambda r: r['replay'].startswith('Actual backend refusal:'))
    if len(SelectedProvider.records) != count or 'Selected recall evidence is stale or unavailable' not in refused['replay']:
        raise RuntimeError('Stale selection did not refuse before HTTP/model work')
    driver.require_preserved(database, original, before_history)
    if proposal_rows(database, selected[0]) != frozen:
        raise RuntimeError('Stale selector replay amended pending proposal')
    ui.reveal_button(application, 'Accept update', window)
    ui.wait_for('ordinary failed acceptance alert', lambda: ui.find(application, lambda n: n.getRole() == ui.pyatspi.ROLE_ALERT and bool(ui.text_of(n))))
    if proposal_rows(database, selected[0]) != frozen:
        raise RuntimeError('Stale acceptance changed proposal')
    driver.require_preserved(database, original, before_history)
    require_draft(application)
    checks.append({'label': 'stale selection and acceptance refused', 'before_fact': old_fact, 'after_fact': changed, 'selection_counter': counter.name,
                   'actual_QA_receipt': refused, 'provider_calls_unchanged': True, 'pending_columns_unchanged': True})
    checkpoint('canonical far-peer mutation clears selection; exact old selection and stale acceptance refuse')
    capture('eidetic-recalled-facts-stale-refused.png')

    ui.reveal_button(application, 'Reject', window)
    ui.wait_for('explicit rejection of stale proposal', lambda: ui.query(database, "SELECT id FROM propagation_proposals WHERE id=? AND status='rejected'", (selected[0],)))
    recall_unspecified(application, window)
    ui.wait_for('fresh real baseline recall after mutation', lambda: driver.visible_text(application, 'profile.tagline: ' + FRESH))
    checks.append({'label': 'fresh explicit selection', 'control': select_fact(application, window, FRESH, first=True)})
    capture('eidetic-recalled-facts-fresh-selected.png')
    ui.click_control(selector_summary(application), window)
    ui.reveal_button(application, 'Preview update', window)
    fresh = ui.wait_for('fresh pending selected proposal', lambda: pending(database, FRESH_PREVIEW))
    checks.append({'label': 'fresh canonical selected custody', 'proposal_id': fresh[0], **require_selection_binding(binding(database, fresh[0]), changed)})
    driver.PROPOSED = FRESH_PREVIEW
    checks[-1]['visible_proposal'] = driver.reveal_visible_proposal(application, window)
    driver.require_preserved(database, original)
    require_draft(application)
    checkpoint('fresh selected proposal visibly pending; exact manual target and unrelated draft still retained')
    capture('eidetic-recalled-facts-fresh-pending.png')
    ui.reveal_button(application, 'Accept update', window)
    ui.wait_for('explicit accepted fresh proposal', lambda: ui.query(database, "SELECT id FROM propagation_proposals WHERE id=? AND status='accepted'", (fresh[0],)))
    b_id = evidence['public_fixture']['b']['id']
    a_id = evidence['public_fixture']['a']['id']
    f_id = evidence['public_fixture']['f']['id']
    accepted = ui.blocks(database, b_id)
    if len(accepted) != 1 or accepted[0][1] != FRESH_PREVIEW or accepted[0][3:] != original[b_id][0][3:]:
        raise RuntimeError('Explicit acceptance did not preserve exact target placement')
    for node_id in (a_id, f_id):
        if ui.blocks(database, node_id) != original[node_id]:
            raise RuntimeError('Acceptance changed unrelated saved manual material')
    require_draft(application)
    if proposal_rows(database, previous[0]) != retired:
        raise RuntimeError('Prior rejected proposal changed')
    dependencies = ui.query(database, "SELECT d.id,d.source_field_id,r.source_revision_event_id FROM semantic_dependencies d JOIN semantic_dependency_revisions r ON r.dependency_id=d.id WHERE d.source_field_id='qualification.keeper.tagline' AND d.deleted_event_id IS NULL")
    if not dependencies or not all(row[2] == changed[1] for row in dependencies):
        raise RuntimeError('Acceptance did not record selected-fact dependency revision')
    checks.append({'label': 'explicit native fresh acceptance', 'proposal_id': fresh[0], 'accepted_block': accepted[0], 'selected_dependencies': dependencies,
                   'unrelated_saved_A_F_preserved': True, 'exact_F_draft_preserved': driver.DRAFT, 'placement_preserved': True})
    ui.wait_for('accepted screenplay rendered', lambda: ui.reveal(application, lambda n: FRESH_PREVIEW.strip() in ui.text_of(n) and n.getRole() == ui.pyatspi.ROLE_PARAGRAPH))
    checkpoint('explicit fresh Accept updates only target and records selected dependency; unrelated manual/draft text intact')
    capture('eidetic-recalled-facts-accepted.png')
    ui.click_button(application, 'QA mutate selected Keeper fact', window)
    ui.wait_for('subsequent accepted-fact mutation', lambda: fact(database) if fact(database)[0] == FINAL else None)
    click_disclosure(application, window, 'What changed')
    cause = ui.wait_for('visible subsequent selected-fact impact', lambda: driver.visible_review_label(application, window, 'Bible fact profile.tagline changed.'))
    if ui.blocks(database, b_id) != accepted:
        raise RuntimeError('Later fact mutation replaced accepted screenplay')
    require_draft(application)
    checks.append({'label': 'later consumed-fact edit derives review without replacement', 'visible_cause': cause, 'after_fact': fact(database), 'saved_accepted_text_preserved': True})
    evidence['explicit_acceptance'] = checks[-2]
    checkpoint('subsequent selected fact edit derives Needs review without replacing accepted text')
    capture('eidetic-recalled-facts-subsequent-impact.png')


def main():
    repo = Path.cwd()
    output = Path(os.environ['EIDETIC_CAPTURE_DIR'])
    output.mkdir(parents=True, exist_ok=True)
    config = repo/'scripts/recalled-facts-vite.config.mts'
    log = Path(os.environ['RUNNER_TEMP'])/'recalled-facts-vite-private.log'
    process = None
    try:
        args = [str(repo/'ui/node_modules/.bin/vite'), 'dev', '--config', str(config), '--port', '5173', '--host', '127.0.0.1', '--strictPort']
        with log.open('w') as stream:
            process = subprocess.Popen(args, cwd=repo/'ui', stdout=stream, stderr=subprocess.STDOUT, start_new_session=True)
        def ready():
            if process.poll() is not None:
                raise RuntimeError('Qualifier host exited')
            try:
                with urlopen('http://127.0.0.1:5173/', timeout=2) as response:
                    return response.status == 200
            except OSError:
                return False
        ui.DEADLINE = time.monotonic() + 480
        ui.wait_for('labelled qualifier Vite host', ready)
        original_wait = ui.wait_for
        def bounded_wait(label, check):
            overall = ui.DEADLINE
            ui.DEADLINE = min(overall, time.monotonic()+45)
            try:
                return original_wait(label, check)
            finally:
                ui.DEADLINE = overall
        ui.wait_for = bounded_wait
        driver.RecallProvider = SelectedProvider
        driver.main(after_pending=qualify, qualification_host={'pid': process.pid, 'arguments': args, 'cwd': str(repo/'ui'), 'config_sha256': ui.file_hash(config), 'instrumented_native_host': True})
    finally:
        if process and process.poll() is None:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
        raw = log.read_text(errors='replace') if log.exists() else 'No qualifier host log.'
        (output/'qualification-host-sanitized.log').write_text(ui.sanitized_log(raw, [(str(repo), '<repo>')]))


if __name__ == '__main__':
    main()
