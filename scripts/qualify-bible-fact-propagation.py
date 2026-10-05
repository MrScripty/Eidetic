#!/usr/bin/env python3
"""Real Tauri Bible edit -> consumed fact review -> explicit screenplay accept.

Uses the existing AT-SPI/X11 driver helpers and production HTTP fixture boundary.
No injected JS/IPC, database writes, sandbox overrides or real-model claims.
"""
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import threading
import time
from http.server import ThreadingHTTPServer

spec = importlib.util.spec_from_file_location('authoring_capture', Path(__file__).with_name('qualify-screenplay-authoring.py'))
ui = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ui)

SOURCE = 'a66e1c74e164178208bd3e63c9ab99147d5290de'
RED = "Mara's umbrella is red."
BLUE = "Mara's umbrella is blue."
DRAFT = 'Retained manual draft. The train waits.\n\n'


class BibleFactProvider(ui.FixtureProvider):
    """Synthetic responses; require exact canonical fact and manual screenplay."""
    records = []
    records_lock = threading.Lock()

    def contains_expected_context(self, user, kind):
        if kind == 'recap':
            return ui.GENERATED_TEXT in user
        fact = BLUE if kind == 'preview' else RED
        return (f'profile.tagline: {fact}' in user and ui.MANUAL_TEXT in user
                and (kind != 'preview' or ui.GENERATED_TEXT in user))


def save_fact_control(application, field):
    """Identify the field's immediately following Save by native control geometry."""
    x, y, width, height = field.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)
    candidates = []
    for node in ui.walk(application):
        if node.getRole() != ui.pyatspi.ROLE_PUSH_BUTTON or node.name != 'Save' or not ui.visible(node):
            continue
        sx, sy, sw, sh = node.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)
        if x <= sx < x + width and y + height <= sy <= y + height + 70:
            candidates.append((sy, node))
    if not candidates:
        return None
    candidates.sort(key=lambda item: item[0])
    if len(candidates) > 1 and candidates[0][0] == candidates[1][0]:
        raise RuntimeError('Ambiguous Bible field Save geometry')
    return candidates[0][1]


def fact(database):
    return ui.query(database, "SELECT text_value,updated_event_id FROM bible_graph_fields WHERE id='qualification.mara.tagline'")[0]


