#!/usr/bin/env python3
"""Actual typed arc memory propagation with labelled synthetic HTTP/SSE.

Only public-service setup precedes launch. GUI controls own all manual edits,
preview, stale refusal and explicit acceptance. Database queries are read-only.
"""
import importlib.util
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import threading
import time
from http.server import ThreadingHTTPServer

spec = importlib.util.spec_from_file_location('arc_authoring_helpers', Path(__file__).with_name('qualify-story-authoring-walkthrough.py'))
walk = importlib.util.module_from_spec(spec)
spec.loader.exec_module(walk)
ui, placement = walk.ui, walk.placement
SOURCE = os.environ.get('EIDETIC_CAPTURE_SOURCE', '292630b91b6677684279f852fdfb7257b8c97aa8')
OLD = "Mara conceals the witness's identity."
NEW = "Mara reveals the witness's identity."
F_TEXT = walk.F_TEXT
DRAFT = walk.DRAFT
GENERATED = 'Synthetic generation: Mara conceals the witness from Eli.\n\n'
MANUAL = 'EXT. STATION - NIGHT\n\nMara keeps the witness hidden. Preserve this station beat.\n\n'
STALE = "Synthetic preview: Mara reveals the witness's identity.\n\n"
PROPOSED = 'Synthetic preview: Mara names the witness for Eli.\n\n'


