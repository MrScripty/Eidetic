#!/usr/bin/env python3
"""Drive the actual Tauri window through accessibility geometry and X11 input.

Fail closed if the app, accessibility tree, canonical edit, or capture is absent.
Never inject JavaScript, replace IPC, disable a sandbox, or capture a browser preview.
"""
import faulthandler
from datetime import datetime, timezone
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
from gi.repository import Atspi


SOURCE = "550650cdc794b15352013d9914eec337e5ac022d"
EDITED_TEXT = (
    "INT. NIGHT SHIFT CAFE - NIGHT\n\n"
    "Mara pockets the timetable and slides Eli his coffee.\n\n"
    "ELI\nWe still have time for the last train."
)
DEADLINE = time.monotonic() + 110


def command(*args):
    return subprocess.check_output(args, text=True, timeout=15).strip()


def wait_for(description, check):
    while time.monotonic() < DEADLINE:
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
    command("xdotool", "mousemove", "--sync", str(point[0]), str(point[1]))
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
    mode = os.environ.get("EIDETIC_CAPTURE_MODE", "manual-edit")
    if mode not in {"manual-edit", "render-diagnostic"}:
        raise RuntimeError("Unsupported native capture mode")
    started = time.monotonic()
    evidence = {
        "status": "failed",
        "application_source_sha": SOURCE,
        "qualification_sha": command("git", "rev-parse", "HEAD"),
        "application_binary_sha256": file_hash(repo / "target/debug/eidetic-desktop"),
        "utc_capture_attempt": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "interaction_route": "AT-SPI control geometry and X11 mouse/keyboard on real Tauri app",
        "content_origin": "local sample created through public backend services; no AI call",
        "capture_mode": mode,
    }
    process = None
    application = None
    faulthandler.dump_traceback_later(90)

    def checkpoint(stage):
        evidence["stage"] = stage
        evidence.setdefault("checkpoints", []).append({
            "stage": stage,
            "utc": datetime.now(timezone.utc).isoformat(timespec="milliseconds"),
            "elapsed_seconds": round(time.monotonic() - started, 3),
        })
        (output / "capture-evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")
        print(f"Capture checkpoint: {stage}", flush=True)

    def observe_window(window, pid, path, screen_read=False):
        actual_pid = int(command("xdotool", "getwindowpid", window))
        if (process is None or process.poll() is not None or actual_pid != pid
                or os.getpgid(actual_pid) != process.pid):
            raise RuntimeError("Native capture window ownership changed")
        record = {
            "file": path.name,
            "window_id": int(window),
            "window_pid": actual_pid,
            "title": command("xdotool", "getwindowname", window),
            "geometry": dict(line.split("=", 1) for line in command(
                "xdotool", "getwindowgeometry", "--shell", window).splitlines()),
            "read_route": "screen" if screen_read else "window",
            "utc_started": datetime.now(timezone.utc).isoformat(timespec="milliseconds"),
        }
        digest = capture_native_window(window, path, screen_read)
        record.update(sha256=digest, utc_finished=datetime.now(timezone.utc).isoformat(
            timespec="milliseconds"))
        evidence.setdefault("captures", []).append(record)
        return digest

    try:
        checkpoint("create real backend sample")
        fixture = json.loads(subprocess.check_output(
            [str(repo / "target/debug/examples/runtime_capture_fixture")],
            env=environment, text=True, timeout=20,
        ))
        database = Path(fixture["project_path"]).resolve()
        before = canonical_block(database)
        if not before or before[0] != fixture["text"]:
            raise RuntimeError("Fixture canonical text missing before launch")
        checkpoint("launch native application")
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
        evidence["window_id"] = int(window)
        evidence["window_pid"] = pid
        evidence["application_binary_sha256"] = file_hash(Path(f"/proc/{pid}/exe"))
        checkpoint("locate native accessibility application")
        application = wait_for(
            "accessibility application for native window PID",
            lambda: next((app for app in pyatspi.Registry.getDesktop(0)
                          if app.get_process_id() == pid), None),
        )
        evidence["accessibility_mode"] = "explicit cache refresh before tree searches"
        checkpoint("capture native home screen before editing")
        wait_for("visible native home screen", lambda: find(
            application, lambda node: node.getRole() == pyatspi.ROLE_PUSH_BUTTON
            and button_label_matches(node.name, "Open Project")))
        time.sleep(2)
        evidence["unedited_screenshot_sha256"] = observe_window(
            window, pid, output / "eidetic-native-unedited.png")
        evidence["unedited_screenshot_stage"] = "native home screen before opening sample or editing"
        checkpoint("open project chooser")
        open_project_chooser(application, window)
        checkpoint("open sample project")
        # list_projects exposes the saved directory name, not the project title.
        # Derive that exact chooser label from the service-returned database path.
        chooser_label = database.parent.name
        evidence["sample_chooser_label"] = chooser_label
        click_button(application, chooser_label, window, prefix=True)
        checkpoint("capture loaded sample before editing")
        block = wait_for("screenplay block", lambda: find(
            application, lambda node: node.name == "Screenplay block"))
        wait_for("imported sample text displayed", lambda: find(
            block, lambda node: "Mara sets two cups beside a folded timetable." in text_of(node)))
        time.sleep(1)
        evidence["unedited_screenshot_sha256"] = observe_window(
            window, pid, output / "eidetic-native-unedited.png")
        evidence["unedited_screenshot_stage"] = "native sample project before screenplay edit"
        checkpoint("wait for project timeline")
        # Plain clip-name spans expose their text separately from accessible names.
        wait_for("project timeline scene text", lambda: find(
            application, lambda node: fixture["scene_name"] in text_of(node)))
        if mode == "render-diagnostic":
            checkpoint("observe sample rendering without screenplay editing")
            # Compare the same owned window's drawable and visible screen after
            # an explicit dwell. Accessibility readiness alone is not visual proof.
            time.sleep(10)
            evidence["render_dwell_seconds"] = 10
            evidence["diagnostic_accessibility_snapshot"] = accessibility_snapshot(application)
            evidence["unedited_screenshot_sha256"] = observe_window(
                window, pid, output / "eidetic-native-unedited.png")
            evidence["screen_read_screenshot_sha256"] = observe_window(
                window, pid, output / "eidetic-native.png", screen_read=True)
            evidence.update({
                "status": "render_diagnostic_captured",
                "unedited_screenshot_stage": "sample render diagnostic after ten-second dwell",
                "manual_edit_committed": False,
                "visual_status": "requires inspection of both PNGs; no visible-editor claim",
            })
            return
        checkpoint("begin manual screenplay edit")
        click_button(block, "Edit", window)
        textarea = wait_for("screenplay text editor", lambda: find(
            application, lambda node: node.getState().contains(pyatspi.STATE_EDITABLE)
            and text_of(node) == fixture["text"]))
        click_control(textarea, window)
        checkpoint("type manual screenplay edit")
        command("xdotool", "key", "--clearmodifiers", "ctrl+a")
        subprocess.run(["xdotool", "type", "--clearmodifiers", "--delay", "1", EDITED_TEXT],
                       check=True, timeout=15)
        wait_for("typed screenplay draft", lambda: text_of(textarea) == EDITED_TEXT)
        checkpoint("save manual screenplay edit")
        click_button(block, "Save", window)
        checkpoint("verify canonical manual edit")
        wait_for("canonical manual edit committed", lambda: committed_edit(database, before))
        wait_for("edited text displayed", lambda: find(
            application, lambda node: "Mara pockets the timetable" in text_of(node)))
        if find(application, lambda node: node.getRole() == pyatspi.ROLE_ALERT):
            raise RuntimeError("App displays an error alert after the edit")
        if process.poll() is not None:
            raise RuntimeError("Native app exited before its window could be captured")
        screenshot = output / "eidetic-native.png"
        checkpoint("capture actual native window")
        screenshot_hash = observe_window(window, pid, screenshot)
        evidence.update({
            "status": "captured",
            "window_pid": pid,
            "manual_edit_committed": True,
            "initial_revision_event_id": before[1],
            "edited_revision_event_id": canonical_block(database)[1],
            "screenshot_sha256": screenshot_hash,
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
        faulthandler.cancel_dump_traceback_later()
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
