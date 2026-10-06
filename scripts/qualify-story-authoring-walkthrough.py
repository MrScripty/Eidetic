#!/usr/bin/env python3
"""Actual Last Train writing, Bible, placement and targeted screenplay review.

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

SOURCE = '42c3c0b94fdc18146b3e9ef128acb36de4399d67'
A_TEXT = 'INT. CAFE - NIGHT\n\nMara folds her blue umbrella.\n\nMARA\nKeep the last train for us.\n\n'
F_TEXT = 'EXT. PLATFORM - NIGHT\n\nEli keeps the gate open.\n\n'
C_TEXT = 'INT. TICKET OFFICE - NIGHT\n\nThe departure board reads midnight.\n\n'
C_EDITED = 'INT. TICKET OFFICE - DAWN\n\nThe departure board reads morning.\n\n'
D_TEXT = 'EXT. SIGNAL BOX - NIGHT\n\nThe green signal holds.\n\n'
E_TEXT = 'INT. LOST PROPERTY - NIGHT\n\nMara finds the missing ticket.\n\n'
GENERATED = 'Synthetic generation: Mara waits for Eli and the blue umbrella.\n\n'
MANUAL = 'EXT. STATION - NIGHT\n\nMara waits beside Eli. Preserve this station beat.\n\n'
PROPOSED = 'Synthetic preview: After finding her ticket, Mara arrives with her amber umbrella for the morning train.\n\n'
DRAFT = 'Unrelated draft: Eli pockets a brass whistle.\n\n'
BLUE = "Mara's umbrella is blue."
AMBER = "Mara's umbrella is amber."

# Reuse only predecessor QA helpers; application code is never injected/replaced.
placement_spec = importlib.util.spec_from_file_location('placement_capture', Path(__file__).with_name('qualify-timeline-placement.py'))
placement = importlib.util.module_from_spec(placement_spec)
placement_spec.loader.exec_module(placement)
ui = placement.ui


class AuthoringProvider(ui.FixtureProvider):
    records = []
    records_lock = threading.Lock()

    def contains_expected_context(self, user, kind):
        if kind == 'recap':
            return GENERATED in user
        if kind == 'generation':
            return (all(text in user for text in (A_TEXT, F_TEXT, C_TEXT, D_TEXT))
                    and E_TEXT not in user and f'profile.tagline: {BLUE}' in user)
        return (all(text in user for text in (F_TEXT, E_TEXT, C_EDITED, D_TEXT, MANUAL))
                and all(text not in user for text in (A_TEXT, C_TEXT, DRAFT))
                and f'profile.tagline: {AMBER}' in user
                and f'profile.tagline: {BLUE}' not in user)

    def response_text(self, kind):
        return PROPOSED if kind == 'preview' else 'Mara waits for Eli at the station.' if kind == 'recap' else GENERATED


def screenplay_anchor(exact_text):
    # ScriptView renders screenplay headings/action/dialogue as separate nodes;
    # a raw multiline block cannot be a substring of any single rendered child.
    # This fixture's distinctive action line identifies the native block, while
    # textarea and canonical-read checks below still require every exact byte.
    lines = [line for line in exact_text.splitlines() if line.strip()]
    if not lines:
        raise RuntimeError('QA screenplay anchor has no authored text')
    return lines[1] if len(lines) > 1 else lines[0]


def fact(database):
    return next(iter(ui.query(database, "SELECT text_value,updated_event_id FROM bible_graph_fields WHERE id='qualification.mara.tagline'")), None)


def save_fact_control(application, field):
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


def source_region(application, scene_name):
    return ui.reveal(application, lambda n: n.name == 'Screenplay source clip'
                     and any(ui.text_of(c) == scene_name for c in ui.walk(n)))


def draft_field(application):
    return ui.reveal(application, lambda n: n.getState().contains(ui.pyatspi.STATE_EDITABLE)
                     and ui.text_of(n) == DRAFT)


def require_canonical_unchanged(database, expected):
    for node_id, rows in expected.items():
        if ui.blocks(database, node_id) != rows:
            raise RuntimeError('Action changed unrelated canonical screenplay: ' + node_id)


def contained(rect, viewport):
    x, y, width, height = rect
    left, top, vw, vh = viewport
    return width > 0 and height > 0 and left <= x and top <= y and x + width <= left + vw and y + height <= top + vh


def script_viewport(application, window):
    geometry = {key: int(value) for key, value in (line.split('=', 1) for line in
        ui.command('xdotool', 'getwindowgeometry', '--shell', window).splitlines())}
    rectangles = [tuple(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN))
                  for n in ui.walk(application) if n.name == 'Resize panels' and ui.visible(n)]
    horizontal = [r for r in rectangles if 0 < r[3] <= 8 and r[2] > geometry['WIDTH'] // 2]
    top = [r for r in horizontal if r[0] > geometry['X'] and r[1] < geometry['Y'] + geometry['HEIGHT'] // 2]
    bottom = [r for r in horizontal if r[2] >= geometry['WIDTH'] - 10 and r[1] > geometry['Y'] + geometry['HEIGHT'] // 2]
    if len(top) != 1 or len(bottom) != 1:
        raise RuntimeError('Cannot identify the actual Script viewport splitters')
    x, y, width, height = top[0]
    # The Script header is outside the scrolling multicolumn body.
    return (x, y + height + 30, width, bottom[0][1] - y - height - 30)


def text_rectangles(node, exact_text):
    text = node.queryText()
    if text.getText(0, text.characterCount) != exact_text:
        raise RuntimeError('Native text changed while inspecting visible paragraph')
    return [tuple(text.getCharacterExtents(i, ui.pyatspi.XY_SCREEN))
            for i, char in enumerate(exact_text) if not char.isspace()]


def visible_paragraph(application, window, exact_text, name='Proposed text'):
    application.clear_cache()
    node = next((n for n in ui.walk(application) if n.name == name and ui.text_of(n) == exact_text), None)
    if node is None:
        return None
    viewport = script_viewport(application, window)
    rectangles = text_rectangles(node, exact_text)
    if rectangles and all(contained(r, viewport) for r in rectangles):
        return {'viewport': list(viewport), 'character_bounds': [list(r) for r in rectangles],
                'exact_text': exact_text, 'native_text_geometry_inside_script': True}
    return None


def reveal_visible_proposal(application, window, pending=True):
    """Actual accessibility scroll and horizontal wheel input; no CSS changes.

    STATE_SHOWING alone did not establish visible proposal pixels in the earlier
    short-pane capture. Require all paragraph character bounds inside the actual
    Script viewport and pending acceptance controls before capturing this view.
    """
    node = ui.wait_for('pending native proposed paragraph', lambda: ui.reveal(application,
        lambda n: n.name == 'Proposed text' and ui.text_of(n) == PROPOSED))
    node.queryComponent().scrollTo(ui.pyatspi.SCROLL_TOP_LEFT)
    for _ in range(24):
        shown = visible_paragraph(application, window, PROPOSED)
        if shown and not pending:
            return shown
        if shown:
            controls = [n for n in ui.walk(application) if n.getRole() == ui.pyatspi.ROLE_PUSH_BUTTON
                        and n.name in ('Accept update', 'Reject') and ui.visible(n)]
            if len(controls) == 2 and all(contained(tuple(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)), tuple(shown['viewport'])) for n in controls):
                shown['pending_controls'] = [{'name': n.name, 'bounds': list(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN))} for n in controls]
                return shown
        node = ui.wait_for('fresh native proposed paragraph', lambda: next((n for n in ui.walk(application)
            if n.name == 'Proposed text' and ui.text_of(n) == PROPOSED), None))
        viewport = script_viewport(application, window)
        rectangles = text_rectangles(node, PROPOSED)
        if any(r[1] < viewport[1] or r[1]+r[3] > viewport[1]+viewport[3] for r in rectangles):
            raise RuntimeError('Proposed paragraph remains vertically fragmented outside Script viewport')
        direction = '6' if min(r[0] for r in rectangles) < viewport[0] else '7'
        ui.command('xdotool', 'mousemove', str(viewport[0]+viewport[2]//2), str(viewport[1]+viewport[3]//2))
        ui.command('xdotool', 'click', '--repeat', '2', '--delay', '50', direction)
        time.sleep(0.1)
        application.clear_cache()
    raise RuntimeError('Pending proposal and controls are not visibly inside Script viewport')

def main():
    repo = Path.cwd()
    output = Path(os.environ['EIDETIC_CAPTURE_DIR'])
    output.mkdir(parents=True, exist_ok=True)
    state_root = Path(os.environ['RUNNER_TEMP']) / 'eidetic-last-train-authoring-state'
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
                'setup_route': 'public native services; four supporting blocks, empty A and ungenerated B',
                'gui_route': 'actual Tauri AT-SPI/X11 Write screenplay, Generate, manual Save, Bible Save, retained unrelated draft, placement, visible preview and explicit acceptance',
                'provider': 'synthetic localhost HTTP/SSE fixture through production client',
                'real_model_quality_qualified': False, 'checkpoints': []}
    process = application = window = None
    provider = ThreadingHTTPServer(('127.0.0.1', 18080), AuthoringProvider)
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
        fixture = json.loads(subprocess.check_output([str(repo / 'target/debug/examples/story_authoring_walkthrough_fixture')],
                                                    env=environment, text=True, timeout=60))
        database = Path(fixture['project_path']).resolve()
        a_id, b_id, c_id, f_id, e_id = (fixture[key]['id'] for key in ('a','b','c','f','e'))
        if ui.blocks(database, a_id) or ui.blocks(database, b_id):
            raise RuntimeError('Native writing/generation targets are not empty')
        original = {scene['id']: ui.blocks(database, scene['id']) for scene in fixture['scenes']}
        original_fact = fact(database)
        evidence['project'] = {'name': database.parent.name, 'initial_scene_order': fixture['before_order'],
                               'setup_route': fixture['setup_route'], 'A_and_B_initially_empty': True}
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
        clip = ui.wait_for('A timeline clip for actual new screenplay', lambda: ui.native_timeline_clip(application, fixture['a']['name'], window))
        ui.click_control(clip[0], window, clip[1])
        ui.reveal_button(application, 'Write screenplay', window)
        field = ui.wait_for('actual labelled writing area', lambda: ui.reveal(application,
            lambda n: n.getState().contains(ui.pyatspi.STATE_EDITABLE) and n.name == 'New screenplay text'))
        ui.type_text(field, window, A_TEXT)
        evidence['writing_area'] = {'exact_draft': A_TEXT, 'selected_scene': a_id, 'not_saved_before_capture': not ui.blocks(database,a_id)}
        checkpoint('exact original screenplay typed in actual Write screenplay area')
        capture('eidetic-story-authoring-writing.png')
        ui.reveal_button(application, 'Save screenplay', window)
        a = ui.wait_for('canonical new A screenplay', lambda: next((row for row in ui.blocks(database,a_id) if row[1]==A_TEXT),None))
        original[a_id] = ui.blocks(database,a_id)
        evidence['manual_creation'] = {'block_id': a[0], 'revision_event_id': a[2], 'exact_text': A_TEXT}
        # Explicit fixture connection and Generate are real native controls.
        ui.click_button(application, 'AI', window)
        ui.click_button(application, 'Save & Connect', window)
        ui.wait_for('connected synthetic HTTP fixture', lambda: ui.find(application,lambda n:'Connected' in ui.text_of(n)))
        clip = ui.wait_for('B timeline clip for generation', lambda: ui.native_timeline_clip(application,fixture['b']['name'],window))
        ui.click_control(clip[0],window,clip[1])
        ui.reveal_button(application,'Generate',window)
        generated = ui.wait_for('native generation canonical B',lambda:next((row for row in ui.blocks(database,b_id) if row[1]==GENERATED),None))
        evidence['generation'] = {'block_id':generated[0], 'revision_event_id':generated[2], 'real_model':False,
                                  'actual_GUI_Generate':True, 'exact_original_A_consumed':True}
        original[b_id] = ui.blocks(database,b_id)
        block = ui.wait_for('generated B native block',lambda:ui.screenplay_block(application,screenplay_anchor(GENERATED)))
        ui.reveal_button(block,'Edit',window)
        field = ui.wait_for('B edit field',lambda:ui.editable(application,GENERATED))
        ui.type_text(field,window,MANUAL)
        block = ui.wait_for('B exact manual draft',lambda:ui.screenplay_block(application,screenplay_anchor(MANUAL)))
        ui.reveal_button(block,'Save',window)
        b = ui.wait_for('saved exact manual B',lambda:next((row for row in ui.blocks(database,b_id) if row[1]==MANUAL),None))
        original[b_id] = ui.blocks(database,b_id)
        evidence['manual_B_save']={'exact_text':MANUAL,'revision_event_id':b[2]}
        checkpoint('native generation and exact manual B saved; begin upstream C edit')
        block = ui.wait_for('C manual source block',lambda:ui.screenplay_block(application,screenplay_anchor(C_TEXT)))
        ui.reveal_button(block,'Edit',window)
        field = ui.wait_for('C exact existing text',lambda:ui.editable(application,C_TEXT))
        ui.type_text(field,window,C_EDITED)
        block = ui.wait_for('C exact edited draft',lambda:ui.screenplay_block(application,screenplay_anchor(C_EDITED)))
        ui.reveal_button(block,'Save',window)
        c = ui.wait_for('saved midnight-to-morning C',lambda:next((row for row in ui.blocks(database,c_id) if row[1]==C_EDITED),None))
        original[c_id] = ui.blocks(database,c_id)
        evidence['manual_edit'] = {'target_B_exact_text':MANUAL,'target_B_revision':b[2],
                                   'upstream_C_before':C_TEXT,'upstream_C_after':C_EDITED,'upstream_C_revision':c[2]}
        disclosure=ui.wait_for('manual-change What changed disclosure',lambda:ui.reveal(application,lambda n:n.name=='What changed'))
        ui.click_control(disclosure,window)
        ui.wait_for('manual source edit causes downstream review',lambda:ui.reveal(application,lambda n:'Source screenplay text changed.' in ui.text_of(n)))
        checkpoint('native exact manual edits save B and change C from midnight to morning')
        capture('eidetic-story-authoring-manual-edit.png')
        block = ui.wait_for('unrelated F saved block',lambda:ui.screenplay_block(application,screenplay_anchor(F_TEXT)))
        ui.reveal_button(block,'Edit',window)
        field = ui.wait_for('F exact edit field',lambda:ui.editable(application,F_TEXT))
        ui.type_text(field,window,DRAFT)
        ui.wait_for('unrelated draft stays open',lambda:draft_field(application))
        require_canonical_unchanged(database,original)
        evidence['unrelated_draft'] = {'source_scene_id':f_id,'saved_text':F_TEXT,'exact_draft':DRAFT,
                                       'saved_revision':original[f_id][0][2],'not_saved':True}
        e_clip = ui.wait_for('E native timeline clip',lambda:ui.native_timeline_clip(application,fixture['e']['name'],window))
        ui.click_control(e_clip[0],window,e_clip[1])
        disclosure = ui.wait_for('exact placement disclosure',lambda:ui.reveal(application,lambda n:n.name=='Placement: 540–570 seconds'))
        ui.click_control(disclosure,window)
        for name,value in (('Placement start seconds','180'),('Placement end seconds','210')):
            field=ui.wait_for(name,lambda name=name:ui.reveal(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and n.name==name))
            ui.type_text(field,window,value)
        ui.reveal_button(application,'Apply placement',window)
        placed=ui.wait_for('E exact persisted placement',lambda:next((row for row in ui.query(database,'SELECT start_ms,end_ms FROM nodes WHERE id=?',(e_id,)) if row==(180000,210000)),None))
        if placement.scene_order(database) != ['SCENE '+letter for letter in fixture['after_order']]:
            raise RuntimeError('Native E placement did not produce expected order')
        placement.require_placement_preserves_screenplay(original[e_id],ui.blocks(database,e_id),180000,210000)
        original[e_id]=ui.blocks(database,e_id)
        require_canonical_unchanged(database,original)
        ui.wait_for('F draft retained after placement',lambda:draft_field(application))
        written=ui.query(database,"SELECT c.id,c.payload_json,e.id FROM commands c JOIN change_events e ON e.command_id=c.id WHERE c.payload_type='timeline.node_range' ORDER BY e.rowid DESC LIMIT 1")
        payload=json.loads(written[0][1])
        if (payload.get('node_id'),payload.get('start_ms'),payload.get('end_ms')) != (e_id,180000,210000) or not payload.get('expected') or (payload['expected']['start_ms'],payload['expected']['end_ms']) != (540000,570000):
            raise RuntimeError('Native placement receipt did not bind original range')
        evidence['placement']={'before_ms':[540000,570000],'after_ms':list(placed),'after_order':fixture['after_order'],
                               'command_id':written[0][0],'change_event_id':written[0][2],'expected_read':payload['expected'],
                               'all_saved_screenplay_and_F_draft_preserved':True}
        checkpoint('native E placement crosses neighbors with F draft open')
        capture('eidetic-story-authoring-placement.png')
        ui.click_button(application,'Bible',window)
        mara=ui.wait_for('Mara Bible control',lambda:ui.reveal(application,lambda n:n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and ui.button_label_matches(n.name,'Mara',prefix=True)))
        ui.click_control(mara,window)
        field=ui.wait_for('blue Bible fact',lambda:ui.reveal(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n)==BLUE))
        ui.type_text(field,window,AMBER)
        save=ui.wait_for('associated Bible field Save',lambda:save_fact_control(application,field))
        ui.click_control(save,window)
        edited_fact=ui.wait_for('canonical amber Bible fact',lambda:fact(database) if fact(database)[0]==AMBER and fact(database)[1]!=original_fact[1] else None)
        require_canonical_unchanged(database,original)
        ui.wait_for('F draft retained after exact Bible Save',lambda:draft_field(application))
        evidence['fact_edit']={'before':BLUE,'after':AMBER,'revision_event_id':edited_fact[1],
                               'saved_screenplay_and_unrelated_draft_preserved':True}
        disclosure=ui.wait_for('downstream What changed disclosure',lambda:ui.reveal(application,lambda n:n.name=='What changed'))
        if not ui.find(application,lambda n:'Source screenplay text changed.' in ui.text_of(n)):
            ui.click_control(disclosure,window)
        for explanation in ('Source screenplay text changed.','Bible fact profile.tagline changed.','Entered: SCENE E. Left: SCENE A.'):
            ui.wait_for(explanation,lambda explanation=explanation:ui.reveal(application,lambda n:explanation in ui.text_of(n)))
        checkpoint('Bible, manual-source and placement changes visibly identify affected B')
        capture('eidetic-story-authoring-review.png')
        ui.reveal_button(application,'Preview update',window)
        proposal=ui.wait_for('pending targeted B proposal',lambda:next(iter(ui.query(database,'SELECT id,status,proposed_text FROM propagation_proposals WHERE target_id=?',(b[0],))),None))
        if proposal[1:] != ('pending',PROPOSED):
            raise RuntimeError('Targeted proposal text/status differs')
        require_canonical_unchanged(database,original)
        ui.require_accepted_preview(AuthoringProvider.records)
        ui.wait_for('F draft stays open during pending preview',lambda:draft_field(application))
        visibility=reveal_visible_proposal(application,window)
        evidence['preview']={'proposal_id':proposal[0],'target_B_unchanged':True,'unrelated_F_draft_open':True,
                             'exact_updated_C_and_Bible_and_FECD_window_consumed':True,
                             'unsaved_F_draft_excluded_from_context':True,'visible_proposal':visibility}
        checkpoint('complete proposal paragraph and explicit pending controls visible before acceptance')
        capture('eidetic-story-authoring-preview.png')
        ui.reveal_button(application,'Accept update',window)
        accepted=ui.wait_for('explicitly accepted B',lambda:next((row for row in ui.blocks(database,b_id) if row[1]==PROPOSED and row[2]!=b[2]),None))
        if ui.blocks(database,b_id) != [accepted] or accepted[0]!=b[0] or accepted[3:]!=b[3:]:
            raise RuntimeError('Acceptance changed B membership or placement')
        original[b_id]=[accepted]
        require_canonical_unchanged(database,original)
        if fact(database)!=edited_fact:
            raise RuntimeError('Acceptance changed Bible fact')
        bindings=ui.query(database,"SELECT r.target_revision_event_id FROM semantic_dependencies d JOIN semantic_dependency_revisions r ON r.dependency_id=d.id WHERE d.target_field_id='qualification.mara.tagline' AND r.source_revision_event_id=?",(accepted[2],))
        if bindings!=[(edited_fact[1],)]:
            raise RuntimeError('Accepted B did not refresh actual Bible revision')
        def cleared():
            application.clear_cache()
            return not any(n.name=='Screenplay needs review' for n in ui.walk(application))
        ui.wait_for('old downstream review cleared',cleared)
        ui.wait_for('F exact draft remains editable after B acceptance',lambda:draft_field(application))
        ui.wait_for('canonical accepted paragraph visible',lambda:ui.screenplay_block(application,screenplay_anchor(PROPOSED)))
        accepted_visibility=reveal_visible_proposal(application,window,pending=False)
        evidence['acceptance']={'revision_event_id':accepted[2],'only_B_changed':True,'Bible_revision_refreshed':edited_fact[1],
                                'accepted_proposal_visible':accepted_visibility,'unrelated_F_draft_still_open':True,'review_cleared':True}
        checkpoint('explicit native acceptance replaces only B with unrelated F draft still open')
        capture('eidetic-story-authoring-accepted.png')
        ui.wait_for('retained F draft final native surface',lambda:draft_field(application))
        require_canonical_unchanged(database,original)
        capture('eidetic-story-authoring-retained-draft.png')
        calls=AuthoringProvider.records
        if len(calls)!=3 or any(r.get('accepted') is not True or r['real_model'] is not False for r in calls):
            raise RuntimeError('Synthetic provider call sequence differs')
        evidence['status']='native_last_train_authoring_passed_with_labelled_synthetic_http_fixture'
    except Exception as error:
        evidence['failure'] = type(error).__name__ + ': ' + str(error)[:1000]
        if application is not None:
            try:
                evidence['accessibility_snapshot'] = ui.accessibility_snapshot(application)
                capture('eidetic-story-authoring-failure.png')
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
        evidence['provider_calls'] = AuthoringProvider.records
        evidence['accessibility_retired_object_polls'] = ui.ACCESSIBILITY_RETIREMENTS
        evidence['typed_text_observations'] = ui.TYPED_TEXT_OBSERVATIONS
        evidence['timeline_locator_observations'] = ui.TIMELINE_LOCATOR_OBSERVATIONS
        raw = raw_log.read_text(errors='replace') if raw_log.exists() else 'No app launch log.'
        (output / 'app-sanitized.log').write_text(ui.sanitized_log(raw, [(str(repo), '<workspace>'), (str(state_root), '<capture-state>'), (os.environ.get('RUNNER_TEMP'), '<runner-temp>')]))
        (output / 'capture-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')


if __name__ == '__main__':
    main()
