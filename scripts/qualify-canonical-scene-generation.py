#!/usr/bin/env python3
"""Real native create/select/author/generate, plus delayed manual-edit refusal.

Production HTTP/SSE client with explicitly synthetic responses. No injected IPC,
JS, SQLite writes, reopening after scene creation, pixel edits or model-quality claim.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import threading
import time
from http.server import ThreadingHTTPServer

spec = importlib.util.spec_from_file_location('scene_helpers', Path(__file__).with_name('qualify-scene-order-memory.py'))
helpers = importlib.util.module_from_spec(spec)
spec.loader.exec_module(helpers)
ui = helpers.ui
SOURCE = '9e8bd1c9d51250ac1c517945269278b7fe7e3d61'
NOTES = 'Exact new scene notes: Mara waits beside the station.\n'
ANCHOR = 'Exact manual new-scene anchor B.\n\n'
A = 'Exact existing authored scene A.\n\n'
OUTPUT = 'Synthetic immediate output: Mara waits beside the station.\n\n'
HUMAN = 'Exact human replacement saved during delayed generation.\n\n'
DELAYED = 'Synthetic delayed response that must not replace human text.\n\n'
HEADER = 'CANONICAL SCREENPLAY CONTEXT (authored text; world interpretations require review):\n'
SUFFIX = 'Write ONLY the structural outline for this scene. Do not include metadata, comments, or explanations.'



def serialized_blocks(rows):
    return ''.join(f'--- document={d} segment={s} block={b} block_revision={br} segment_revision={sr} presentation={start}..{end}ms ---\n{text}\n\n'
                   for d, s, b, br, sr, start, end, text in rows)


def exact_context(user, expected):
    """Exact complete serialized section, including ordered identity/text/revisions.

    Fixture projects have no RAG/style/legacy anchors after the canonical section.
    A swapped, extra, duplicated or obsolete block therefore cannot pass.
    """
    if user.count(HEADER) != 1:
        return False
    before, section = user.split(HEADER)
    return (section == serialized_blocks(expected) + SUFFIX
            and before.count(NOTES) == 1
            and "profile.tagline: Mara's umbrella is blue." in before
            and all(user.count(text) == sum(row[-1] == text for row in expected)
                    for text in (ANCHOR, A, OUTPUT, HUMAN, DELAYED)))


def canonical_rows(database):
    return ui.query(database, '''SELECT s.document_id,s.id,b.id,b.updated_event_id,s.updated_event_id,s.start_ms,s.end_ms,b.text
        FROM script_segments s JOIN script_blocks b ON b.segment_id=s.id JOIN script_documents d ON d.id=s.document_id
        WHERE s.document_id='script.document.main' AND s.deleted_event_id IS NULL AND b.deleted_event_id IS NULL AND d.deleted_event_id IS NULL
        ORDER BY s.start_ms,s.sort_order,s.id,b.sort_order,b.id''')


class CanonicalProvider(ui.FixtureProvider):
    records = []
    records_lock = threading.Lock()
    expected = {}
    delayed_seen = threading.Event()
    release = threading.Event()

    def do_POST(self):
        size = int(self.headers.get('Content-Length', '0'))
        if self.path != '/v1/chat/completions' or not 0 < size <= 512000:
            self.send_error(400)
            return
        try:
            body = json.loads(self.rfile.read(size))
            user = next(m['content'] for m in body['messages'] if m['role'] == 'user')
        except (ValueError, KeyError, StopIteration, TypeError):
            self.send_error(400)
            return
        recap = user.startswith('Generate a scene recap for this screenplay beat:')
        with self.records_lock:
            generations = sum(r.get('accepted', False) and r['kind'] == 'generation' for r in self.records)
            phase = 'recap' if recap else f'generation{generations + 1}'
            expected = self.expected.get(phase)
            exact = (user == f'Generate a scene recap for this screenplay beat:\n\nSCRIPT:\n{OUTPUT}\n\nProduce the scene recap now. Use the exact format specified. Be concise — aim for 100-150 tokens.'
                     if recap else expected is not None and exact_context(user, expected))
            prior_recap = any(r.get('accepted') and r['phase'] == 'recap' for r in self.records)
            accepted = (body.get('stream') is True and exact and phase in ('generation1', 'recap', 'generation2')
                        and (not recap or generations == 1) and (phase != 'generation2' or prior_recap)
                        and not any(r.get('accepted') and r['phase'] == phase for r in self.records))
            record = {'kind': 'recap' if recap else 'generation', 'phase': phase,
                      'ordered_serialized_context_exact': exact, 'accepted': accepted, 'stream': body.get('stream') is True,
                      'real_model': False, 'user_sha256': hashlib.sha256(user.encode()).hexdigest()}
            if expected is not None:
                record['expected_block_ids'] = [row[2] for row in expected]
                record['expected_section_sha256'] = hashlib.sha256(serialized_blocks(expected).encode()).hexdigest()
            self.records.append(record)
        if not accepted:
            self.send_error(422, 'Exact canonical HTTP fixture request required')
            return
        if phase == 'generation2':
            self.delayed_seen.set()
            if not self.release.wait(timeout=90):
                self.send_error(504, 'Native manual edit did not release fixture')
                return
        text = 'Synthetic recap: Mara waits beside the station.' if recap else DELAYED if phase == 'generation2' else OUTPUT
        data = (f'data: {json.dumps({"choices": [{"delta": {"content": text}}]})}\n\ndata: [DONE]\n\n').encode()
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)


def main():
    repo = Path.cwd()
    output = Path(os.environ['EIDETIC_CAPTURE_DIR']); output.mkdir(parents=True, exist_ok=True)
    state_root = Path(os.environ['RUNNER_TEMP']) / 'eidetic-canonical-scene-state'; state_root.mkdir(parents=True, exist_ok=True)
    raw_log = state_root / 'app-private.log'
    environment = os.environ.copy(); environment['EIDETIC_LAUNCHER_STATE_ROOT'] = str(state_root)
    for suffix in ('CACHE', 'CONFIG', 'DATA', 'STATE'):
        directory = state_root / 'dev' / ('xdg-' + suffix.lower()); directory.mkdir(parents=True, exist_ok=True)
        environment['XDG_' + suffix + '_HOME'] = str(directory)
    environment.update(NO_AT_BRIDGE='0', GTK_MODULES='atk-bridge')
    ui.Atspi.set_timeout(1500, 1500)
    started = time.monotonic()
    evidence = {'status': 'failed', 'application_source_sha': SOURCE,
                'qualification_sha': ui.command('git', 'rev-parse', 'HEAD'),
                'application_binary_sha256': ui.file_hash(repo / 'target/debug/eidetic-desktop'),
                'capture_label': 'prototype views', 'setup_route': 'public services prepare existing A/Sequence/Bible before GUI launch',
                'gui_route': 'real Tauri AT-SPI geometry/scroll and X11 create/select/notes/manual screenplay/generate/manual edit',
                'provider': 'synthetic localhost HTTP/SSE fixture through production client',
                'real_model_quality_qualified': False, 'project_open_count': 0, 'checkpoints': []}
    process = application = window = None
    provider = ThreadingHTTPServer(('127.0.0.1', 18080), CanonicalProvider)
    threading.Thread(target=provider.serve_forever, daemon=True).start()

    def checkpoint(stage):
        evidence['stage'] = stage
        evidence['checkpoints'].append({'stage': stage, 'elapsed_seconds': round(time.monotonic() - started, 3)})
        (output / 'capture-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')

    def capture(name):
        pid = int(ui.command('xdotool', 'getwindowpid', window))
        if pid != evidence['window_pid'] or os.getpgid(pid) != process.pid:
            raise RuntimeError('Capture window is not owned by this actual app')
        digest = ui.capture_native_window(window, output / name, screen_read=True)
        evidence.setdefault('captures', []).append({'file': name, 'sha256': digest, 'window_id': int(window), 'pid': pid})

    try:
        fixture = json.loads(subprocess.check_output([str(repo / 'target/debug/examples/canonical_scene_capture_fixture')],
                            env=environment, text=True, timeout=60))
        database = Path(fixture['path']); a_id = fixture['a']['id']; original_a = ui.blocks(database, a_id)
        initial_ids = {row[0] for row in ui.query(database, 'SELECT id FROM nodes')}
        with raw_log.open('w') as stream:
            process = subprocess.Popen(['./launcher.sh', '--run'], env=environment, stdout=stream, stderr=subprocess.STDOUT, start_new_session=True)
        def owned_window():
            if process.poll() is not None: raise RuntimeError('Launcher exited: ' + str(process.returncode))
            found = subprocess.run(['xdotool', 'search', '--onlyvisible', '--name', '^Eidetic$'], capture_output=True, text=True, timeout=5)
            return next(((window, int(ui.command('xdotool', 'getwindowpid', window))) for window in found.stdout.splitlines()
                         if os.getpgid(int(ui.command('xdotool', 'getwindowpid', window))) == process.pid), None)
        window, pid = ui.wait_for('owned native window', owned_window); ui.NATIVE_WINDOW_PID = pid
        evidence.update(window_id=int(window), window_pid=pid, application_binary_sha256=ui.file_hash(Path('/proc/' + str(pid) + '/exe')))
        application = ui.wait_for('native accessibility app', lambda: next((app for app in ui.pyatspi.Registry.getDesktop(0) if app.get_process_id() == pid), None))
        ui.open_project_chooser(application, window); ui.click_button(application, database.parent.name, window, prefix=True)
        evidence['project_open_count'] += 1
        ui.click_button(application, 'AI', window); ui.click_button(application, 'Save & Connect', window)
        ui.wait_for('connected synthetic HTTP fixture', lambda: ui.find(application, lambda n: 'Connected' in ui.text_of(n)))
        sequence = ui.wait_for('canonical Sequence clip', lambda: ui.native_timeline_clip(application, fixture['sequence']['name'], window))
        ui.click_control(sequence[0], window, sequence[1]); ui.reveal_button(application, 'Add Scene', window)
        b = ui.wait_for('new canonical GUI-created scene', lambda: next((row for row in ui.query(database,
            "SELECT id,parent_id,start_ms,end_ms,name FROM nodes WHERE level='Scene'") if row[0] not in initial_ids), None))
        if b[1:] != (fixture['sequence']['id'], 0, 600000, 'New Scene'):
            raise RuntimeError('GUI child did not use canonical selected parent/range')
        b_id = b[0]; evidence['creation'] = {'node_id': b_id, 'parent_id': b[1], 'start_ms': b[2], 'end_ms': b[3],
            'exact_name': b[4], 'route': 'native Add Scene in selected Sequence; automatic canonical child selection'}
        ui.wait_for('new scene selected in editor', lambda: ui.find(application, lambda n: n.getRole() == ui.pyatspi.ROLE_HEADING and n.name == 'New Scene'))
        notes = ui.wait_for('new scene Notes field', lambda: ui.reveal(application, lambda n: n.getState().contains(ui.pyatspi.STATE_EDITABLE) and (n.name or '').strip().casefold() == 'notes'))
        # WebKit exposes the CSS-transformed label as NOTES. Normal native
        # scrolling brings the complete textarea into the editor viewport.
        notes.queryComponent().scrollTo(ui.pyatspi.SCROLL_TOP_EDGE)
        evidence['notes_locator'] = {'accessible_name': notes.name, 'scroll_route': 'standard AT-SPI top-edge'}
        ui.type_text(notes, window, NOTES)
        ui.wait_for('exact canonical new-scene notes', lambda: json.loads(ui.query(database, 'SELECT content_json FROM nodes WHERE id=?', (b_id,))[0][0])['notes'] == NOTES)
        ui.reveal_button(application, 'Write screenplay', window)
        textarea = ui.wait_for('new screenplay editor', lambda: ui.reveal(application, lambda n: n.getState().contains(ui.pyatspi.STATE_EDITABLE) and n.name == 'New screenplay text'))
        ui.type_text(textarea, window, ANCHOR); ui.reveal_button(application, 'Save screenplay', window)
        anchor = ui.wait_for('exact canonical manual B anchor', lambda: next((row for row in ui.blocks(database, b_id) if row[1] == ANCHOR), None))
        ui.click_button(application, 'Bible', window)
        mara = ui.wait_for('Mara Bible control', lambda: ui.reveal(application, lambda n: n.getRole() == ui.pyatspi.ROLE_PUSH_BUTTON and ui.button_label_matches(n.name, 'Mara', prefix=True)))
        ui.click_control(mara, window)
        ui.wait_for('blue Bible field', lambda: ui.find(application, lambda n: n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n) == "Mara's umbrella is blue."))
        CanonicalProvider.expected['generation1'] = canonical_rows(database)
        if [row[-1] for row in CanonicalProvider.expected['generation1']] != [ANCHOR, A]: raise RuntimeError('Unexpected canonical initial context order')
        evidence['manual_anchor'] = {'exact_text': ANCHOR, 'revision_event_id': anchor[2]}
        checkpoint('native create/select/notes/manual anchor; no project reopen')
        capture('eidetic-canonical-scene-created.png')
        ui.reveal_button(application, 'Generate', window)
        generated = ui.wait_for('new scene canonical generated output', lambda: next((row for row in ui.blocks(database, b_id) if row[1] == OUTPUT), None))
        ui.wait_for('completed synthetic recap', lambda: any(r['phase'] == 'recap' and r['accepted'] for r in CanonicalProvider.records))
        command = ui.query(database, "SELECT payload_json FROM commands WHERE payload_type='script.generate_block' ORDER BY rowid DESC LIMIT 1")[0][0]
        binding = json.loads(command)['target_binding']
        if binding['node_id'] != b_id or binding['notes'] != NOTES or binding['segment_revision_event_id'] != anchor[2] or binding['block_revision_event_id'] is not None:
            raise RuntimeError('Generated command lost exact canonical admission custody')
        if ui.blocks(database, a_id) != original_a or anchor not in ui.blocks(database, b_id): raise RuntimeError('Initial generation changed human screenplay')
        helpers.enlarge_script_pane(application, window)
        ui.wait_for('visible canonical generated target', lambda: ui.screenplay_block(application, OUTPUT.strip()))
        evidence['generation'] = {'revision_event_id': generated[2], 'target_binding': binding, 'manual_anchor_unchanged': True,
                                  'A_unchanged': True, 'project_reopened_after_creation': False}
        checkpoint('native generation immediately after canonical scene creation; saved output visible')
        capture('eidetic-canonical-scene-generated.png')
        CanonicalProvider.expected['generation2'] = canonical_rows(database)
        if [row[-1] for row in CanonicalProvider.expected['generation2']] != [OUTPUT, ANCHOR, A]: raise RuntimeError('Unexpected regeneration context order')
        ui.reveal_button(application, 'Generate', window)
        ui.wait_for('paused second production HTTP request', CanonicalProvider.delayed_seen.is_set)
        block = ui.wait_for('generated B to manually edit during paused response', lambda: ui.screenplay_block(application, OUTPUT.strip()))
        ui.reveal_button(block, 'Edit', window)
        field = ui.wait_for('B editor', lambda: ui.editable(application, OUTPUT))
        ui.type_text(field, window, HUMAN)
        block = ui.wait_for('exact human draft during delayed response', lambda: ui.screenplay_block(application, HUMAN.strip()))
        ui.reveal_button(block, 'Save', window)
        human = ui.wait_for('saved exact manual replacement', lambda: next((row for row in ui.blocks(database, b_id) if row[1] == HUMAN), None))
        count_before = ui.query(database, "SELECT COUNT(*) FROM commands WHERE payload_type='script.generate_block'")[0][0]
        CanonicalProvider.release.set()
        ui.wait_for('visible stale generation refusal', lambda: ui.reveal(application, lambda n: 'generation target changed' in ui.text_of(n)))
        if (human not in ui.blocks(database, b_id) or anchor not in ui.blocks(database, b_id)
                or ui.blocks(database, a_id) != original_a or ui.query(database, "SELECT COUNT(*) FROM commands WHERE payload_type='script.generate_block'")[0][0] != count_before):
            raise RuntimeError('Delayed response changed human text/history')
        ui.wait_for('saved human canonical target', lambda: ui.screenplay_block(application, HUMAN.strip()))
        evidence['delayed_refusal'] = {'exact_text': HUMAN, 'revision_event_id': human[2], 'generation_commands_unchanged': True,
                                     'A_and_manual_anchor_unchanged': True, 'visible_native_refusal': True}
        checkpoint('native manual save during paused response; stale output refused and human canonical text retained')
        capture('eidetic-canonical-scene-refused.png')
        evidence['status'] = 'native_canonical_scene_generation_passed_with_synthetic_http_fixture'
    except Exception as error:
        evidence['failure'] = type(error).__name__ + ': ' + str(error)[:1000]
        if application is not None:
            try: evidence['ui_snapshot'] = ui.accessibility_snapshot(application)
            except Exception as snapshot_error: evidence['ui_snapshot_error'] = type(snapshot_error).__name__
            try: capture('eidetic-canonical-scene-failure.png')
            except Exception as capture_error: evidence['failure_capture_error'] = type(capture_error).__name__
        raise
    finally:
        CanonicalProvider.release.set()
        if process is not None:
            try: os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError: pass
            try: process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL); process.wait(timeout=5)
        provider.shutdown(); provider.server_close()
        evidence['provider_calls'] = CanonicalProvider.records
        evidence['typed_text_observations'] = ui.TYPED_TEXT_OBSERVATIONS
        raw = raw_log.read_text(errors='replace') if raw_log.exists() else 'No app launch log.\n'
        (output / 'app-sanitized.log').write_text(ui.sanitized_log(raw, [(str(repo), '<workspace>'), (str(state_root), '<capture-state>'), (os.environ.get('RUNNER_TEMP'), '<runner-temp>')]))
        (output / 'capture-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')


if __name__ == '__main__':
    main()
