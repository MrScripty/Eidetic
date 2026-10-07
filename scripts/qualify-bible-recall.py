#!/usr/bin/env python3
"""Actual native one-hop recall and Bible edit with downstream review.

Public services prepare the project. All desktop interactions use normal
AT-SPI scrolling/geometry and X11 input; database access is read-only.
Synthetic HTTP/SSE responses are labelled and do not qualify model quality.
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

SOURCE = os.environ.get('EIDETIC_CAPTURE_SOURCE', '14f757561be5cc833b408d4355386afc41a92255')
A_TEXT = 'INT. CAFE - NIGHT\n\nMara folds her blue umbrella.\n\n'
F_TEXT = 'EXT. PLATFORM - NIGHT\n\nEli keeps the gate open.\n\n'
GENERATED = 'EXT. STATION - NIGHT\n\nSynthetic generation: Mara waits for Eli with her blue umbrella.\n\n'
MANUAL = 'EXT. STATION - NIGHT\n\nMara keeps the witness hidden. Preserve this station beat.\n\n'
PROPOSED = 'Synthetic preview: Mara waits for Eli with her amber umbrella. Preserve this station beat.\n\n'
DRAFT = 'Unrelated draft: Eli pockets a brass whistle.\n\n'
BLUE = "Mara's umbrella is blue."
AMBER = "Mara's umbrella is amber."


class RecallProvider(ui.FixtureProvider):
    records = []
    records_lock = threading.Lock()

    def contains_expected_context(self, user, kind):
        if kind == 'recap':
            return GENERATED in user
        return (A_TEXT in user and F_TEXT in user and DRAFT not in user
                and f'profile.tagline: {AMBER if kind == "preview" else BLUE}' in user
                and 'environment.weather: Rain' not in user and 'Flooded road' not in user
                and (kind != 'preview' or (MANUAL in user and f'profile.tagline: {BLUE}' not in user)))

    def response_text(self, kind):
        return PROPOSED if kind == 'preview' else 'Mara waits for Eli.' if kind == 'recap' else GENERATED



# Native geometry helpers reused unchanged from qualification e78a38c.
def screenplay_anchor(exact_text):
    # ScriptView renders screenplay headings/action/dialogue as separate nodes;
    # a raw multiline block cannot be a substring of any single rendered child.
    # This fixture's distinctive action line identifies the native block, while
    # textarea and canonical-read checks below still require every exact byte.
    lines = [line for line in exact_text.splitlines() if line.strip()]
    if not lines:
        raise RuntimeError('QA screenplay anchor has no authored text')
    return lines[1] if len(lines) > 1 else lines[0]

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

def visible_review_label(application, window, exact_label):
    # reveal() chooses the first matching object, including hidden select options.
    # Read a visible, bounded native control/text object instead of scrolling a
    # closed option and assuming that None means the rendered review is absent.
    viewport=script_viewport(application,window)
    node=ui.find(application,lambda n: exact_label in ui.text_of(n)
        and contained(tuple(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)),viewport))
    if node is None:
        return None
    return {'expected_label':exact_label,'native_name':node.name,
            'native_text':ui.text_of(node),'role':node.getRoleName(),
            'bounds':list(node.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)),
            'viewport':list(viewport)}

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

ui.walk = walk_live

def canonical_state(database):
    tables = ['commands', 'change_events', 'object_revisions', 'object_revision_fields',
              'semantic_dependencies', 'semantic_dependency_revisions', 'propagation_proposals']
    result = {}
    for table in tables:
        if ui.query(database, "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?", (table,)):
            result[table] = ui.query(database, f'SELECT * FROM {table} ORDER BY rowid')
    return result


def require_preserved(database, original, before_history=None):
    current = {node_id: ui.blocks(database, node_id) for node_id in original}
    if current != original:
        raise RuntimeError('Recall or fact edit changed saved screenplay text/revisions/placement')
    if before_history is not None and canonical_state(database) != before_history:
        raise RuntimeError('Explicit recall changed canonical history, proposals or dependencies')


def fact(database):
    rows = ui.query(database, "SELECT text_value,updated_event_id FROM bible_graph_fields WHERE id='qualification.mara.tagline'")
    return rows[0] if len(rows) == 1 else None


def visible_text(application, label):
    return ui.find(application, lambda n: label in ui.text_of(n) and n.getRole() not in (ui.pyatspi.ROLE_COMBO_BOX,))


def recall_disclosure(application):
    # The labelled section and its summary share text. Only the actual
    # expandable control is a valid X11 click target, including after opening.
    return ui.reveal(application, lambda n: n.name == 'Related story facts'
        and n.getRole() in (ui.pyatspi.ROLE_PUSH_BUTTON, ui.pyatspi.ROLE_TOGGLE_BUTTON))


def recall_time(application, window, text):
    field = ui.wait_for('explicit fictional-time input', lambda: ui.reveal(application,
        lambda n: n.getState().contains(ui.pyatspi.STATE_EDITABLE) and n.name == 'Recall story time (ms)'))
    ui.type_text(field, window, text)
    ui.reveal_button(application, 'Recall related story facts', window)


def main():
    repo = Path.cwd()
    output = Path(os.environ['EIDETIC_CAPTURE_DIR'])
    output.mkdir(parents=True, exist_ok=True)
    state_root = Path(os.environ['RUNNER_TEMP']) / 'eidetic-bible-recall-state'
    state_root.mkdir(parents=True, exist_ok=True)
    raw_log = state_root / 'app-private.log'
    environment = os.environ.copy()
    environment['EIDETIC_LAUNCHER_STATE_ROOT'] = str(state_root)
    for suffix in ('CACHE','CONFIG','DATA','STATE'):
        directory = state_root / 'dev' / ('xdg-' + suffix.lower())
        directory.mkdir(parents=True, exist_ok=True)
        environment['XDG_' + suffix + '_HOME'] = str(directory)
    environment.update(NO_AT_BRIDGE='0', GTK_MODULES='atk-bridge')
    ui.Atspi.set_timeout(1500, 1500)
    started = time.monotonic()
    evidence = {'status':'failed','application_source_sha':SOURCE,
        'qualification_sha':ui.command('git','rev-parse','HEAD'),
        'application_binary_sha256':ui.file_hash(repo/'target/debug/eidetic-desktop'),
        'capture_label':'actual native prototype views',
        'setup_route':'public services only; no model execution in setup',
        'gui_route':'AT-SPI/X11; graph closed; Bible, timeline and screenplay visible',
        'provider':'labelled synthetic localhost HTTP/SSE through production client',
        'recall_model_execution':'none','real_model_quality_qualified':False,'checkpoints':[]}
    process = application = window = None
    provider = ThreadingHTTPServer(('127.0.0.1',18080), RecallProvider)
    threading.Thread(target=provider.serve_forever,daemon=True).start()

    def checkpoint(stage):
        evidence['stage']=stage
        evidence['checkpoints'].append({'stage':stage,'elapsed_seconds':round(time.monotonic()-started,3)})
        (output/'capture-evidence.json').write_text(json.dumps(evidence,indent=2)+'\n')
        print('Bible recall checkpoint: '+stage,flush=True)

    def capture(name):
        pid=int(ui.command('xdotool','getwindowpid',window))
        if process.poll() is not None or os.getpgid(pid)!=process.pid:
            raise RuntimeError('Native window ownership changed')
        digest=ui.capture_native_window(window,output/name,screen_read=True)
        evidence.setdefault('captures',[]).append({'file':name,'sha256':digest,'window_id':int(window),'pid':pid})

    try:
        checkpoint('public services prepare three scenes and 205 unrelated entities')
        fixture=json.loads(subprocess.check_output([str(repo/'target/debug/examples/bible_recall_capture_fixture')],
            env=environment,text=True,timeout=60))
        database=Path(fixture['project_path']).resolve()
        a_id,f_id,b_id=(fixture[key]['id'] for key in ('a','f','b'))
        if ui.blocks(database,b_id):
            raise RuntimeError('Native generation target was already populated')
        evidence['public_fixture']=fixture
        evidence['public_fixture']['project_path']='<fresh qualification project>'
        original_fact=fact(database)
        with raw_log.open('w') as stream:
            process=subprocess.Popen(['./launcher.sh','--run'],env=environment,
                stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)

        def owned_window():
            if process.poll() is not None:
                raise RuntimeError('Launcher exited: '+str(process.returncode))
            windows=subprocess.run(['xdotool','search','--onlyvisible','--name','^Eidetic$'],
                capture_output=True,text=True,timeout=5)
            for candidate in windows.stdout.splitlines():
                pid=int(ui.command('xdotool','getwindowpid',candidate))
                if os.getpgid(pid)==process.pid:
                    return candidate,pid
            return None

        window,pid=ui.wait_for('owned native window',owned_window)
        ui.NATIVE_WINDOW_PID=pid
        evidence.update(window_id=int(window),window_pid=pid)
        evidence['application_binary_sha256']=ui.file_hash(Path('/proc/'+str(pid)+'/exe'))
        ui.command('xdotool','windowsize','--sync',window,'1920','1440')
        evidence['native_window_geometry']=ui.command('xdotool','getwindowgeometry','--shell',window)
        application=ui.wait_for('native accessibility app',lambda:next((app for app in ui.pyatspi.Registry.getDesktop(0) if app.get_process_id()==pid),None))
        ui.open_project_chooser(application,window)
        ui.click_button(application,database.parent.name,window,prefix=True)
        ui.choose_mode(application,'Script',window)
        enlarge_script_pane(application,window)
        ui.reveal_button(application,'Zoom to Fit (Ctrl+0)',window)
        for _ in range(3):
            ui.reveal_button(application,'+',window)
        ui.click_button(application,'AI',window)
        ui.click_button(application,'Save & Connect',window)
        ui.wait_for('connected synthetic fixture',lambda:visible_text(application,'Connected'))
        clip=ui.wait_for('B timeline clip',lambda:ui.native_timeline_clip(application,fixture['b']['name'],window))
        ui.click_control(clip[0],window,clip[1])
        ui.reveal_button(application,'Generate',window)
        generated=ui.wait_for('native generated B',lambda:next((r for r in ui.blocks(database,b_id) if r[1]==GENERATED),None))
        ui.wait_for('generation and recap completed',lambda:len([r for r in RecallProvider.records if r.get('accepted')])==2
            and ui.find(application,lambda n:n.name=='Generate' and n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and n.getState().contains(ui.pyatspi.STATE_ENABLED)))
        evidence['generation']={'actual_GUI_Generate':True,'real_model':False,'block_id':generated[0],'revision_event_id':generated[2]}
        block=ui.wait_for('generated B edit surface',lambda:ui.screenplay_block(application,screenplay_anchor(GENERATED)))
        ui.reveal_button(block,'Edit',window)
        field=ui.wait_for('generated B editor',lambda:ui.editable(application,GENERATED))
        ui.type_text(field,window,MANUAL)
        block=ui.wait_for('manual B draft surface',lambda:ui.screenplay_block(application,screenplay_anchor(MANUAL)))
        ui.reveal_button(block,'Save',window)
        manual=ui.wait_for('exact saved manual B',lambda:next((r for r in ui.blocks(database,b_id) if r[1]==MANUAL),None))
        block=ui.wait_for('F supporting scene',lambda:ui.screenplay_block(application,screenplay_anchor(F_TEXT)))
        ui.reveal_button(block,'Edit',window)
        draft=ui.wait_for('F edit area',lambda:ui.editable(application,F_TEXT))
        ui.type_text(draft,window,DRAFT)
        original={node_id:ui.blocks(database,node_id) for node_id in (a_id,f_id,b_id)}
        evidence['saved_manual_B']={'exact_text':MANUAL,'block_id':manual[0],'revision_event_id':manual[2]}
        evidence['retained_F_draft']={'exact_text':DRAFT,'not_saved':True,'saved_F_text':F_TEXT}
        checkpoint('exact manual B saved and unrelated F draft retained')
        ui.click_button(application,'Bible',window)
        search=ui.wait_for('Bible entity search before selection',lambda:ui.find(application,lambda n:
            n.getState().contains(ui.pyatspi.STATE_EDITABLE)
            and 0 <= n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0] < 400
            and ui.text_of(n)==''))
        ui.type_text(search,window,'Mara')
        mara=ui.wait_for('exact Mara entity control',lambda:ui.reveal(application,lambda n:
            n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and ui.button_label_matches(n.name,'Mara',prefix=True)))
        ui.click_control(mara,window)
        disclosure=ui.wait_for('related-fact disclosure',lambda:recall_disclosure(application))
        ui.click_control(disclosure,window)
        before_history=canonical_state(database)
        ui.reveal_button(application,'Recall related story facts',window)
        ui.wait_for('one-hop neighbor with unresolved timed fact',lambda:visible_text(application,'Unresolved: environment.weather'))
        require_preserved(database,original,before_history)
        evidence['untimed_recall']={'anchor':'qualification.mara','neighbor':'qualification.beach-house',
            'exact_UI_unresolved':'Unresolved: environment.weather','story_time_ms':None,'canonical_history_unchanged':True}
        checkpoint('explicit untimed recall reaches linked Beach House beyond renderer prefix')
        capture('eidetic-bible-recall-untimed.png')
        recall_time(application,window,'999')
        ui.wait_for('baseline weather at 999ms',lambda:visible_text(application,'environment.weather: Dry'))
        require_preserved(database,original,before_history)
        checkpoint('explicit 999ms recall resolves whole baseline without future assertion')
        capture('eidetic-bible-recall-before.png')
        recall_time(application,window,'1000')
        ui.wait_for('timed Rain assertion at 1000ms',lambda:visible_text(application,'environment.weather: Rain'))
        ui.wait_for('typed explanation path',lambda:visible_text(application,'Mara → Beach House · located in'))
        require_preserved(database,original,before_history)
        evidence['timed_recall']={'story_time_ms':1000,'exact_UI_value':'environment.weather: Rain',
            'typed_path':'Mara → Beach House · located in','relationships_untimed':True,'canonical_history_unchanged':True}
        checkpoint('explicit 1000ms recall resolves exact assertion with typed untimed path')
        capture('eidetic-bible-recall-at.png')
        # Close only the normal recall disclosure to reach the existing authored profile editor.
        disclosure=ui.wait_for('open recall disclosure',lambda:recall_disclosure(application))
        ui.click_control(disclosure,window)
        field=ui.wait_for('exact blue authored Bible field',lambda:ui.reveal(application,lambda n:
            n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n)==BLUE))
        ui.type_text(field,window,AMBER)
        save=ui.wait_for('associated authored fact Save',lambda:save_fact_control(application,field))
        ui.click_control(save,window)
        edited=ui.wait_for('exact amber fact and new owned revision',lambda:fact(database) if fact(database)[0]==AMBER and fact(database)[1]!=original_fact[1] else None)
        require_preserved(database,original)
        ui.wait_for('retained F draft after Bible event',lambda:ui.editable(application,DRAFT))
        evidence['manual_fact_edit']={'before':BLUE,'after':AMBER,'original_revision_event_id':original_fact[1],
            'current_revision_event_id':edited[1],'saved_screenplay_unchanged':True,'unrelated_draft_retained':True}
        notice=ui.wait_for('downstream What changed',lambda:ui.reveal(application,lambda n:n.name=='What changed'))
        ui.click_control(notice,window)
        evidence['manual_fact_edit']['visible_review']=ui.wait_for('visible Bible fact cause',lambda:visible_review_label(application,window,'Bible fact profile.tagline changed.'))
        checkpoint('native exact fact Save identifies affected screenplay without replacing manual text')
        capture('eidetic-bible-recall-fact-review.png')
        disclosure=ui.wait_for('reopen recall disclosure',lambda:recall_disclosure(application))
        ui.click_control(disclosure,window)
        ui.wait_for('old recall evidence revoked',lambda:visible_text(application,'Facts changed. Recall again'))
        if visible_text(application,'environment.weather: Rain'):
            raise RuntimeError('Old recall values survived the authored fact edit')
        checkpoint('Bible mutation revokes prior recall evidence')
        capture('eidetic-bible-recall-invalidated.png')
        updated_history=canonical_state(database)
        ui.reveal_button(application,'Recall related story facts',window)
        ui.wait_for('fresh exact amber recalled field',lambda:visible_text(application,'profile.tagline: '+AMBER))
        ui.wait_for('fresh timed neighbor fact',lambda:visible_text(application,'environment.weather: Rain'))
        require_preserved(database,original,updated_history)
        evidence['refreshed_recall']={'exact_anchor_value':AMBER,'story_time_ms':1000,'canonical_history_unchanged':True}
        checkpoint('explicit fresh recall reflects edited canon and leaves existing work unchanged')
        capture('eidetic-bible-recall-refreshed.png')
        scroll_script_start(application,window,screenplay_anchor(A_TEXT))
        ui.reveal_button(application,'Preview update',window)
        pending=ui.wait_for('saved pending targeted proposal',lambda:ui.query(database,
            "SELECT id,proposed_text,status FROM propagation_proposals WHERE status='pending' AND proposed_text=?", (PROPOSED,)))
        ui.require_accepted_preview(RecallProvider.records)
        evidence['pending_preview']={'proposal_id':pending[0][0],'proposed_text':PROPOSED,'status':'pending','real_model':False}
        evidence['pending_preview']['visible_proposal']=reveal_visible_proposal(application,window)
        require_preserved(database,original)
        ui.wait_for('retained draft with pending review',lambda:ui.editable(application,DRAFT))
        checkpoint('targeted synthetic proposal visible with explicit acceptance and saved manual B retained')
        capture('eidetic-bible-recall-pending-review.png')
        evidence['provider_records']=RecallProvider.records
        if len(RecallProvider.records)!=3 or not all(r.get('accepted') and not r['real_model'] for r in RecallProvider.records):
            raise RuntimeError('Unexpected or unlabelled synthetic provider records')
        evidence['saved_screenplay_after']=original
        evidence['status']='native_explicit_bible_recall_passed_with_labelled_synthetic_review'
        checkpoint('qualified; saved screenplay, draft and pending review preserved')
    except Exception as error:
        evidence['error']=str(error)[:2000]
        if application is not None:
            evidence['accessibility']=ui.accessibility_snapshot(application)
        if window is not None:
            try: capture('eidetic-bible-recall-failure.png')
            except Exception as capture_error: evidence['failure_capture_error']=str(capture_error)[:500]
        raise
    finally:
        evidence['provider_records']=RecallProvider.records
        evidence['accessibility_retirements']=ui.ACCESSIBILITY_RETIREMENTS
        evidence['typed_text_observations']=ui.TYPED_TEXT_OBSERVATIONS
        (output/'capture-evidence.json').write_text(json.dumps(evidence,indent=2)+'\n')
        if process and process.poll() is None:
            os.killpg(process.pid,signal.SIGTERM)
            try:process.wait(timeout=10)
            except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL)
        provider.shutdown();provider.server_close()
        raw=raw_log.read_text(errors='replace') if raw_log.exists() else 'No app launch log.'
        (output/'app-sanitized.log').write_text(ui.sanitized_log(raw,[(str(state_root),'<qualification state>'),(str(repo),'<repo>')]))


if __name__=='__main__':
    main()