class ArcProvider(ui.FixtureProvider):
    records = []
    records_lock = threading.Lock()
    arc_id = ''
    sequence = ['generation', 'recap', 'preview', 'preview']

    def contains_expected_context(self, user, kind):
        if kind == 'recap':
            return GENERATED in user
        if kind == 'generation':
            return f'STORY ARCS: Witness (APlot) — {OLD}' in user and F_TEXT in user
        return (f'{self.arc_id} description: {NEW}' in user
                and 'CURRENT TAGGED STORY ARC FIELDS:' in user
                and MANUAL in user and F_TEXT in user and DRAFT not in user)

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
        record = {'kind': kind, 'contains_exact_expected_context': self.contains_expected_context(user, kind),
                  'stream': body.get('stream') is True, 'real_model': False,
                  'prompt_sha256': hashlib.sha256(user.encode()).hexdigest()}
        with self.records_lock:
            accepted = [item for item in self.records if item.get('accepted') is True]
            record['accepted'] = (len(accepted) < len(self.sequence) and kind == self.sequence[len(accepted)]
                                  and record['stream'] and record['contains_exact_expected_context'])
            self.records.append(record)
            position = len(accepted)
        if not record['accepted']:
            self.send_error(422, 'Synthetic qualification requires exact current canonical arc evidence')
            return
        text = GENERATED if kind == 'generation' else 'Mara waits for Eli.' if kind == 'recap' else STALE if position == 2 else PROPOSED
        data = ''.join('data: ' + json.dumps({'choices': [{'delta': {'content': part}}]}) + '\n\n'
                       for part in (text[:len(text)//2], text[len(text)//2:])).encode() + b'data: [DONE]\n\n'
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)


def arc_value(database):
    rows = ui.query(database, "SELECT a.description,r.change_event_id FROM arcs a JOIN object_revisions r ON r.object_id=a.id JOIN object_revision_fields f ON f.revision_id=r.id JOIN change_events e ON e.id=r.change_event_id WHERE a.id=? AND r.object_kind='story_arc' AND f.field_key='description' ORDER BY e.rowid DESC,r.rowid DESC LIMIT 1", (ArcProvider.arc_id,))
    return rows[0] if rows else None


def edit_arc(application, window, database, old, new):
    ui.click_button(application, 'Arcs', window)
    if ui.editable(application, old) is None:
        ui.click_button(application, 'Witness', window, prefix=True)
    field = ui.wait_for('exact current arc description', lambda: ui.editable(application, old))
    before = arc_value(database)
    ui.type_text(field, window, new)
    current = ui.wait_for('canonical exact arc description and new owned revision', lambda: arc_value(database) if arc_value(database)[0] == new and arc_value(database)[1] != before[1] else None)
    return current


def show_bible(application, window):
    ui.click_button(application, 'Bible', window)
    if ui.editable(application, walk.BLUE) is None:
        ui.click_button(application, 'Mara', window, prefix=True)


def unchanged(database, original, original_fact):
    walk.require_canonical_unchanged(database, original)
    if walk.fact(database) != original_fact:
        raise RuntimeError('Arc-only qualification changed the Bible')


def main():
    repo = Path.cwd()
    output = Path(os.environ['EIDETIC_CAPTURE_DIR'])
    output.mkdir(parents=True, exist_ok=True)
    state_root = Path(os.environ['RUNNER_TEMP']) / 'eidetic-story-arc-memory-state'
    state_root.mkdir(parents=True, exist_ok=True)
    raw_log = state_root / 'app-private.log'
    environment = os.environ.copy()
    environment['EIDETIC_LAUNCHER_STATE_ROOT'] = str(state_root)
    for suffix in ('CACHE', 'CONFIG', 'DATA', 'STATE'):
        directory = state_root / 'dev' / ('xdg-' + suffix.lower())
        directory.mkdir(parents=True, exist_ok=True)
        environment['XDG_' + suffix + '_HOME'] = str(directory)
    environment.update(NO_AT_BRIDGE='0', GTK_MODULES='atk-bridge')
    ui.Atspi.set_timeout(1500, 1500)
    started = time.monotonic()
    evidence = {'status': 'failed', 'application_source_sha': SOURCE,
                'qualification_sha': ui.command('git', 'rev-parse', 'HEAD'),
                'application_binary_sha256': ui.file_hash(repo / 'target/debug/eidetic-desktop'),
                'capture_label': 'prototype views',
                'setup_route': 'public services; tagged B ungenerated and unconsumed F manually authored',
                'gui_route': 'actual Tauri AT-SPI/X11 Generate, manual screenplay Save, typed arc edit, retained draft, stale refusal, preview and explicit acceptance',
                'provider': 'synthetic localhost HTTP/SSE fixture through production client',
                'real_model_quality_qualified': False, 'checkpoints': []}
    process = application = window = None
    provider = ThreadingHTTPServer(('127.0.0.1', 18080), ArcProvider)
    threading.Thread(target=provider.serve_forever, daemon=True).start()

    def checkpoint(stage):
        evidence['stage'] = stage
        evidence['checkpoints'].append({'stage': stage, 'elapsed_seconds': round(time.monotonic() - started, 3)})
        (output / 'capture-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')
        print('Authoring walkthrough checkpoint: ' + stage, flush=True)

    def capture(name):
        pid = int(ui.command('xdotool', 'getwindowpid', window))
        if process.poll() is not None or os.getpgid(pid) != process.pid:
            raise RuntimeError('Native window ownership changed')
        digest = ui.capture_native_window(window, output / name, screen_read=True)
        evidence.setdefault('captures', []).append({'file': name, 'sha256': digest, 'window_id': int(window), 'pid': pid})

    try:
        checkpoint('public services prepare fresh Last Train project with empty A and ungenerated B')
        fixture = json.loads(subprocess.check_output([str(repo / 'target/debug/examples/story_arc_memory_fixture')],
                                                    env=environment, text=True, timeout=60))
        database = Path(fixture['project_path']).resolve()
        b_id, f_id = fixture['b']['id'], fixture['f']['id']
        ArcProvider.arc_id = fixture['arc']['id']
        if ui.blocks(database, b_id):
            raise RuntimeError('Native B generation target is not empty')
        original = {node: ui.blocks(database, node) for node in (b_id, f_id)}
        original_fact = walk.fact(database)
        original_arc = arc_value(database)
        evidence['project'] = {'name': database.parent.name, 'setup_route': fixture['setup_route'],
                               'arc_id': ArcProvider.arc_id, 'b': fixture['b'], 'f': fixture['f']}
        evidence['original_arc'] = {'exact_text': original_arc[0], 'revision_event_id': original_arc[1]}
        checkpoint('launch actual desktop and begin writing')
        with raw_log.open('w') as stream:
            process = subprocess.Popen(['./launcher.sh', '--run'], env=environment,
                stdout=stream, stderr=subprocess.STDOUT, start_new_session=True)

        def owned_window():
            if process.poll() is not None:
                raise RuntimeError('Launcher exited: ' + str(process.returncode))
            windows = subprocess.run(['xdotool', 'search', '--onlyvisible', '--name', '^Eidetic$'], capture_output=True, text=True, timeout=5)
            for candidate in windows.stdout.splitlines():
                pid = int(ui.command('xdotool', 'getwindowpid', candidate))
                if os.getpgid(pid) == process.pid:
                    return candidate, pid
            return None

        window, pid = ui.wait_for('owned native window', owned_window)
        ui.NATIVE_WINDOW_PID = pid
        evidence.update(window_id=int(window), window_pid=pid)
        evidence['application_binary_sha256'] = ui.file_hash(Path('/proc/' + str(pid) + '/exe'))
        ui.command('xdotool', 'windowsize', '--sync', window, '1920', '1440')
        evidence['native_window_geometry'] = ui.command('xdotool', 'getwindowgeometry', '--shell', window)
        application = ui.wait_for('native accessibility app', lambda: next((app for app in ui.pyatspi.Registry.getDesktop(0) if app.get_process_id() == pid), None))
        ui.open_project_chooser(application, window)
        ui.click_button(application, database.parent.name, window, prefix=True)
        placement.enlarge_script_pane(application, window)
        ui.reveal_button(application, 'Zoom to Fit (Ctrl+0)', window)
        for _ in range(3):
            ui.reveal_button(application, '+', window)
        ui.click_button(application, 'AI', window)
        ui.click_button(application, 'Save & Connect', window)
        ui.wait_for('connected synthetic HTTP fixture', lambda: ui.find(application, lambda n: 'Connected' in ui.text_of(n)))
        clip = ui.wait_for('tagged B native clip', lambda: ui.native_timeline_clip(application, fixture['b']['name'], window))
        ui.click_control(clip[0], window, clip[1])
        ui.reveal_button(application, 'Generate', window)
        generated = ui.wait_for('native generated B', lambda: next((row for row in ui.blocks(database, b_id) if row[1] == GENERATED), None))
        ui.wait_for('generation complete in actual selected inspector', lambda: walk.visible_inspector_feedback(application, window, fixture['b']['name'], 'Generate'))
        generated_json = ui.query(database, "SELECT c.payload_json FROM commands c JOIN change_events e ON e.command_id=c.id WHERE e.id=?", (generated[2],))[0][0]
        receipt = json.loads(generated_json)['arc_inputs']
        consumed = next(item for item in receipt if item['field'] == 'description' and item['arc_id'] == ArcProvider.arc_id)
        if consumed['value'] != OLD or consumed['revision_event_id'] != original_arc[1]:
            raise RuntimeError('Generation did not retain exact original arc consumption')
        evidence['generation'] = {'block_id': generated[0], 'revision_event_id': generated[2], 'arc_inputs': receipt, 'real_model': False}
        capture('eidetic-story-arc-generated.png')
        block = ui.wait_for('generated B editor', lambda: ui.screenplay_block(application, walk.screenplay_anchor(GENERATED)))
        ui.reveal_button(block, 'Edit', window)
        field = ui.wait_for('generated B edit area', lambda: ui.editable(application, GENERATED))
        ui.type_text(field, window, MANUAL)
        block = ui.wait_for('exact manual B draft', lambda: ui.screenplay_block(application, walk.screenplay_anchor(MANUAL)))
        ui.reveal_button(block, 'Save', window)
        saved = ui.wait_for('exact manual B saved', lambda: next((row for row in ui.blocks(database, b_id) if row[1] == MANUAL), None))
        original[b_id] = [saved]
        evidence['manual_saved_B'] = {'exact_text': MANUAL, 'revision_event_id': saved[2]}
        capture('eidetic-story-arc-manual-edit.png')
        block = ui.wait_for('unrelated F block', lambda: ui.screenplay_block(application, walk.screenplay_anchor(F_TEXT)))
        ui.reveal_button(block, 'Edit', window)
        field = ui.wait_for('F edit area', lambda: ui.editable(application, F_TEXT))
        ui.type_text(field, window, DRAFT)
        ui.wait_for('exact unrelated draft retained', lambda: walk.draft_field(application))
        unchanged(database, original, original_fact)
        capture('eidetic-story-arc-draft.png')
        changed = edit_arc(application, window, database, OLD, NEW)
        evidence['manual_arc_edit'] = {'before': OLD, 'after': NEW, 'before_revision_event_id': original_arc[1], 'after_revision_event_id': changed[1], 'route': 'actual native typed description and existing debounced canonical command'}
        ui.wait_for('F draft survives arc autosave', lambda: walk.draft_field(application))
        unchanged(database, original, original_fact)
        capture('eidetic-story-arc-arc-edit.png')
        show_bible(application, window)
        ui.wait_for('actual arc cause visible', lambda: walk.visible_review_label(application, window, 'Story arc description changed.'))
        checkpoint('exact typed arc edit identifies affected saved B without changing manual screenplay or Bible')
        capture('eidetic-story-arc-review.png')
        ui.reveal_button(application, 'Preview update', window)
        pending = ui.wait_for('first pending native arc preview', lambda: ui.proposal(database, b_id))
        if pending[1] != STALE:
            raise RuntimeError('First synthetic preview differs')
        walk.PROPOSED = STALE
        evidence['preview'] = {'proposal_id': pending[0], 'canonical_unchanged': True, 'visible_proposal': walk.reveal_visible_proposal(application, window)}
        binding_json = ui.query(database, "SELECT binding_json FROM script_impact_proposal_bindings WHERE proposal_id=?", (pending[0],))[0][0]
        binding = json.loads(binding_json)
        if binding['arc_previous_inputs'] != receipt or next(item for item in binding['arc_inputs'] if item['field'] == 'description')['value'] != NEW:
            raise RuntimeError('Pending preview loses original/current arc evidence')
        evidence['preview']['binding'] = binding
        unchanged(database, original, original_fact)
        ui.wait_for('pending preview retains draft', lambda: walk.draft_field(application))
        capture('eidetic-story-arc-preview.png')
        restored = edit_arc(application, window, database, NEW, OLD)
        newer = edit_arc(application, window, database, OLD, NEW)
        show_bible(application, window)
        ui.reveal_button(application, 'Accept update', window)
        ui.wait_for('native stale acceptance refusal', lambda: ui.find(application, lambda n: 'screenplay proposal is stale' in ui.text_of(n)))
        unchanged(database, original, original_fact)
        evidence['stale_refusal'] = {'arc_aba_revisions': [changed[1], restored[1], newer[1]], 'exact_text_restored': NEW, 'saved_B_unchanged': True, 'draft_retained': bool(walk.draft_field(application))}
        checkpoint('arc description ABA visibly refuses old preview and retains saved manual B')
        capture('eidetic-story-arc-stale.png')
        ui.reveal_button(application, 'Reject', window)
        ui.wait_for('old preview explicitly rejected', lambda: ui.query(database, "SELECT status FROM propagation_proposals WHERE id=?", (pending[0],))[0][0] == 'rejected')
        ui.reveal_button(application, 'Preview update', window)
        fresh = ui.wait_for('fresh pending native preview', lambda: ui.proposal(database, b_id))
        if fresh[1] != PROPOSED or fresh[0] == pending[0]:
            raise RuntimeError('Fresh synthetic proposal differs')
        walk.PROPOSED = PROPOSED
        evidence['fresh_preview'] = {'proposal_id': fresh[0], 'visible_proposal': walk.reveal_visible_proposal(application, window)}
        unchanged(database, original, original_fact)
        capture('eidetic-story-arc-fresh-preview.png')
        ui.reveal_button(application, 'Accept update', window)
        accepted = ui.wait_for('explicitly accepted native B', lambda: next((row for row in ui.blocks(database, b_id) if row[1] == PROPOSED and row[2] != saved[2]), None))
        if accepted[0] != saved[0] or accepted[3:] != saved[3:] or ui.blocks(database, b_id) != [accepted]:
            raise RuntimeError('Acceptance replaced more than the selected B block')
        original[b_id] = [accepted]
        unchanged(database, original, original_fact)
        dependencies = ui.query(database, "SELECT r.target_revision_event_id FROM semantic_dependencies d JOIN semantic_dependency_revisions r ON r.dependency_id=d.id WHERE d.target_kind='story_arc_field' AND d.target_id=? AND d.target_field_key='description' AND r.source_revision_event_id=?", (ArcProvider.arc_id, accepted[2]))
        if dependencies != [(newer[1],)]:
            raise RuntimeError('Accepted B did not install actual current arc consumption')
        ui.wait_for('arc review cause cleared', lambda: not ui.find(application, lambda n: 'Story arc description changed.' in ui.text_of(n)))
        ui.wait_for('F draft retained after explicit acceptance', lambda: walk.draft_field(application))
        evidence['acceptance'] = {'revision_event_id': accepted[2], 'only_B_changed': True, 'arc_revision_consumed': newer[1], 'Bible_unchanged': True, 'review_cleared': True, 'exact_F_draft_retained': DRAFT, 'visible_proposal': walk.reveal_visible_proposal(application, window, pending=False)}
        checkpoint('explicit acceptance alone replaces B and refreshes owned arc lineage')
        capture('eidetic-story-arc-accepted.png')
        capture('eidetic-story-arc-retained-draft.png')
        calls = ArcProvider.records
        if len(calls) != 4 or any(item.get('accepted') is not True or item['real_model'] is not False for item in calls):
            raise RuntimeError('Bounded synthetic provider sequence differs')
        evidence['status'] = 'native_story_arc_memory_passed_with_labelled_synthetic_http_fixture'
    except Exception as error:
        evidence['failure'] = type(error).__name__ + ': ' + str(error)[:1000]
        if application is not None:
            try:
                evidence['accessibility_snapshot'] = ui.accessibility_snapshot(application)
                capture('eidetic-story-arc-failure.png')
            except Exception as capture_error:
                evidence['failure_capture_error'] = type(capture_error).__name__
        raise
    finally:
        provider.shutdown()
        provider.server_close()
        if process is not None:
            try:
                os.killpg(process.pid, signal.SIGTERM)
                process.wait(timeout=5)
            except (ProcessLookupError, subprocess.TimeoutExpired):
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
        evidence['provider_calls'] = ArcProvider.records
        evidence['accessibility_retired_object_polls'] = ui.ACCESSIBILITY_RETIREMENTS
        evidence['typed_text_observations'] = ui.TYPED_TEXT_OBSERVATIONS
        evidence['timeline_locator_observations'] = ui.TIMELINE_LOCATOR_OBSERVATIONS
        raw = raw_log.read_text(errors='replace') if raw_log.exists() else 'No app launch log.'
        (output / 'app-sanitized.log').write_text(ui.sanitized_log(raw, [(str(repo), '<workspace>'), (str(state_root), '<capture-state>'), (os.environ.get('RUNNER_TEMP'), '<runner-temp>')]))
        (output / 'capture-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')


if __name__ == '__main__':
    main()
