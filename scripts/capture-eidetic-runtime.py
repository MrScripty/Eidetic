#!/usr/bin/env python3
"""Drive the actual Tauri window through desktop accessibility and keyboard input.

Fail closed if the app, accessibility tree, canonical edit, or capture is absent.
Never inject JavaScript, replace IPC, disable a sandbox, or capture a browser preview.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import sqlite3
import subprocess
import time

import pyatspi
from gi.repository import Atspi, GLib


SOURCE = "550650cdc794b15352013d9914eec337e5ac022d"
EDITED_TEXT = (
    "INT. NIGHT SHIFT CAFE - NIGHT\n\n"
    "Mara pockets the timetable and slides Eli his coffee.\n\n"
    "ELI\nWe still have time for the last train."
)
DEADLINE = time.monotonic() + 110


def command(*args):
    return subprocess.check_output(args, text=True, timeout=15).strip()


def dispatch_accessibility_events():
    context = GLib.MainContext.default()
    for _ in range(50):
        if not context.iteration(False):
            break


def wait_for(description, check):
    while time.monotonic() < DEADLINE:
        dispatch_accessibility_events()
        found = check()
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


def click_button(root, label, prefix=False):
    node = wait_for(
        f"visible button {label}",
        lambda: find(root, lambda node: node.getRole() == pyatspi.ROLE_PUSH_BUTTON
                     and button_label_matches(node.name, label, prefix)),
    )
    action = node.queryAction()
    for index in range(action.nActions):
        if action.getName(index) in ("click", "press", "activate"):
            if action.doAction(index):
                return
    raise RuntimeError(f"No supported accessibility action for {label}")


def text_of(node):
    try:
        text = node.queryText()
        return text.getText(0, min(text.characterCount, 4096))
    except NotImplementedError:
        return node.name or ""


def canonical_block(path):
    with sqlite3.connect(path.as_uri() + "?mode=ro", uri=True, timeout=2) as conn:
        return conn.execute(
            "SELECT text, updated_event_id FROM script_blocks "
            "WHERE id = 'capture.screenplay.block' AND deleted_event_id IS NULL"
        ).fetchone()


def committed_edit(path, before):
    row = canonical_block(path)
    return row if row and row[0] == EDITED_TEXT and row[1] != before[1] else None


def file_hash(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


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


def main():
    repo = Path.cwd()
    output = Path(os.environ["EIDETIC_CAPTURE_DIR"])
    output.mkdir(parents=True, exist_ok=True)
    state_root = Path(os.environ["RUNNER_TEMP"]) / "eidetic-capture-state"
    raw_log = state_root / "app-private.log"
    environment = os.environ.copy()
    environment["EIDETIC_LAUNCHER_STATE_ROOT"] = str(state_root)
    for suffix in ("CACHE", "CONFIG", "DATA", "STATE"):
        directory = state_root / "dev" / f"xdg-{suffix.lower()}"
        directory.mkdir(parents=True, exist_ok=True)
        environment[f"XDG_{suffix}_HOME"] = str(directory)
    environment.update({"NO_AT_BRIDGE": "0", "GTK_MODULES": "atk-bridge"})
    Atspi.set_timeout(1500, 1500)
    evidence = {
        "status": "failed",
        "application_source_sha": SOURCE,
        "qualification_sha": command("git", "rev-parse", "HEAD"),
        "application_binary_sha256": file_hash(repo / "target/debug/eidetic-desktop"),
        "utc_capture_attempt": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "interaction_route": "AT-SPI buttons and X11 keyboard on real Tauri app",
        "content_origin": "local sample created through public backend services; no AI call",
    }
    process = None
    application = None
    try:
        fixture = json.loads(subprocess.check_output(
            [str(repo / "target/debug/examples/runtime_capture_fixture")],
            env=environment, text=True, timeout=20,
        ))
        database = Path(fixture["project_path"]).resolve()
        before = canonical_block(database)
        if not before or before[0] != fixture["text"]:
            raise RuntimeError("Fixture canonical text missing before launch")
        with raw_log.open("w") as stream:
            process = subprocess.Popen(
                ["./launcher.sh", "--run"], env=environment,
                stdout=stream, stderr=subprocess.STDOUT, start_new_session=True,
            )

        def app_window():
            if process.poll() is not None:
                raise RuntimeError(f"Launcher exited before capture: {process.returncode}")
            windows = subprocess.run(
                ["xdotool", "search", "--onlyvisible", "--name", "^Eidetic$"],
                capture_output=True, text=True, timeout=5,
            )
            for window in windows.stdout.splitlines():
                pid = int(command("xdotool", "getwindowpid", window))
                if os.getpgid(pid) == process.pid:
                    return window, pid
            return None

        window, pid = wait_for("native Eidetic window owned by launcher", app_window)
        evidence["window_pid"] = pid
        evidence["application_binary_sha256"] = file_hash(Path(f"/proc/{pid}/exe"))
        application = wait_for(
            "accessibility application for native window PID",
            lambda: next((app for app in pyatspi.Registry.getDesktop(0)
                          if app.get_process_id() == pid), None),
        )
        # Poll live data after Svelte replaces controls; do not retain old child lists.
        application.set_cache_mask(Atspi.Cache.NONE)
        evidence["accessibility_mode"] = "uncached queries with bounded GLib event dispatch"
        click_button(application, "Open Project")
        click_button(application, fixture["project_name"], prefix=True)
        wait_for("project timeline", lambda: find(
            application, lambda node: fixture["scene_name"] in (node.name or "")))
        block = wait_for("screenplay block", lambda: find(
            application, lambda node: node.name == "Screenplay block"))
        click_button(block, "Edit")
        textarea = wait_for("screenplay text editor", lambda: find(
            application, lambda node: node.getState().contains(pyatspi.STATE_EDITABLE)
            and text_of(node) == fixture["text"]))
        command("xdotool", "windowfocus", "--sync", window)
        component = textarea.queryComponent()
        if not component.grabFocus():
            raise RuntimeError("Screenplay editor refused keyboard focus")
        command("xdotool", "key", "--clearmodifiers", "ctrl+a")
        subprocess.run(["xdotool", "type", "--clearmodifiers", "--delay", "1", EDITED_TEXT],
                       check=True, timeout=15)
        wait_for("typed screenplay draft", lambda: text_of(textarea) == EDITED_TEXT)
        click_button(block, "Save")
        wait_for("canonical manual edit committed", lambda: committed_edit(database, before))
        wait_for("edited text displayed", lambda: find(
            application, lambda node: "Mara pockets the timetable" in text_of(node)))
        if find(application, lambda node: node.getRole() == pyatspi.ROLE_ALERT):
            raise RuntimeError("App displays an error alert after the edit")
        if process.poll() is not None:
            raise RuntimeError("Native app exited before its window could be captured")
        screenshot = output / "eidetic-native.png"
        subprocess.run(["import", "-window", window, str(screenshot)], check=True, timeout=10)
        if not 10000 < screenshot.stat().st_size < 5 * 1024**2:
            screenshot.unlink()
            raise RuntimeError("Screenshot size outside expected artifact bounds")
        evidence.update({
            "status": "captured",
            "window_pid": pid,
            "manual_edit_committed": True,
            "initial_revision_event_id": before[1],
            "edited_revision_event_id": canonical_block(database)[1],
            "screenshot_sha256": file_hash(screenshot),
        })
    except Exception as error:
        evidence["failure"] = type(error).__name__ + ": " + str(error)[:1000]
        if application is not None:
            try:
                evidence["accessibility_snapshot"] = accessibility_snapshot(application)
            except Exception as snapshot_error:
                evidence["accessibility_snapshot_error"] = type(snapshot_error).__name__
        raise
    finally:
        if process is not None:
            try:
                os.killpg(process.pid, signal.SIGTERM)
                process.wait(timeout=5)
            except (ProcessLookupError, subprocess.TimeoutExpired):
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
        raw = raw_log.read_text(errors="replace") if raw_log.exists() else "No app launch log."
        (output / "app-sanitized.log").write_text(sanitized_log(raw, [
            (str(repo), "<workspace>"), (str(state_root), "<capture-state>"),
            (os.environ.get("RUNNER_TEMP"), "<runner-temp>"),
        ]))
        (output / "capture-evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")


if __name__ == "__main__":
    main()