def main():
    repo = Path.cwd()
    output = Path(os.environ['EIDETIC_CAPTURE_DIR'])
    output.mkdir(parents=True, exist_ok=True)
    state_root = Path(os.environ['RUNNER_TEMP']) / 'eidetic-bible-fact-state'
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
                'interaction_route': 'Real Tauri; existing AT-SPI geometry and X11 input',
                'provider': 'synthetic localhost HTTP/SSE fixture through production provider client',
                'real_model_quality_qualified': False, 'checkpoints': []}
    process = application = window = None
    provider = ThreadingHTTPServer(('127.0.0.1', 18080), BibleFactProvider)
    threading.Thread(target=provider.serve_forever, daemon=True).start()

    def checkpoint(stage):
        evidence['stage'] = stage
        evidence['checkpoints'].append({'stage': stage, 'elapsed_seconds': round(time.monotonic() - started, 3)})
        (output / 'capture-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')
        print('Bible fact checkpoint: ' + stage, flush=True)

    def capture(name):
        pid = int(ui.command('xdotool', 'getwindowpid', window))
        if process.poll() is not None or os.getpgid(pid) != process.pid:
            raise RuntimeError('Native window ownership changed')
        path = output / name
        digest = ui.capture_native_window(window, path, screen_read=True)
        evidence.setdefault('captures', []).append({'file': name, 'sha256': digest, 'window_id': int(window), 'pid': pid})

    try:
        checkpoint('public service setup; screenplay initially empty')
        fixture = json.loads(subprocess.check_output([str(repo / 'target/debug/examples/bible_fact_capture_fixture')],
                                                    env=environment, text=True, timeout=20))
        database = Path(fixture['project_path']).resolve()
        a_id, b_id = fixture['a']['id'], fixture['b']['id']
        original_fact = fact(database)
        if original_fact[0] != RED or ui.blocks(database, a_id) or ui.blocks(database, b_id):
            raise RuntimeError('Unexpected setup canon')
        checkpoint('launch actual desktop and author exact manual screenplay')
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
        application = ui.wait_for('native accessibility app', lambda: next((app for app in ui.pyatspi.Registry.getDesktop(0) if app.get_process_id() == pid), None))
        ui.open_project_chooser(application, window)
        ui.click_button(application, database.parent.name, window, prefix=True)
        scene = ui.wait_for('first scene', lambda: ui.native_timeline_clip(application, fixture['a']['name'], window))
        ui.click_control(scene[0], window, scene[1])
        ui.click_button(application, 'Write screenplay', window)
        field = ui.wait_for('composer', lambda: ui.find(application, lambda n: n.getState().contains(ui.pyatspi.STATE_EDITABLE) and n.name == 'New screenplay text'))
        ui.type_text(field, window, ui.MANUAL_TEXT)
        ui.reveal_button(application, 'Save screenplay', window)
        a = ui.wait_for('manual canon', lambda: next((row for row in ui.blocks(database, a_id) if row[1] == ui.MANUAL_TEXT), None))
        checkpoint('generate linked scene consuming exact red Bible field')
        ui.click_button(application, 'AI', window)
        ui.click_button(application, 'Save & Connect', window)
        ui.wait_for('connected fixture', lambda: ui.find(application, lambda n: 'Connected' in ui.text_of(n)))
        scene = ui.wait_for('second scene', lambda: ui.native_timeline_clip(application, fixture['b']['name'], window))
        ui.click_control(scene[0], window, scene[1])
        ui.click_button(application, 'Generate', window)
        b = ui.wait_for('generated canon', lambda: next((row for row in ui.blocks(database, b_id) if row[1] == ui.GENERATED_TEXT), None))
        dependencies = ui.query(database, "SELECT d.id,r.target_revision_event_id FROM semantic_dependencies d JOIN semantic_dependency_revisions r ON r.dependency_id=d.id WHERE d.target_field_id='qualification.mara.tagline'")
        if len(dependencies) != 1 or dependencies[0][1] != original_fact[1]:
            raise RuntimeError('Generation did not persist consumed Bible revision')
        evidence['consumed_fact'] = {'dependency_id': dependencies[0][0], 'revision_event_id': original_fact[1], 'exact_value': RED}
        # Retain a human draft while the Bible event refreshes the screenplay.
        block = ui.wait_for('manual screenplay block', lambda: ui.screenplay_block(application, 'red umbrella'))
        ui.click_button(block, 'Edit', window)
        draft = ui.wait_for('manual draft', lambda: ui.editable(application, ui.MANUAL_TEXT))
        ui.type_text(draft, window, DRAFT)
        checkpoint('manually edit exact Bible fact through native field and Save')
        ui.click_button(application, 'Bible', window)
        mara = ui.wait_for('Mara Bible control', lambda: ui.reveal(application, lambda n: n.getRole() == ui.pyatspi.ROLE_PUSH_BUTTON and ui.button_label_matches(n.name, 'Mara', prefix=True)))
        ui.click_control(mara, window)
        field = ui.wait_for('red Bible field', lambda: ui.reveal(application, lambda n: n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n) == RED))
        ui.type_text(field, window, BLUE)
        save = ui.wait_for('associated Bible Save', lambda: save_fact_control(application, field))
        ui.click_control(save, window)
        edited = ui.wait_for('canonical blue Bible field', lambda: fact(database) if fact(database)[0] == BLUE and fact(database)[1] != original_fact[1] else None)
        if ui.blocks(database, a_id)[0] != a or ui.blocks(database, b_id)[0] != b:
            raise RuntimeError('Fact save replaced screenplay')
        ui.wait_for('retained manual draft after Bible refresh', lambda: ui.editable(application, DRAFT))
        evidence['fact_edit'] = {'before': RED, 'after': BLUE, 'revision_event_id': edited[1], 'saved_screenplay_preserved': True, 'manual_draft_preserved': True}
        # Reload the canonical manual block explicitly; the exact saved A is unchanged.
        block = ui.wait_for('retained draft block', lambda: ui.screenplay_block(application, 'Retained manual draft'))
        ui.click_button(block, 'Discard draft and reload', window)
        ui.wait_for('Bible review cause', lambda: ui.reveal(application, lambda n: 'Bible fact profile.tagline changed.' in ui.text_of(n)))
        checkpoint('downstream review with Bible timeline and screenplay visible')
        capture('eidetic-bible-fact-review.png')
        ui.reveal_button(application, 'Preview update', window)
        proposal = ui.wait_for('pending proposal', lambda: next(iter(ui.query(database, "SELECT id,status,proposed_text FROM propagation_proposals WHERE target_id=?", (b[0],))), None))
        if proposal[1:] != ('pending', ui.PROPOSED_TEXT) or ui.blocks(database, b_id)[0] != b:
            raise RuntimeError('Preview changed saved target')
        ui.require_accepted_preview(BibleFactProvider.records)
        evidence['preview'] = {'proposal_id': proposal[0], 'target_unchanged': True, 'current_bible_fact_and_exact_manual_context_consumed': True}
        checkpoint('pending targeted preview; explicit acceptance still required')
        capture('eidetic-bible-fact-preview.png')
        ui.reveal_button(application, 'Accept update', window)
        accepted = ui.wait_for('accepted target', lambda: next((row for row in ui.blocks(database, b_id) if row[1] == ui.PROPOSED_TEXT and row[2] != b[2]), None))
        if ui.blocks(database, a_id)[0] != a or fact(database) != edited:
            raise RuntimeError('Accept changed source screenplay or Bible')
        bindings = ui.query(database, "SELECT r.target_revision_event_id FROM semantic_dependencies d JOIN semantic_dependency_revisions r ON r.dependency_id=d.id WHERE d.target_field_id='qualification.mara.tagline' AND r.source_revision_event_id=?", (accepted[2],))
        if bindings != [(edited[1],)]:
            raise RuntimeError('Acceptance did not refresh actual Bible consumption')
        evidence['acceptance'] = {'revision_event_id': accepted[2], 'manual_input_and_fact_unchanged': True, 'refreshed_consumed_fact_revision': edited[1]}
        checkpoint('explicit acceptance commits only targeted screenplay')
        capture('eidetic-bible-fact-accepted.png')
        evidence['status'] = 'native_bible_fact_propagation_passed_with_synthetic_http_fixture'
    except Exception as error:
        evidence['failure'] = type(error).__name__ + ': ' + str(error)[:1000]
        if application is not None:
            try:
                evidence['accessibility_snapshot'] = ui.accessibility_snapshot(application)
                capture('eidetic-bible-fact-failure.png')
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
        evidence['provider_calls'] = BibleFactProvider.records
        evidence['typed_text_observations'] = ui.TYPED_TEXT_OBSERVATIONS
        evidence['timeline_locator_observations'] = ui.TIMELINE_LOCATOR_OBSERVATIONS
        evidence['accessibility_retired_object_polls'] = ui.ACCESSIBILITY_RETIREMENTS
        raw = raw_log.read_text(errors='replace') if raw_log.exists() else 'No app launch log.'
        (output / 'app-sanitized.log').write_text(ui.sanitized_log(raw, [(str(repo), '<workspace>'), (str(state_root), '<capture-state>'), (os.environ.get('RUNNER_TEMP'), '<runner-temp>')]))
        (output / 'capture-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')


if __name__ == '__main__':
    main()
