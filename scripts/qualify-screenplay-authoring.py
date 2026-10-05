#!/usr/bin/env python3
"""Drive the actual Tauri window through accessibility geometry and X11 input.

Fail closed if the app, accessibility tree, canonical edit, or capture is absent.
Never inject JavaScript, replace IPC, disable a sandbox, or capture a browser preview.
"""
import faulthandler
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import threading
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import sqlite3
import struct
import subprocess
import time

import pyatspi
from gi.repository import Atspi, GLib


SOURCE = "72ae4806ef70a8465e6fe26f5ce8e891b0dfb8f1"
MANUAL_TEXT = "INT. CAFE - NIGHT\n\nMara carries a red umbrella.\n\nELI\nThe last train is still waiting."
EDITED_TEXT = "INT. CAFE - NIGHT\n\nMara carries a blue umbrella.\n\nELI\nThe last train is still waiting."
GENERATED_TEXT = "EXT. STATION - NIGHT\n\nEli spots Mara and her red umbrella."
PROPOSED_TEXT = "EXT. STATION - NIGHT\n\nEli spots Mara and her blue umbrella."
DEADLINE = time.monotonic() + 270
NATIVE_WINDOW_PID = None
ACCESSIBILITY_RETIREMENTS = 0


def command(*args):
    return subprocess.check_output(args, text=True, timeout=15).strip()


def wait_for(description, check):
    global ACCESSIBILITY_RETIREMENTS
    while time.monotonic() < DEADLINE:
        if NATIVE_WINDOW_PID is not None and not Path(f'/proc/{NATIVE_WINDOW_PID}').exists():
            raise RuntimeError('Native application exited during qualification')
        try:
            found = check()
        except GLib.Error as error:
            # WebKit retires DOM accessibility objects while switching views.
            # Reacquire from the refreshed root on the next bounded poll only.
            if (error.domain != 'atspi_error' or error.code != 0
                    or 'application no longer exists' not in error.message.lower()):
                raise
            ACCESSIBILITY_RETIREMENTS += 1
            found = None
        if found:
            return found
        time.sleep(0.25)
    raise RuntimeError(f"Timed out waiting for {description}")


def walk(root):
    # Bound traversal even if a broken accessibility tree cycles or expands.
    pending = [root]
    visited = 0
    while pending:
        node = pending.pop()
        visited += 1
        if visited > 3000:
            raise RuntimeError("Accessibility tree exceeds capture traversal bound")
        yield node
        pending.extend(reversed(list(node)))


def visible(node):
    return node.getState().contains(pyatspi.STATE_SHOWING)


def find(root, predicate):
    root.clear_cache()
    for node in walk(root):
        if visible(node) and predicate(node):
            return node
    return None


def button_label_matches(name, label, prefix=False):
    name = " ".join((name or "").split())
    # The real splash button includes its folder-icon span in the accessible name.
    if label == "Open Project":
        name = name.removeprefix("\U0001f4c2").strip()
    return name == label or prefix and name.startswith(label)


def control_click_point(rect, geometry):
    x, y, width, height = rect
    left, top = geometry["X"], geometry["Y"]
    if (width <= 0 or height <= 0 or x < left or y < top
            or x + width > left + geometry["WIDTH"]
            or y + height > top + geometry["HEIGHT"]):
        raise RuntimeError("Control bounds are outside the verified native window")
    return x + width // 2, y + height // 2


def click_control(node, window):
    rect = node.queryComponent().getExtents(pyatspi.XY_SCREEN)
    geometry = dict(line.split("=", 1) for line in command(
        "xdotool", "getwindowgeometry", "--shell", window).splitlines())
    point = control_click_point(rect, {key: int(value) for key, value in geometry.items()})
    print(f"Native pointer input: control={(node.name or '')[:100]!r}, bounds={tuple(rect)}, point={point}",
          flush=True)
    command("xdotool", "windowfocus", "--sync", window)
    command("xdotool", "mousemove", str(point[0]), str(point[1]))
    # The hosted --sync move stalled on a repeated target coordinate.
    # Verify exact native pointer coordinates before clicking instead.
    def pointer_positioned():
        location = dict(line.split("=", 1) for line in command(
            "xdotool", "getmouselocation", "--shell").splitlines())
        return (int(location["X"]), int(location["Y"])) == point
    wait_for("native pointer at verified control", pointer_positioned)
    command("xdotool", "click", "1")


def click_button(root, label, window, prefix=False):
    node = wait_for(
        f"visible button {label}",
        lambda: find(root, lambda node: node.getRole() == pyatspi.ROLE_PUSH_BUTTON
                     and button_label_matches(node.name, label, prefix)),
    )
    click_control(node, window)


