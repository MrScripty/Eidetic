"""Established native geometry helpers; no DOM/IPC injection or canonical writes."""
import importlib.util
from pathlib import Path
import time
spec=importlib.util.spec_from_file_location('authoring_ui',Path(__file__).with_name('qualify-screenplay-authoring.py'))
ui=importlib.util.module_from_spec(spec)
spec.loader.exec_module(ui)
def screenplay_anchor(exact_text):
    # ScriptView renders screenplay headings/action/dialogue as separate nodes;
    # a raw multiline block cannot be a substring of any single rendered child.
    # This fixture's distinctive action line identifies the native block, while
    # textarea and canonical-read checks below still require every exact byte.
    lines = [line for line in exact_text.splitlines() if line.strip()]
    if not lines:
        raise RuntimeError('QA screenplay anchor has no authored text')
    return (lines[1] if len(lines) > 1 else lines[0]).strip()

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

def visible_text(application, label):
    # Recall results extend below the nested Bible sidebar viewport. Native
    # scrolling must expose the exact paragraph before claiming it visible.
    expected = ' '.join(label.split())
    def matches(node):
        role = node.getRole()
        allowed = role != ui.pyatspi.ROLE_COMBO_BOX if label == 'Connected' else role in (
            ui.pyatspi.ROLE_PARAGRAPH, ui.pyatspi.ROLE_STATUS_BAR, ui.pyatspi.ROLE_TEXT)
        return allowed and expected in ' '.join(ui.text_of(node).split())
    found = ui.find(application, matches)
    if found:
        return found
    # An enclosing hidden section can expose descendant text but cannot be
    # scrolled into view. Reveal only the actual offscreen result paragraph.
    return ui.reveal(application, lambda n: n.getRole() in (
        ui.pyatspi.ROLE_PARAGRAPH, ui.pyatspi.ROLE_STATUS_BAR, ui.pyatspi.ROLE_TEXT) and matches(n))

ui.walk = walk_live

def native_input_steps(text):
    """Native keys preserve LF and Unicode that xdotool type can omit."""
    steps=[];ascii_text=''
    for character in text:
        if character!='\n' and ord(character)<128:
            ascii_text+=character;continue
        if ascii_text:steps.append(('type',ascii_text));ascii_text=''
        steps.append(('newline','') if character=='\n' else ('unicode',format(ord(character),'x')))
    if ascii_text:steps.append(('type',ascii_text))
    return steps


def type_text(node,window,text):
    import subprocess
    ui.click_control(node,window)
    ui.wait_for('native text focus',lambda:node.getState().contains(ui.pyatspi.STATE_FOCUSED))
    ui.command('xdotool','key','--clearmodifiers','ctrl+a')
    for kind,value in native_input_steps(text):
        if kind=='type':subprocess.run(['xdotool','type','--clearmodifiers','--delay','10',value],check=True,timeout=20)
        elif kind=='newline':ui.command('xdotool','key','--clearmodifiers','Return')
        else:
            # Ordinary GTK Unicode composition, driven entirely with native keys.
            ui.command('xdotool','key','--clearmodifiers','ctrl+shift+u')
            ui.command('xdotool','type','--clearmodifiers',value)
            ui.command('xdotool','key','--clearmodifiers','Return')
    def exact():
        node.clear_cache();actual=ui.text_of(node)
        observation={'field':node.name,'expected_length':len(text),'actual_text':actual,'exact':actual==text,'route':'native ASCII/Return and GTK Unicode composition'}
        if not ui.TYPED_TEXT_OBSERVATIONS or ui.TYPED_TEXT_OBSERVATIONS[-1]!=observation:ui.TYPED_TEXT_OBSERVATIONS.append(observation)
        return actual==text
    ui.wait_for('exact native typed text including Unicode/whitespace',exact)

ui.type_text=type_text


def bible_viewport(application,window,side):
    geometry={key:int(value) for key,value in (line.split('=',1) for line in
        ui.command('xdotool','getwindowgeometry','--shell',window).splitlines())}
    x,y,width,height=script_viewport(application,window)
    bottom=y+height
    if side=='left':return (geometry['X'],geometry['Y'],x-geometry['X'],bottom-geometry['Y'])
    if side=='right':return (x+width,geometry['Y'],geometry['X']+geometry['WIDTH']-x-width,bottom-geometry['Y'])
    raise RuntimeError('Unknown Bible inspector side')


def find_bible_editor(application,window,value,side):
    viewport=bible_viewport(application,window,side)
    return ui.reveal(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE)
        and ui.text_of(n)==value and viewport[0]<=n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0]<viewport[0]+viewport[2])


def bible_field(application,window,value,side,align=False):
    node=ui.wait_for('exact '+side+' Bible editor',lambda:find_bible_editor(application,window,value,side))
    if align:
        node.queryComponent().scrollTo(ui.pyatspi.SCROLL_TOP_LEFT)
        application.clear_cache()
        node=ui.wait_for('aligned '+side+' Bible editor',lambda:find_bible_editor(application,window,value,side))
    viewport=bible_viewport(application,window,side)
    rectangles=text_rectangles(node,value)
    if not rectangles or not all(contained(r,viewport) for r in rectangles):
        raise RuntimeError('Exact Bible text not bounded in '+side+' native inspector')
    return {'side':side,'native_name':node.name,'exact_text':value,
        'native_bounds':list(node.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)),
        'viewport':list(viewport),'character_bounds':[list(r) for r in rectangles]}
