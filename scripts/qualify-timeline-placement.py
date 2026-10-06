#!/usr/bin/env python3
"""Actual native exact placement authoring and targeted screenplay review.

Setup uses real public services and a synthetic production HTTP/SSE provider.
GUI manual save/preview/accept use AT-SPI geometry, scrolling and X11 input.
No pixel changes, injected JS/IPC, database writes, or model-quality claims.
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

SOURCE = '8c1956f7afa18a2a3ea3a1732fb8eca58a1129b3'
GENERATED = 'Synthetic generation: B waits at the station.\n\n'
MANUAL = 'Manual B: retain this station beat.\n\n'
PROPOSED = 'Synthetic preview: B follows the newly preceding E.\n\n'
DRAFT = 'Retained manual B placement draft.\n\n'


def walk_live(root):
    pending = [root]
    visited = 0
    while pending:
        node = pending.pop()
        visited += 1
        if visited > 3000:
            raise RuntimeError("Accessibility tree exceeds capture traversal bound")
        if node is None:
            ui.ACCESSIBILITY_RETIREMENTS += 1
            continue
        yield node
        pending.extend(reversed(list(node)))


ui.walk = walk_live


class SceneOrderProvider(ui.FixtureProvider):
    records = []
    records_lock = threading.Lock()

    def contains_expected_context(self, user, kind):
        if kind == 'recap':
            return GENERATED in user
        expected = ('F', 'E', 'C', 'D') if kind == 'preview' else ('A', 'F', 'C', 'D')
        excluded = 'A' if kind == 'preview' else 'E'
        return (all(f'Exact authored scene {letter}.\n\n' in user for letter in expected)
                and f'Exact authored scene {excluded}.\n\n' not in user
                and "profile.tagline: Mara's umbrella is blue." in user
                and (kind != 'preview' or (MANUAL in user and 'context_changed' in user)))

    def response_text(self, kind):
        return PROPOSED if kind == 'preview' else 'B waits at the station.' if kind == 'recap' else GENERATED


def scene_order(database):
    return [row[0] for row in ui.query(database, "SELECT name FROM nodes WHERE level='Scene' ORDER BY start_ms,sort_order,id")]


def scroll_script_start(application, window, exact_text):
    """Native horizontal wheel navigation in the actual multicolumn Script pane.

    AT-SPI ANYWHERE can leave a fragmented disclosure behind the sidebar. Use
    the known visible saved block as the pointer surface and real X11 left-wheel
    events before locating controls again; never click an occluded old rectangle.
    """
    block = ui.wait_for('visible saved Script pointer surface',
                        lambda: ui.screenplay_block(application, exact_text.strip()))
    rectangle = block.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)
    geometry = {key: int(value) for key, value in (line.split('=', 1) for line in
        ui.command('xdotool', 'getwindowgeometry', '--shell', window).splitlines())}
    point = ui.control_click_point(rectangle, geometry)
    ui.command('xdotool', 'windowfocus', '--sync', window)
    ui.command('xdotool', 'mousemove', str(point[0]), str(point[1]))
    ui.command('xdotool', 'click', '--repeat', '16', '--delay', '40', '6')
    application.clear_cache()


def enlarge_script_pane(application, window):
    """Use the existing focusable splitter's normal ArrowUp keyboard controls."""
    geometry = {key: int(value) for key, value in (line.split('=', 1) for line in
        ui.command('xdotool', 'getwindowgeometry', '--shell', window).splitlines())}
    def editor_splitter():
        application.clear_cache()
        candidates = []
        for node in ui.walk(application):
            if node.name != 'Resize panels' or not ui.visible(node):
                continue
            rectangle = tuple(node.queryComponent().getExtents(ui.pyatspi.XY_SCREEN))
            x, y, width, height = rectangle
            if 0 < height <= 8 and width > geometry['WIDTH'] // 2 and y < geometry['Y'] + geometry['HEIGHT'] // 2:
                ui.control_click_point(rectangle, geometry)
                candidates.append(node)
        if len(candidates) > 1:
            raise RuntimeError('Ambiguous native editor splitter')
        return candidates[0] if candidates else None
    splitter = ui.wait_for('existing editor/script splitter', editor_splitter)
    ui.click_control(splitter, window)
    def focused():
        splitter.clear_cache()
        return splitter.getState().contains(ui.pyatspi.STATE_FOCUSED)
    ui.wait_for('native splitter focus', focused)
    ui.command('xdotool', 'key', '--clearmodifiers', '--repeat', '8', '--delay', '40', 'Up')
    application.clear_cache()