def open_project_chooser(application, window):
    def chooser_ready():
        if find(application, lambda node: node.getRole() == pyatspi.ROLE_HEADING
                and node.name == "Open Project"):
            return True
        # Vite first renders HTML, then loads/hydrates the client controls.
        # A click before hydration has no handler, so retry only the home button
        # until the actual chooser transition is visible, within the same deadline.
        button = find(application, lambda node: node.getRole() == pyatspi.ROLE_PUSH_BUTTON
                      and button_label_matches(node.name, "Open Project"))
        if button:
            click_control(button, window)
        time.sleep(0.75)
        return False
    wait_for("native project chooser transition", chooser_ready)


def text_of(node):
    try:
        text = node.queryText()
        return text.getText(0, min(text.characterCount, 4096))
    except NotImplementedError:
        return node.name or ""


def file_hash(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def validate_capture_png(path):
    size = path.stat().st_size
    if size > 5 * 1024**2:
        path.unlink()
        raise RuntimeError(f"Native PNG exceeds the 5 MiB upload bound: {size} bytes")
    with path.open("rb") as stream:
        header = stream.read(24)
    if (len(header) != 24
            or header[:8] != b"\x89PNG\r\n\x1a\n" or header[12:16] != b"IHDR"):
        raise RuntimeError(f"Invalid or oversized native PNG: {size} bytes")
    width, height = struct.unpack(">II", header[16:24])
    if not (1024 <= width <= 4096 and 720 <= height <= 4096):
        raise RuntimeError(f"Native PNG dimensions outside main-window bounds: {width}x{height}")
    return width, height


def capture_native_window(window, path, screen_read=False):
    arguments = ["import", "-window", window]
    if screen_read:
        arguments.append("-screen")
    subprocess.run([*arguments, str(path)], check=True, timeout=10)
    validate_capture_png(path)
    return file_hash(path)


def sanitized_log(raw, replacements):
    raw = re.sub(r"\x1b\[[0-9;]*[A-Za-z]", "", raw)
    result = []
    for line in raw.splitlines():
        # Keep launch diagnostics; exclude environment dumps, URLs and credential lines.
        if re.search(r"(?i)token|authorization|password|secret|api.?key|https?://", line):
            continue
        for value, label in replacements:
            if value:
                line = line.replace(value, label)
        if line.strip():
            result.append(line[:1000])
    return "\n".join(result[-100:])[-20000:] + "\n"


def accessibility_snapshot(root):
    result = []
    for node in walk(root):
        result.append({"role": node.getRoleName(), "name": (node.name or "")[:160],
                       "showing": visible(node)})
        if len(result) == 80:
            break
    return result


class FixtureProvider(BaseHTTPRequestHandler):
    """Explicit localhost HTTP fixture; the application's real provider client consumes SSE."""
    records = []

    def log_message(self, *args):
        pass

    def do_GET(self):
        if self.path != '/v1/models':
            self.send_error(404)
            return
        data = json.dumps({'data': [{'id': 'authoring-http-fixture'}]}).encode()
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_POST(self):
        size = int(self.headers.get('Content-Length', '0'))
        if self.path != '/v1/chat/completions' or not 0 < size <= 512_000:
            self.send_error(400)
            return
        body = json.loads(self.rfile.read(size))
        messages = body.get('messages', [])
        system = next((m['content'] for m in messages if m.get('role') == 'system'), '')
        user = next((m['content'] for m in messages if m.get('role') == 'user'), '')
        preview = 'targeted screenplay update' in system
        recap = user.startswith('Generate a scene recap for this screenplay beat:')
        kind = 'preview' if preview else 'recap' if recap else 'generation'
        required = EDITED_TEXT if preview else GENERATED_TEXT if recap else MANUAL_TEXT
        record = {'kind': kind,
                  'contains_exact_expected_context': required in user,
                  'stream': body.get('stream') is True,
                  'real_model': False}
        self.records.append(record)
        if sum(r['kind'] == kind for r in self.records) != 1 or len(self.records) > 3 or not record['stream'] or required not in user:
            self.send_error(422, 'Qualification prompt did not contain exact authored context')
            return
        text = PROPOSED_TEXT if preview else 'Eli waits at the station with Mara and the red umbrella.' if recap else GENERATED_TEXT
        parts = [text[:len(text)//2], text[len(text)//2:]]
        data = ''.join('data: ' + json.dumps({'choices': [{'delta': {'content': part}}]})
                       + '\n\n' for part in parts).encode() + b'data: [DONE]\n\n'
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)


def query(database, sql, parameters=()):
    with sqlite3.connect(database.as_uri() + '?mode=ro', uri=True, timeout=2) as conn:
        if not conn.execute("SELECT 1 FROM sqlite_master WHERE name='script_blocks'").fetchone():
            return []
        return conn.execute(sql, parameters).fetchall()


def blocks(database, node_id):
    return query(database, 'SELECT b.id,b.text,b.updated_event_id,s.start_ms,s.end_ms '
                 'FROM script_blocks b JOIN script_segments s ON s.id=b.segment_id '
                 'WHERE s.source_node_id=? AND b.deleted_event_id IS NULL '
                 'AND s.deleted_event_id IS NULL ORDER BY b.sort_order,b.id', (node_id,))


def choose_mode(application, label, window):
    node = wait_for('workspace ' + label, lambda: find(application, lambda n:
        n.getRole() == pyatspi.ROLE_TOGGLE_BUTTON and n.name == label))
    click_control(node, window)


def editable(application, expected):
    return find(application, lambda n: n.getState().contains(pyatspi.STATE_EDITABLE)
                and text_of(n) == expected)


def type_text(node, window, text):
    click_control(node, window)
    def focused():
        node.clear_cache()
        return node.getState().contains(pyatspi.STATE_FOCUSED)
    wait_for('native text focus', focused)
    command('xdotool', 'key', '--clearmodifiers', 'ctrl+a')
    subprocess.run(['xdotool', 'type', '--clearmodifiers', '--delay', '10', text],
                   check=True, timeout=20)
    wait_for('exact typed text', lambda: text_of(node) == text)


def reveal(application, predicate):
    """Standard AT-SPI scrolling; input is still native X11, never injected JS."""
    application.clear_cache()
    node = next((n for n in walk(application) if predicate(n)), None)
    if node:
        node.queryComponent().scrollTo(pyatspi.SCROLL_ANYWHERE)
    return node if node and visible(node) else None


def reveal_button(application, label, window):
    node = wait_for('reachable button ' + label, lambda: reveal(application,
        lambda n: n.getRole() == pyatspi.ROLE_PUSH_BUTTON and n.name == label))
    click_control(node, window)


def screenplay_block(application, excerpt):
    return reveal(application, lambda n: n.name == 'Screenplay block' and any(
        excerpt in text_of(child) for child in walk(n)))


def main():
    global NATIVE_WINDOW_PID
    repo = Path.cwd()
    output = Path(os.environ['EIDETIC_CAPTURE_DIR'])
    output.mkdir(parents=True, exist_ok=True)
    state_root = Path(os.environ['RUNNER_TEMP']) / 'eidetic-authoring-state'
    state_root.mkdir(parents=True, exist_ok=True)
    raw_log = state_root / 'app-private.log'
    environment = os.environ.copy()
    environment['EIDETIC_LAUNCHER_STATE_ROOT'] = str(state_root)
    for suffix in ('CACHE', 'CONFIG', 'DATA', 'STATE'):
        directory = state_root / 'dev' / ('xdg-' + suffix.lower())
        directory.mkdir(parents=True, exist_ok=True)
        environment['XDG_' + suffix + '_HOME'] = str(directory)
    environment.update(NO_AT_BRIDGE='0', GTK_MODULES='atk-bridge')
    Atspi.set_timeout(1500, 1500)
    started = time.monotonic()
    evidence = {'status': 'failed', 'application_source_sha': SOURCE,
                'qualification_sha': command('git', 'rev-parse', 'HEAD'),
                'application_binary_sha256': file_hash(repo / 'target/debug/eidetic-desktop'),
                'interaction_route': 'Real Tauri; AT-SPI geometry/scroll and X11 input',
                'provider': 'localhost HTTP/SSE fixture through production provider client',
                'real_model_quality_qualified': False, 'checkpoints': []}
    process = application = window = None
    provider = ThreadingHTTPServer(('127.0.0.1', 18080), FixtureProvider)
    provider_thread = threading.Thread(target=provider.serve_forever, daemon=True)
    provider_thread.start()
    faulthandler.dump_traceback_later(250)

    def checkpoint(stage):
        evidence['stage'] = stage
        evidence['checkpoints'].append({'stage': stage,
            'utc': datetime.now(timezone.utc).isoformat(timespec='milliseconds'),
            'elapsed_seconds': round(time.monotonic() - started, 3)})
        (output / 'capture-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')
        print('Authoring checkpoint: ' + stage, flush=True)

    def capture(name):
        actual_pid = int(command('xdotool', 'getwindowpid', window))
        if process.poll() is not None or os.getpgid(actual_pid) != process.pid:
            raise RuntimeError('Native window ownership changed')
        path = output / name
        digest = capture_native_window(window, path, screen_read=True)
        evidence.setdefault('captures', []).append({'file': name, 'sha256': digest,
            'window_id': int(window), 'pid': actual_pid,
            'utc': datetime.now(timezone.utc).isoformat(timespec='milliseconds')})

    try:
        checkpoint('prepare empty screenplay project through public services')
        fixture = json.loads(subprocess.check_output(
            [str(repo / 'target/debug/examples/authoring_capture_fixture')],
            env=environment, text=True, timeout=20))
        database = Path(fixture['project_path']).resolve()
        a_id, b_id = fixture['a']['id'], fixture['b']['id']
        if blocks(database, a_id) or blocks(database, b_id):
            raise RuntimeError('Qualification requires GUI to create the first screenplay')
        checkpoint('launch real desktop')
        with raw_log.open('w') as stream:
            process = subprocess.Popen(['./launcher.sh', '--run'], env=environment,
                stdout=stream, stderr=subprocess.STDOUT, start_new_session=True)

        def owned_window():
            if process.poll() is not None:
                raise RuntimeError('Launcher exited: ' + str(process.returncode))
            candidates = subprocess.run(['xdotool', 'search', '--onlyvisible', '--name', '^Eidetic$'],
                capture_output=True, text=True, timeout=5)
            for candidate in candidates.stdout.splitlines():
                pid = int(command('xdotool', 'getwindowpid', candidate))
                if os.getpgid(pid) == process.pid:
                    return candidate, pid
            return None

        window, pid = wait_for('owned native window', owned_window)
        NATIVE_WINDOW_PID = pid
        evidence.update(window_id=int(window), window_pid=pid)
        evidence['application_binary_sha256'] = file_hash(Path('/proc/' + str(pid) + '/exe'))
        application = wait_for('native accessibility app', lambda: next((app for app in
            pyatspi.Registry.getDesktop(0) if app.get_process_id() == pid), None))
        open_project_chooser(application, window)
        click_button(application, database.parent.name, window, prefix=True)
        checkpoint('select first scene and begin manual screenplay')
        scene = wait_for('first scene in timeline', lambda: find(application,
            lambda n: text_of(n) == fixture['a']['name'] and n.getRole() != pyatspi.ROLE_HEADING))
        click_control(scene, window)
        click_button(application, 'Write screenplay', window)
        # Select the exact labelled composer, since the selected scene also has an empty Notes field.
        textarea = wait_for('labelled new screenplay editor', lambda: find(application, lambda n:
            n.getState().contains(pyatspi.STATE_EDITABLE) and n.name == 'New screenplay text'))
        type_text(textarea, window, MANUAL_TEXT)
        choose_mode(application, 'Graph', window)
        choose_mode(application, 'Split', window)
        choose_mode(application, 'Script', window)
        textarea = wait_for('creation draft after navigation', lambda: editable(application, MANUAL_TEXT))
        evidence['creation_navigation_preserved_exact_text'] = True
        reveal_button(application, 'Save screenplay', window)
        a = wait_for('canonical first screenplay', lambda: next((row for row in blocks(database, a_id)
            if row[1] == MANUAL_TEXT), None))
        evidence['manual_creation'] = {'block_id': a[0], 'revision_event_id': a[2], 'exact_text': True}
        choose_mode(application, 'Script', window)
        checkpoint('configure explicit local provider fixture through actual AI panel')
        click_button(application, 'AI', window)
        click_button(application, 'Save & Connect', window)
        wait_for('provider fixture connected', lambda: find(application,
            lambda n: 'Connected' in text_of(n)))
        checkpoint('generate second scene through real desktop provider client')
        scene = wait_for('second timeline scene', lambda: find(application,
            lambda n: text_of(n) == fixture['b']['name'] and n.getRole() != pyatspi.ROLE_HEADING))
        click_control(scene, window)
        click_button(application, 'Generate', window)
        b = wait_for('canonical generated second scene', lambda: next((row for row in blocks(database, b_id)
            if row[1] == GENERATED_TEXT), None))
        evidence['generated_fixture_block'] = {'block_id': b[0], 'revision_event_id': b[2]}
        checkpoint('edit exact manual text and preserve draft through navigation')
        block = wait_for('manual screenplay block', lambda: screenplay_block(application, 'red umbrella'))
        click_button(block, 'Edit', window)
        textarea = wait_for('manual edit field', lambda: editable(application, MANUAL_TEXT))
        type_text(textarea, window, EDITED_TEXT)
        choose_mode(application, 'Graph', window)
        choose_mode(application, 'Split', window)
        choose_mode(application, 'Script', window)
        wait_for('existing draft after navigation', lambda: editable(application, EDITED_TEXT))
        evidence['edit_navigation_preserved_exact_text'] = True
        block = wait_for('draft screenplay block', lambda: screenplay_block(application, 'blue umbrella'))
        click_button(block, 'Compare saved text', window)
        wait_for('comparison contains saved red umbrella', lambda: find(application,
            lambda n: n.name == 'Saved text comparison' and any(MANUAL_TEXT == text_of(c) for c in walk(n))))
        if blocks(database, a_id)[0] != a:
            raise RuntimeError('Draft comparison unexpectedly wrote canonical text')
        evidence['comparison_preserved_canon_and_draft'] = True
        block = wait_for('draft block after comparison', lambda: screenplay_block(application, 'blue umbrella'))
        click_button(block, 'Save', window)
        edited = wait_for('canonical edit', lambda: next((row for row in blocks(database, a_id)
            if row[1] == EDITED_TEXT and row[2] != a[2]), None))
        evidence['manual_edit'] = {'revision_event_id': edited[2], 'exact_text': True}
        choose_mode(application, 'Script', window)
        checkpoint('retime original scene through native timeline shortcut')
        source = wait_for('original source header', lambda: reveal(application, lambda n:
            n.name == 'Screenplay source clip' and any(text_of(c) == fixture['a']['name'] for c in walk(n))))
        click_button(source, 'Go to clip', window)
        command('xdotool', 'key', '--clearmodifiers', 'alt+Right')
        retimed = wait_for('canonical screenplay placement after retime', lambda: next((row for row in
            blocks(database, a_id) if row[3:] == (a[3] + 1000, a[4] + 1000)), None))
        if retimed[1:3] != edited[1:3]:
            raise RuntimeError('Retiming changed exact text or block revision')
        evidence['retime'] = {'before_ms': list(a[3:]), 'after_ms': list(retimed[3:]),
                              'text_and_block_revision_preserved': True}
        checkpoint('observe affected generated story memory and preview explicit update')
        reveal_button(application, 'Preview update', window)
        proposal = wait_for('pending canonical proposal', lambda: next(iter(query(database,
            "SELECT id,status,proposed_text FROM propagation_proposals WHERE target_id=?",
            (b[0],))), None))
        if proposal[1:] != ('pending', PROPOSED_TEXT) or blocks(database, b_id)[0] != b:
            raise RuntimeError('Preview did not preserve canonical target text')
        if not FixtureProvider.records or FixtureProvider.records[-1]['kind'] != 'preview':
            raise RuntimeError('Preview did not reach the production HTTP provider client')
        evidence['preview'] = {'proposal_id': proposal[0], 'target_unchanged': True,
                               'exact_updated_manual_context_consumed': True}
        capture('eidetic-authoring-preview.png')
        checkpoint('explicitly accept targeted update through GUI')
        reveal_button(application, 'Accept update', window)
        accepted = wait_for('explicit target commit', lambda: next((row for row in blocks(database, b_id)
            if row[1] == PROPOSED_TEXT and row[2] != b[2]), None))
        if blocks(database, a_id)[0] != retimed:
            raise RuntimeError('Accept changed the unrelated manual input')
        evidence['acceptance'] = {'revision_event_id': accepted[2], 'input_unchanged': True}
        capture('eidetic-authoring-accepted.png')
        evidence['status'] = 'integrated_authoring_passed_with_http_fixture'
        checkpoint('completed real GUI and canonical qualification')
    except Exception as error:
        evidence['failure'] = type(error).__name__ + ': ' + str(error)[:1000]
        if application is not None:
            try:
                evidence['accessibility_snapshot'] = accessibility_snapshot(application)
                if window is not None:
                    capture('eidetic-authoring-failure.png')
            except Exception as capture_error:
                evidence['failure_capture_error'] = type(capture_error).__name__
        raise
    finally:
        faulthandler.cancel_dump_traceback_later()
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
        evidence['accessibility_retired_object_polls'] = ACCESSIBILITY_RETIREMENTS
        evidence['provider_calls'] = FixtureProvider.records
        raw = raw_log.read_text(errors='replace') if raw_log.exists() else 'No app launch log.'
        (output / 'app-sanitized.log').write_text(sanitized_log(raw, [
            (str(repo), '<workspace>'), (str(state_root), '<capture-state>'),
            (os.environ.get('RUNNER_TEMP'), '<runner-temp>')]))
        (output / 'capture-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')


if __name__ == '__main__':
    main()