def main():
    repo = Path.cwd()
    output = Path(os.environ['EIDETIC_CAPTURE_DIR'])
    output.mkdir(parents=True, exist_ok=True)
    state_root = Path(os.environ['RUNNER_TEMP']) / 'eidetic-exact-placement-state'
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
                'setup_route': 'public native services; six scenes and synthetic generation only',
                'gui_route': 'actual Tauri AT-SPI/X11 manual Save, retained draft, exact placement Apply, preview and explicit acceptance',
                'provider': 'synthetic localhost HTTP/SSE fixture through production client',
                'real_model_quality_qualified': False, 'checkpoints': []}
    process = application = window = None
    provider = ThreadingHTTPServer(('127.0.0.1', 18080), SceneOrderProvider)
    threading.Thread(target=provider.serve_forever, daemon=True).start()

    def checkpoint(stage):
        evidence['stage'] = stage
        evidence['checkpoints'].append({'stage': stage, 'elapsed_seconds': round(time.monotonic() - started, 3)})
        (output / 'capture-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')
        print('Scene order checkpoint: ' + stage, flush=True)

    def capture(name):
        pid = int(ui.command('xdotool', 'getwindowpid', window))
        if process.poll() is not None or os.getpgid(pid) != process.pid:
            raise RuntimeError('Native window ownership changed')
        digest = ui.capture_native_window(window, output / name, screen_read=True)
        evidence.setdefault('captures', []).append({'file': name, 'sha256': digest, 'window_id': int(window), 'pid': pid})

    try:
        checkpoint('public services prepare original six-scene window')
        fixture = json.loads(subprocess.check_output([str(repo / 'target/debug/examples/timeline_placement_capture_fixture')],
                                                    env=environment, text=True, timeout=60))
        database = Path(fixture['project_path']).resolve()
        b_id = fixture['b']['id']
        evidence['reorder'] = {key: fixture[key] for key in ('before_order', 'after_order', 'reorder_route')}
        expected_order = ['SCENE ' + letter for letter in fixture['before_order']]
        # Public fixture asserts membership and unchanged target; this read independently confirms order.
        actual_order = scene_order(database)
        if actual_order != expected_order:
            raise RuntimeError('Actual saved scene order differs: ' + repr(actual_order))
        original = {s['id']: ui.blocks(database, s['id']) for s in fixture['scenes']}
        if original[b_id][0][1] != GENERATED:
            raise RuntimeError('Unexpected initial B')
        checkpoint('launch actual desktop and save exact human B text')
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
        enlarge_script_pane(application, window)
        block = ui.wait_for('generated B in native Script panel', lambda: ui.screenplay_block(application, GENERATED.strip()))
        ui.reveal_button(block, 'Edit', window)
        field = ui.wait_for('B editor', lambda: ui.editable(application, GENERATED))
        ui.type_text(field, window, MANUAL)
        block = ui.wait_for('exact human draft B', lambda: ui.screenplay_block(application, MANUAL.strip()))
        ui.reveal_button(block, 'Save', window)
        b = ui.wait_for('saved exact human B', lambda: next((row for row in ui.blocks(database, b_id) if row[1] == MANUAL), None))
        evidence['manual_edit'] = {'exact_text': MANUAL, 'revision_event_id': b[2]}
        block = ui.wait_for('saved manual B block', lambda: ui.screenplay_block(application, MANUAL.strip()))
        ui.reveal_button(block, 'Edit', window)
        field = ui.wait_for('retained B draft editor', lambda: ui.editable(application, MANUAL))
        ui.type_text(field, window, DRAFT)
        ui.reveal_button(application, 'Zoom to Fit (Ctrl+0)', window)
        for _ in range(3):
            ui.reveal_button(application, '+', window)
        e_id = fixture['e']['id']
        e_clip = ui.wait_for('native E timeline clip', lambda: ui.native_timeline_clip(application, fixture['e']['name'], window))
        ui.click_control(e_clip[0], window, e_clip[1])
        disclosure = ui.wait_for('exact placement disclosure', lambda: ui.reveal(application,
            lambda n: n.name == 'Placement: 540–570 seconds'))
        ui.click_control(disclosure, window)
        start = ui.wait_for('native placement start field', lambda: ui.reveal(application,
            lambda n: n.getState().contains(ui.pyatspi.STATE_EDITABLE) and n.name == 'Placement start seconds'))
        ui.type_text(start, window, '180')
        end = ui.wait_for('native placement end field', lambda: ui.reveal(application,
            lambda n: n.getState().contains(ui.pyatspi.STATE_EDITABLE) and n.name == 'Placement end seconds'))
        ui.type_text(end, window, '210')
        ui.reveal_button(application, 'Apply placement', window)
        placed = ui.wait_for('exact canonical placement', lambda: next((row for row in ui.query(database,
            'SELECT start_ms,end_ms FROM nodes WHERE id=?', (e_id,)) if row == (180000, 210000)), None))
        expected_order = ['SCENE ' + letter for letter in fixture['after_order']]
        if scene_order(database) != expected_order:
            raise RuntimeError('Native exact placement did not reorder across neighbors')
        if ui.blocks(database, b_id)[0] != b or ui.blocks(database, e_id) != original[e_id]:
            raise RuntimeError('Placement changed saved manual screenplay')
        ui.wait_for('retained exact B draft after placement', lambda: ui.reveal(application,
            lambda n: n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n) == DRAFT))
        ui.wait_for('placement acknowledged in native UI', lambda: ui.find(application,
            lambda n: 'Placement saved. Review affected screenplay before replacing it.' in ui.text_of(n)))
        evidence['placement'] = {'source_node_id': e_id, 'before_ms': [540000,570000], 'after_ms': list(placed),
            'before_order': fixture['before_order'], 'after_order': fixture['after_order'],
            'native_route': 'selected clip exact start/end seconds then Apply placement',
            'saved_manual_text_preserved': True, 'retained_draft_preserved': True}
        checkpoint('native exact placement crosses neighbors and preserves saved text and draft')
        capture('eidetic-timeline-placement-authored.png')
        block = ui.wait_for('retained manual draft block', lambda: ui.screenplay_block(application, DRAFT.strip()))
        ui.reveal_button(block, 'Cancel', window)
        if ui.blocks(database, b_id)[0] != b:
            raise RuntimeError('Explicit draft cancellation changed saved screenplay')
        ui.click_button(application, 'AI', window)
        ui.click_button(application, 'Save & Connect', window)
        ui.wait_for('connected HTTP fixture', lambda: ui.find(application, lambda n: 'Connected' in ui.text_of(n)))
        ui.click_button(application, 'Bible', window)
        mara = ui.wait_for('Mara Bible control', lambda: ui.reveal(application, lambda n: n.getRole() == ui.pyatspi.ROLE_PUSH_BUTTON and ui.button_label_matches(n.name, 'Mara', prefix=True)))
        ui.click_control(mara, window)
        ui.wait_for('blue Bible fact', lambda: ui.reveal(application, lambda n: n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n) == "Mara's umbrella is blue."))
        scroll_script_start(application, window, MANUAL)
        ui.wait_for('context cause in native review UI', lambda: ui.find(application, lambda n: 'Screenplay context changed.' in ui.text_of(n)))
        # The existing What changed disclosure shows the exact entering/displaced scenes.
        disclosure = ui.wait_for('What changed disclosure', lambda: ui.find(application, lambda n: n.name == 'What changed'))
        ui.click_control(disclosure, window)
        ui.wait_for('exact scene membership explanation', lambda: ui.find(application, lambda n: 'Entered: SCENE E. Left: SCENE A.' in ui.text_of(n)))
        checkpoint('native downstream review; human B and actual order preserved')
        capture('eidetic-timeline-placement-review.png')
        ui.reveal_button(application, 'Preview update', window)
        proposal = ui.wait_for('pending proposal', lambda: next(iter(ui.query(database, "SELECT id,status,proposed_text FROM propagation_proposals WHERE target_id=?", (b[0],))), None))
        if proposal[1:] != ('pending', PROPOSED) or ui.blocks(database, b_id)[0] != b:
            raise RuntimeError('Preview changed saved human B')
        ui.require_accepted_preview(SceneOrderProvider.records)
        ui.wait_for('exact proposed text', lambda: ui.reveal(application, lambda n: n.name == 'Proposed text' and ui.text_of(n) == PROPOSED))
        evidence['preview'] = {'proposal_id': proposal[0], 'target_unchanged': True, 'entered_E_consumed_and_displaced_A_excluded': True}
        checkpoint('pending fresh-window preview; explicit acceptance required')
        capture('eidetic-timeline-placement-preview.png')
        ui.reveal_button(application, 'Accept update', window)
        accepted = ui.wait_for('accepted B', lambda: next((row for row in ui.blocks(database, b_id) if row[1] == PROPOSED and row[2] != b[2]), None))
        for node_id, rows in original.items():
            if node_id != b_id and ui.blocks(database, node_id) != rows:
                raise RuntimeError('Acceptance changed unrelated human screenplay')
        def cleared():
            application.clear_cache()
            return not any(n.name == 'Screenplay needs review' for n in ui.walk(application))
        ui.wait_for('old review cleared after UI refresh', cleared)
        # Normal native accessibility scrolling brings the canonical target block into view.
        ui.wait_for('accepted canonical B after normal scrolling', lambda: ui.screenplay_block(application, PROPOSED.strip()))
        evidence['acceptance'] = {'revision_event_id': accepted[2], 'only_B_changed': True,
                                  'old_review_cleared_in_ui': True, 'canonical_target_revealed_by_normal_scroll': True}
        checkpoint('explicit native acceptance commits only B and reveals canonical target')
        capture('eidetic-timeline-placement-accepted.png')
        evidence['status'] = 'native_exact_timeline_placement_passed_with_synthetic_http_fixture'
    except Exception as error:
        evidence['failure'] = type(error).__name__ + ': ' + str(error)[:1000]
        if application is not None:
            try:
                evidence['accessibility_snapshot'] = ui.accessibility_snapshot(application)
                capture('eidetic-timeline-placement-failure.png')
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
        evidence['provider_calls'] = SceneOrderProvider.records
        evidence['accessibility_retired_object_polls'] = ui.ACCESSIBILITY_RETIREMENTS
        evidence['typed_text_observations'] = ui.TYPED_TEXT_OBSERVATIONS
        raw = raw_log.read_text(errors='replace') if raw_log.exists() else 'No app launch log.'
        (output / 'app-sanitized.log').write_text(ui.sanitized_log(raw, [(str(repo), '<workspace>'), (str(state_root), '<capture-state>'), (os.environ.get('RUNNER_TEMP'), '<runner-temp>')]))
        (output / 'capture-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')


if __name__ == '__main__':
    main()
