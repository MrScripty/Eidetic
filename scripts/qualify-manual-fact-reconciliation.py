#!/usr/bin/env python3
"""Exact frozen native source; ordinary UI actions, labelled public QA replays.

Synthetic HTTP only. No DOM/IPC injection, direct DB writes or real inference.
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
from urllib.request import urlopen

spec=importlib.util.spec_from_file_location('native_helpers',Path(__file__).with_name('manual-fact-native-helpers.py'))
driver=importlib.util.module_from_spec(spec);spec.loader.exec_module(driver)
ui=driver.ui
SOURCE='d20dcd4126f2317c155b87a2f77db83cac61e10a'
SOURCE_TREE='6bd5e7689afc70e85ee5b55d9563be14e82288a9'
KNOWN_EMPTY_MODES=('known-empty-name','known-empty-second-tag','known-empty-deletion')
KNOWN_EMPTY_NAME='Mara chooses the road — 雨.'
KNOWN_EMPTY_PREVIEW='  Synthetic known-empty preview: Mara considers the road — 雨.\n\n  '
ARC_NEW='  Mara chooses exile — 雨.\n\n  '
ARC_UNRELATED='Unrelated direction: Eli stays at the station.\n\n'
ARC_PREVIEW='  Synthetic arc update: Mara chooses exile — 雨.\n\n  '
NOTES='  Mara reveals the witness — 雨.\n\n  '
ANCESTOR_OLD='  Mara conceals the witness — 雨.\n\n  '
ANCESTOR_MANUAL='  Exact authored B: Eli keeps his whistle — 雨.\n\n  '
UNRELATED_ACT_OLD='Unrelated Act: Eli guards the station.'
NOTES_PREVIEW='  Synthetic Notes update: Mara identifies the witness — 雨.\n\n  '
RED="Mara's umbrella is red."
BLUE="Mara's umbrella is blue — 雨.\n\n"
GREEN="Synthetic later proposal: Mara's umbrella is green."
SAVED='  Mara carries a blue umbrella — 雨.\n\n  '
TEMP='Temporary manual source change — 雨.\n\n'
F_TEXT='Unrelated saved scene: Eli guards the station.\n\n'
DRAFT='Unrelated draft: Eli pockets a brass whistle.\n\n'
BIBLE_DRAFT='  Unaccepted local Bible draft — 雨.\n\n  '
MOTIVATION_DRAFT='  Unaccepted motivation: protect Eli — 雨.\n\n  '
MOTIVATION='UNCONSUMED sibling: preserve the witness.'
PENDING_DRAFT='  Pending blue draft — 雨.\n\n  '
RECOVERY_DRAFT='  Recovery draft — 雨.\n\n  '
NEWER_FACT='Green saved fact — 雨.'
GENERATED_A='Synthetic A: Mara checks her red umbrella.\n\n'
GENERATED_B='Synthetic B: Mara carries her red umbrella.\n\n'


def fact_prompt(user,phase):
    try: edit=json.loads(user)
    except ValueError:return False
    facts=edit.get('facts',[])
    decoded=json.dumps(edit,ensure_ascii=False)
    no_local_drafts=all(json.dumps(draft,ensure_ascii=False)[1:-1] not in decoded for draft in (DRAFT,BIBLE_DRAFT,MOTIVATION_DRAFT))
    return (edit.get('text')==SAVED and edit.get('before_revision_event_id')!=edit.get('revision_event_id')
        and len(facts)==1 and facts[0].get('field_id')=='qualification.mara.tagline'
        and facts[0].get('text')==(BLUE if phase==4 else RED)
        and facts[0].get('consumed_text')==RED and 'UNCONSUMED' not in decoded and no_local_drafts)

def notes_prompt(system,user):
    return ('targeted screenplay update' in system and
        'CURRENT SELECTED CLIP NOTES:\n'+NOTES in user and GENERATED_B.strip() in user
        and RED in user and DRAFT not in user)

def ancestor_notes_prompt(system,user):
    return ('targeted screenplay update' in system and
        'CURRENT CONSUMED ANCESTOR NOTES (' in user and '):\n'+NOTES+'\n' in user
        and 'TARGET BLOCK TO UPDATE:\n'+ANCESTOR_MANUAL in user
        and RED in user and DRAFT not in user and UNRELATED_ACT_OLD not in user)


def arc_description_prompt(system,user):
    return ('targeted screenplay update' in system and ARC_NEW in user
        and 'ORIGINAL KNOWN-EMPTY ARC DESCRIPTION APPLICABILITY (' in user
        and 'description prose was not supplied; field revision ' in user
        and 'TARGET BLOCK TO UPDATE:\n'+ANCESTOR_MANUAL in user
        and RED in user and DRAFT not in user and ARC_UNRELATED not in user)


def known_empty_arc_prompt(system,user,mode,phase):
    if mode not in KNOWN_EMPTY_MODES or phase not in (0,1):return False
    if mode!='known-empty-deletion' and phase!=0:return False
    if not ('targeted screenplay update' in system and 'TARGET BLOCK TO UPDATE:\n'+ANCESTOR_MANUAL+'\n\nPROVEN INPUT CHANGE:\n' in user and RED in user and DRAFT not in user and ARC_UNRELATED not in user):return False
    omission='ORIGINAL KNOWN-EMPTY ARC DESCRIPTION APPLICABILITY ('
    if user.count(omission)!=(2 if mode=='known-empty-second-tag' else 1) or user.count('description prose was not supplied; field revision ')!=user.count(omission):return False
    try:
        cause=json.loads(user.split('\nPROVEN INPUT CHANGE:\n',1)[1].split('\nReturn only',1)[0])
        source=cause['input'];arc=source['arc_id']
    except (KeyError,ValueError,IndexError):return False
    if source.get('kind')!='story_arc_field':return False
    if mode=='known-empty-second-tag':
        return source.get('field')=='description' and cause.get('reason')=='context_changed' and '\n'+arc+' description: '+ARC_NEW+'\n' in user and 'Still-empty supporting arc' in user
    if source.get('field')!='name':return False
    if mode=='known-empty-deletion' and phase==1:
        return cause.get('reason')=='deleted' and 'CURRENT TAGGED STORY ARC FIELDS:' not in user
    return cause.get('reason')!='deleted' and '\n'+arc+' name: '+KNOWN_EMPTY_NAME+'\n' in user and '\n'+arc+' description:' not in user


REMOVAL_PREVIEW='  Synthetic removal update: Eli keeps his whistle; the removed source is no longer current — 雨.\n\n  '

def removal_prompt(system,user,phase):
    if phase not in (0,1) or 'targeted screenplay update' not in system or DRAFT in user:return False
    if 'TARGET BLOCK TO UPDATE:\n'+ANCESTOR_MANUAL not in user or RED not in user:return False
    try:
        cause=json.loads(user.split('\nPROVEN INPUT CHANGE:\n',1)[1].split('\nReturn only',1)[0])
    except (ValueError,IndexError):return False
    if cause.get('input',{}).get('kind')!='script_block' or cause.get('input_excerpt')!=GENERATED_A:return False
    current=user.split('\nPROVEN INPUT CHANGE:\n',1)[0]
    return (cause.get('reason')=='changed' and SAVED in current) if phase==0 else (cause.get('reason')=='deleted' and cause.get('current_revision_event_id') is None and SAVED not in current and GENERATED_A not in current)


def notes_review(application):
    return ui.reveal(application,lambda n:n.name=='Screenplay update proposals' and any(
        child.name=='Current preview Notes' or child.getRole()==ui.pyatspi.ROLE_COMBO_BOX and 'Timeline Notes changed.' in child.name
        for child in ui.walk(n)))


def review_for_source(application,clip_name):
    # Follow the native accessibility document order from the ordinary source
    # clip landmark. Multicolumn rendering can separate adjacent components.
    application.clear_cache();selected=False
    for node in ui.walk(application):
        if node.name=='Screenplay source clip':
            selected=any(ui.text_of(child)==clip_name for child in ui.walk(node))
        elif selected and node.name=='Screenplay update proposals':
            node.queryComponent().scrollTo(ui.pyatspi.SCROLL_ANYWHERE)
            return node if ui.visible(node) else None
    return None


TITLE_NEW='  Station departure — 雨.  '
TITLE_LATER='  Station departure — rain returns.  '
TITLE_PREVIEW='  Synthetic title update: Eli departs from the station — 雨.\n\n  '

def title_prompt(system,user,phase):
    current=TITLE_NEW if phase==0 else TITLE_LATER
    return ('targeted screenplay update' in system and ANCESTOR_MANUAL in user
        and 'ORIGINAL CONSUMED TIMELINE TITLE' in user and '\nSCENE A\n' in user
        and 'CURRENT CONSUMED TIMELINE TITLE' in user and '\n'+current+'\n' in user
        and 'timeline_title.' in user and '"kind":"timeline_node"' in user
        and 'Return only the complete replacement text' in user
        and DRAFT not in user and BIBLE_DRAFT not in user)

def title_input(database,node):
    rows=ui.query(database,'SELECT name FROM nodes WHERE id=?',(node,))
    return rows[0][0] if rows else None

def select_title(application,window,name):
    clip=ui.wait_for('ordinary named timeline clip '+name,lambda:ui.native_timeline_clip(application,name,window))
    ui.click_control(clip[0],window,clip[1])
    return ui.wait_for('ordinary Clip title editor',lambda:ui.reveal(application,lambda n:n.name=='Clip title' and n.getState().contains(ui.pyatspi.STATE_EDITABLE)))

class Provider(ui.FixtureProvider):
    records=[]
    allow_fresh_notes=False
    def do_POST(self):
        size=int(self.headers.get('Content-Length','0'))
        if self.path!='/v1/chat/completions' or not 0<size<=512_000:
            self.send_error(400);return
        body=json.loads(self.rfile.read(size));messages=body.get('messages',[])
        system=next((m['content'] for m in messages if m.get('role')=='system'),'')
        user=next((m['content'] for m in messages if m.get('role')=='user'),'')
        kind='notes_preview' if 'targeted screenplay update' in system else 'fact' if 'Analyze one saved manual screenplay edit' in system else 'recap' if user.startswith('Generate a scene recap for this screenplay beat:') else 'generation'
        with self.records_lock:
            phase=sum(r.get('accepted') and r['kind']==kind for r in self.records)
            valid=body.get('stream') is True
            if kind=='generation':
                valid=valid and phase<2 and RED in user and 'UNCONSUMED' not in user
                if os.environ.get('EIDETIC_CAPTURE_SCOPE')=='ancestor-notes':valid=valid and ANCESTOR_OLD in user and UNRELATED_ACT_OLD not in user
                if os.environ.get('EIDETIC_CAPTURE_SCOPE')=='arc-description' or os.environ.get('EIDETIC_CAPTURE_SCOPE') in KNOWN_EMPTY_MODES:valid=valid and ARC_NEW not in user and ARC_UNRELATED not in user and "Mara's choice" in user
            elif kind=='recap':valid=valid and phase<2 and (GENERATED_A if phase==0 else GENERATED_B).strip() in user
            elif kind=='notes_preview':valid=valid and (phase==0 or phase==1 and self.allow_fresh_notes) and (title_prompt(system,user,phase) if os.environ.get('EIDETIC_CAPTURE_SCOPE','').startswith('timeline-title') else removal_prompt(system,user,phase) if os.environ.get('EIDETIC_CAPTURE_SCOPE','').startswith('screenplay-removal') else known_empty_arc_prompt(system,user,os.environ['EIDETIC_CAPTURE_SCOPE'],phase) if os.environ.get('EIDETIC_CAPTURE_SCOPE') in KNOWN_EMPTY_MODES else arc_description_prompt(system,user) if os.environ.get('EIDETIC_CAPTURE_SCOPE')=='arc-description' else ancestor_notes_prompt(system,user) if os.environ.get('EIDETIC_CAPTURE_SCOPE')=='ancestor-notes' else notes_prompt(system,user))
            else:valid=valid and phase<5 and fact_prompt(user,phase)
            record={'kind':kind,'phase':phase,'accepted':valid,'real_model':False,'synthetic':True,'exact_system_prompt':system,'exact_user_prompt':user}
            self.records.append(record)
        if not valid:self.send_error(422,'Synthetic qualification prompt/phase mismatch');return
        text=(GENERATED_A if phase==0 else GENERATED_B) if kind=='generation' else 'Synthetic recap: Mara carries red.' if kind=='recap' else (TITLE_PREVIEW if os.environ.get('EIDETIC_CAPTURE_SCOPE','').startswith('timeline-title') else REMOVAL_PREVIEW if os.environ.get('EIDETIC_CAPTURE_SCOPE','').startswith('screenplay-removal') else KNOWN_EMPTY_PREVIEW if os.environ.get('EIDETIC_CAPTURE_SCOPE') in KNOWN_EMPTY_MODES else ARC_PREVIEW if os.environ.get('EIDETIC_CAPTURE_SCOPE')=='arc-description' else NOTES_PREVIEW) if kind=='notes_preview' else json.dumps({'value':GREEN if phase==4 else BLUE,'rationale':'SYNTHETIC QA response: human acceptance required; no model quality claim.'})
        record['synthetic_response']=text
        parts=[text[:len(text)//2],text[len(text)//2:]]
        data=''.join('data: '+json.dumps({'choices':[{'delta':{'content':p}}]})+'\n\n' for p in parts).encode()+b'data: [DONE]\n\n'
        self.send_response(200);self.send_header('Content-Type','text/event-stream');self.send_header('Content-Length',str(len(data)));self.end_headers();self.wfile.write(data)


def receipt(application):
    def read():
        node=ui.find(application,lambda n:n.name=='QA saved-edit receipt' and n.getRole() in (ui.pyatspi.ROLE_TEXT,ui.pyatspi.ROLE_ENTRY))
        if not node:return None
        # The receipt can exceed the generic text helper's short diagnostic bound.
        text=node.queryText();value=json.loads(text.getText(0,text.characterCount))
        if not value['fixture'].startswith('qualifier-only public commands'):raise RuntimeError('Missing explicit QA label')
        if value['error']:raise RuntimeError('Public QA command failure: '+value['error'])
        return value
    # Rerendering can retire a child during traversal after an ordinary edit.
    # Reacquire only this read through the existing deadline/liveness guard;
    # never replay the action, swallow arbitrary failures, or relax assertions.
    return ui.wait_for('live labelled public QA receipt',read)


def disclosure(application,window,label):
    node=ui.wait_for('native disclosure '+label,lambda:ui.reveal(application,lambda n:n.name==label and n.getRole() in (ui.pyatspi.ROLE_UNKNOWN,ui.pyatspi.ROLE_PUSH_BUTTON,ui.pyatspi.ROLE_TOGGLE_BUTTON)))
    ui.click_control(node,window)


def field(database):
    return ui.query(database,"SELECT text_value,updated_event_id FROM bible_graph_fields WHERE id='qualification.mara.tagline'")[0]


def proposal(database,status='pending'):
    rows=ui.query(database,"SELECT id,script_fact_binding_json FROM propagation_proposals WHERE status=? AND script_fact_binding_json IS NOT NULL ORDER BY rowid DESC",(status,))
    return (rows[0][0],json.loads(rows[0][1])) if rows else None


def proposal_row(database,pid):return ui.query(database,'SELECT * FROM propagation_proposals WHERE id=?',(pid,))


def material(database):
    # Read-only, including exact spans/segment ownership; no synthesized projections.
    result={table:ui.query(database,f'SELECT * FROM {table} ORDER BY rowid') for table in ('script_blocks','script_spans','script_segments','script_locks')}
    result['timeline_placement']=ui.query(database,'SELECT id,parent_id,level,sort_order,start_ms,end_ms FROM nodes ORDER BY id')
    return result


def bible_fields(database):return ui.query(database,'SELECT * FROM bible_graph_fields ORDER BY rowid')


def require_draft(application):return ui.wait_for('exact unrelated draft retained',lambda:ui.reveal(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n)==DRAFT))


def save_script(application,window,database,node_id,current,new):
    block=ui.wait_for('source screenplay block',lambda:ui.screenplay_block(application,driver.screenplay_anchor(current)))
    ui.reveal_button(block,'Edit',window)
    editor=ui.wait_for('source screenplay editor',lambda:ui.editable(application,current))
    ui.type_text(editor,window,new)
    block=ui.wait_for('edited source block',lambda:ui.screenplay_block(application,driver.screenplay_anchor(new)))
    ui.reveal_button(block,'Save',window)
    return ui.wait_for('exact canonical saved source',lambda:next((r for r in ui.blocks(database,node_id) if r[1]==new),None))


def select_consumed(application,window):
    # WebKit appends the current option to this control's accessible label.
    node=ui.wait_for('ordinary consumed fact dropdown',lambda:ui.reveal(application,lambda n:n.getRole()==ui.pyatspi.ROLE_COMBO_BOX and ui.button_label_matches(n.name,'Consumed baseline fact',prefix=True)))
    ui.click_control(node,window)
    ui.command('xdotool','key','--clearmodifiers','Home','Down','Return')
    ui.wait_for('ordinary Analyze enabled after selection',lambda:ui.reveal(application,lambda n:n.name=='Analyze saved edit' and n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and n.getState().contains(ui.pyatspi.STATE_ENABLED)))
    return {'name':node.name,'route':'native combo Home/Down/Return'}


def analyze(application,window,database):
    count=len(Provider.records)
    prior=ui.query(database,'SELECT count(*) FROM propagation_proposals')[0][0]
    ui.reveal_button(application,'Analyze saved edit',window)
    def pending():
        if len(Provider.records)>count and not Provider.records[-1]['accepted']:raise RuntimeError('Synthetic provider rejected actual prompt; inspect receipt')
        return proposal(database) if ui.query(database,'SELECT count(*) FROM propagation_proposals')[0][0]==prior+1 else None
    result=ui.wait_for('canonical pending fact proposal',pending)
    if result[1]['edit']['facts'][0]['field_id']!='qualification.mara.tagline' or len(result[1]['edit']['facts'])!=1:raise RuntimeError('Wrong consumed field selection')
    return result


def fact_article(application,status='pending'):
    return ui.reveal(application,lambda n:n.name=='Bible fact proposal' and any(
        ('qualification.mara · profile.tagline · '+status) in ' '.join(ui.text_of(child).split())
        for child in ui.walk(n)))


def visible_proposal(application,window,value,status='pending'):
    article=ui.wait_for('pending native fact article',lambda:fact_article(application,status))
    node=ui.wait_for('native pending proposed Bible fact',lambda:ui.reveal(article,lambda n:n.name=='Proposed Bible fact' and ui.text_of(n)==value))
    node.queryComponent().scrollTo(ui.pyatspi.SCROLL_TOP_LEFT)
    for _ in range(24):
        viewport=driver.script_viewport(application,window)
        rects=driver.text_rectangles(node,value)
        controls=[n for n in ui.walk(article) if n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and n.name in ('Accept fact update','Reject fact update') and ui.visible(n)]
        if rects and all(driver.contained(r,viewport) for r in rects) and (status!='pending' or len(controls)==2 and all(driver.contained(tuple(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)),viewport) for n in controls)):
            return {'viewport':list(viewport),'character_bounds':[list(r) for r in rects],'exact_text':value,'ordinary_controls':[{'name':n.name,'bounds':list(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN))} for n in controls]}
        direction='6' if rects and min(r[0] for r in rects)<viewport[0] else '7'
        ui.command('xdotool','mousemove',str(viewport[0]+viewport[2]//2),str(viewport[1]+viewport[3]//2))
        ui.command('xdotool','click','--repeat','2','--delay','50',direction)
        time.sleep(.1);application.clear_cache()
        article=ui.wait_for('fresh pending article',lambda:fact_article(application,status))
        node=ui.wait_for('fresh proposed fact object',lambda:next((n for n in ui.walk(article) if n.name=='Proposed Bible fact' and ui.text_of(n)==value),None))
    raise RuntimeError('Fact proposal and ordinary controls not bounded inside Script viewport')


def reject(application,window,database,pid):
    ui.reveal_button(application,'Reject fact update',window)
    ui.wait_for('ordinary rejected fact proposal',lambda:ui.query(database,"SELECT id FROM propagation_proposals WHERE id=? AND status='rejected'",(pid,)))


def stale_accept(application,window,database,pid,evidence,label):
    before=driver.canonical_state(database);pending=proposal_row(database,pid);saved=material(database);fact_before=field(database)
    ui.reveal_button(application,'Accept fact update',window)
    error=ui.wait_for('visible ordinary stale fact refusal',lambda:ui.find(application,lambda n:n.name!='QA saved-edit receipt' and 'Saved edit, consumed fact or relationship changed' in ui.text_of(n)))
    if before!=driver.canonical_state(database) or pending!=proposal_row(database,pid) or saved!=material(database) or fact_before!=field(database):raise RuntimeError('Stale refusal changed canonical state')
    require_draft(application)
    evidence['stale_checks'].append({'label':label,'proposal_id':pid,'ordinary_visible_refusal':ui.text_of(error),'native_role':error.getRoleName(),'history_pending_material_fact_unchanged':True})


def edit_fact(application,window,database,before,after):
    editor=ui.wait_for('ordinary Bible field editor',lambda:ui.reveal(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n)==before))
    ui.type_text(editor,window,after)
    save=ui.wait_for('associated verified Bible Save',lambda:(control if control and control.getState().contains(ui.pyatspi.STATE_ENABLED) else None) if (control:=driver.save_fact_control(application,editor)) else None)
    ui.click_control(save,window)
    return ui.wait_for('exact canonical Bible edit',lambda:field(database) if field(database)[0]==after else None)


def require_bible_drafts(application,window):
    # Ordinary native text only: both drafts remain local and never enter a command.
    dirty=driver.bible_field(application,window,BIBLE_DRAFT,'left',align=True)
    sibling=driver.bible_field(application,window,MOTIVATION_DRAFT,'left')
    return {'selected_field_draft':dirty,'unselected_field_draft':sibling}


def accepted_bible_display(application,window):
    drafts=require_bible_drafts(application,window)
    clean=driver.bible_field(application,window,BLUE,'right',align=True)
    left=driver.bible_viewport(application,window,'left')
    status=ui.wait_for('visible preserved Bible draft conflict',lambda:ui.reveal(application,lambda n:
        ui.text_of(n)=='Saved fact changed while editing. Your draft is preserved.'
        and driver.contained(tuple(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)),left)))
    committed=ui.wait_for('committed Bible fact beside preserved draft',lambda:ui.reveal(application,lambda n:
        n.name=='Committed Bible fact' and ui.text_of(n)==BLUE
        and driver.contained(tuple(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)),left)))
    base=ui.wait_for('original red draft base displayed',lambda:ui.find(application,lambda n:
        ui.text_of(n)==RED and not n.getState().contains(ui.pyatspi.STATE_EDITABLE)
        and driver.contained(tuple(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)),left)))
    save=ui.wait_for('conflicted Bible Save disabled',lambda:driver.save_fact_control(application,committed))
    if save.getState().contains(ui.pyatspi.STATE_ENABLED):raise RuntimeError('Conflicted draft Save remained enabled')
    discard=ui.wait_for('explicit draft discard available',lambda:ui.find(application,lambda n:
        n.name=='Discard draft' and n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON
        and n.getState().contains(ui.pyatspi.STATE_ENABLED)
        and driver.contained(tuple(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)),left)))
    return {'clean_committed_editor':clean,**drafts,'original_base':ui.text_of(base),
        'conflict_status':ui.text_of(status),'committed_value_beside_draft':ui.text_of(committed),
        'conflicted_save_disabled':True,'explicit_discard_available':bool(discard),
        'committed_character_bounds':[list(r) for r in driver.text_rectangles(committed,BLUE)]}


def qualify(application,window,database,fixture,capture,checkpoint,evidence):
    a_id,f_id,b_id=(fixture[key]['id'] for key in ('a','f','b'))
    original_bible=bible_fields(database);original_fact=field(database)
    before=ui.blocks(database,b_id)[0]
    saved=save_script(application,window,database,b_id,GENERATED_B,SAVED)
    if saved[2]==before[2] or saved[3:]!=before[3:]:raise RuntimeError('Saved manual revision/placement mismatch')
    evidence['ordinary_saved_edit']={'before':before,'after':saved,'exact_whitespace_and_unicode':True}
    block=ui.wait_for('unrelated F block',lambda:ui.screenplay_block(application,driver.screenplay_anchor(F_TEXT)))
    ui.reveal_button(block,'Edit',window)
    ui.type_text(ui.wait_for('unrelated editor',lambda:ui.editable(application,F_TEXT)),window,DRAFT)
    evidence['unsaved_draft']=DRAFT
    ui.click_button(application,'Bible',window)
    search=ui.wait_for('Bible entity search',lambda:ui.find(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and 0<=n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0]<400 and ui.text_of(n)==''))
    ui.type_text(search,window,'Mara')
    ui.click_control(ui.wait_for('Mara entity',lambda:ui.reveal(application,lambda n:n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and ui.button_label_matches(n.name,'Mara',prefix=True))),window)
    evidence['consumed_selection']=select_consumed(application,window)
    # Neither Save nor selection initiates analysis.
    if len(Provider.records)!=4 or field(database)!=original_fact:raise RuntimeError('Save/selection silently changed facts or ran provider')
    disclosure(application,window,'Saved screenplay edit used for analysis')
    capture('manual-fact-01-saved-edit.png')
    disclosure(application,window,'Saved screenplay edit used for analysis')
    frozen=material(database)
    first=analyze(application,window,database)
    evidence['first_proposal']={'id':first[0],'binding':first[1],'visible':visible_proposal(application,window,BLUE)}
    if material(database)!=frozen or bible_fields(database)!=original_bible:raise RuntimeError('Analysis changed saved material/facts')
    require_draft(application);visible_proposal(application,window,BLUE);checkpoint('one consumed-field analysis pending; exact manual text and draft preserved');capture('manual-fact-02-pending.png')
    history=driver.canonical_state(database);calls=len(Provider.records)
    ui.click_button(application,'QA replay exact analysis',window)
    replay=ui.wait_for('exact analysis idempotent replay',lambda:receipt(application) if receipt(application) and 'already_recorded' in receipt(application)['replay'] else None)
    if driver.canonical_state(database)!=history or len(Provider.records)!=calls:raise RuntimeError('Exact request replay repeated provider/write')
    evidence['analysis_replay']=replay
    reject(application,window,database,first[0])
    if material(database)!=frozen or bible_fields(database)!=original_bible:raise RuntimeError('Reject changed facts/material')
    evidence['rejected_proposal_id']=first[0];checkpoint('ordinary Reject preserves all authored material');capture('manual-fact-03-rejected.png')
    source_pending=analyze(application,window,database)
    source_before=ui.blocks(database,b_id)[0]
    save_script(application,window,database,b_id,SAVED,TEMP)
    source_restored=save_script(application,window,database,b_id,TEMP,SAVED)
    if source_restored[2]==source_before[2]:raise RuntimeError('Source ABA revision did not change')
    stale_accept(application,window,database,source_pending[0],evidence,'native screenplay edit/restore ABA')
    evidence['stale_checks'][-1].update(before_revision=source_before[2],restored_revision=source_restored[2])
    checkpoint('ordinary source edit/restore keeps same bytes but stale Accept refuses');capture('manual-fact-04-source-stale.png')
    reject(application,window,database,source_pending[0])
    fact_pending=analyze(application,window,database)
    before_fact=field(database);unchanged_material=material(database)
    edit_fact(application,window,database,RED,"Mara's umbrella is amber.")
    restored=edit_fact(application,window,database,"Mara's umbrella is amber.",RED)
    if restored[1]==before_fact[1] or material(database)!=unchanged_material:raise RuntimeError('Fact ABA revision/material mismatch')
    stale_accept(application,window,database,fact_pending[0],evidence,'ordinary Bible field edit/restore ABA')
    evidence['stale_checks'][-1].update(before_fact=before_fact,restored_fact=restored)
    checkpoint('ordinary fact edit/restore keeps same value but stale Accept refuses');capture('manual-fact-05-fact-stale.png')
    reject(application,window,database,fact_pending[0])
    fresh=analyze(application,window,database)
    evidence['fresh_proposal']={'id':fresh[0],'binding':fresh[1],'visible':visible_proposal(application,window,BLUE)}
    require_draft(application);visible_proposal(application,window,BLUE);capture('manual-fact-06-fresh-pending.png')
    # Right inspector stays clean; type both exact drafts into only the left Bible.
    dirty=ui.wait_for('left ordinary Bible fact editor',lambda:driver.find_bible_editor(application,window,RED,'left'))
    ui.type_text(dirty,window,BIBLE_DRAFT)
    sibling=ui.wait_for('left unconsumed motivation editor',lambda:driver.find_bible_editor(application,window,MOTIVATION,'left'))
    ui.type_text(sibling,window,MOTIVATION_DRAFT)
    evidence['bible_drafts_before_acceptance']=require_bible_drafts(application,window)
    driver.bible_field(application,window,RED,'right')
    if field(database)!=restored:raise RuntimeError('Typing Bible drafts wrote the canonical field')
    visible_proposal(application,window,BLUE)
    checkpoint('fresh pending review plus native unsaved Bible drafts; clean right editor still red')
    capture('manual-fact-06b-bible-drafts-pending.png')
    preserved=material(database);all_fields=bible_fields(database)
    ui.reveal_button(application,'Accept fact update',window)
    ui.wait_for('ordinary accepted fact proposal',lambda:ui.query(database,"SELECT id FROM propagation_proposals WHERE id=? AND status='accepted'",(fresh[0],)))
    after_fact=field(database)
    if after_fact[0]!=BLUE or after_fact[1]==restored[1] or material(database)!=preserved:raise RuntimeError('Fact acceptance rewrote material or missed chosen field')
    after_fields=bible_fields(database)
    # Only the chosen field row may change; stable rows include the unconsumed sibling and Eli.
    if [r for r in all_fields if r[0]!='qualification.mara.tagline']!=[r for r in after_fields if r[0]!='qualification.mara.tagline']:raise RuntimeError('Acceptance changed another Bible field')
    require_draft(application)
    evidence['accepted_bible_display']=accepted_bible_display(application,window)
    ui.click_button(application,'QA read canonical impacts',window)
    impacts=ui.wait_for('canonical impact receipt',lambda:receipt(application) if receipt(application) and receipt(application)['impacts'] else None)
    rows=json.loads(impacts['impacts']);affected=[]
    for node_id in (a_id,b_id):
        row=next(r for r in rows if r['source']==node_id)
        causes=[c for c in row['causes'] if c['input'].get('field_id')=='qualification.mara.tagline' and c['current_revision_event_id']==after_fact[1] and c['consumed_revision_event_id']==original_fact[1]]
        if not row['needsReview'] or not causes:raise RuntimeError('Actual consumer lacks accepted Bible revision review cause')
        affected.append(row)
    history=driver.canonical_state(database);calls=len(Provider.records)
    ui.click_button(application,'QA replay exact decision',window)
    replay=ui.wait_for('exact acceptance replay',lambda:receipt(application) if receipt(application) and 'already_recorded' in receipt(application)['replay'] else None)
    if history!=driver.canonical_state(database) or len(Provider.records)!=calls:raise RuntimeError('Acceptance replay repeated write/provider')
    evidence['explicit_acceptance']={'proposal_id':fresh[0],'before_fact':restored,'after_fact':after_fact,'affected_consumers':affected,'all_script_tables_preserved':True,'unselected_fields_preserved':True,'draft_preserved':DRAFT,'exact_decision_replay':replay}
    checkpoint('ordinary Accept updates only selected Bible field; both consumers Need review; no screenplay or placement rewrite');capture('manual-fact-07-accepted.png')
    disclosure(application,window,'What changed')
    evidence['visible_impact']=ui.wait_for('visible chosen Bible fact cause',lambda:driver.visible_review_label(application,window,'Bible fact profile.tagline changed.'))
    capture('manual-fact-08-needs-review.png')
    # Separate pending proposal after accepted fact; no further fact replacement.
    relationship_pending=analyze(application,window,database)
    ui.click_button(application,'QA relationship edit and restore',window)
    ui.wait_for('public relationship ABA completed',lambda:receipt(application) if receipt(application) and receipt(application)['edgeMutations']==2 else None)
    stale_accept(application,window,database,relationship_pending[0],evidence,'labelled public association edit/restore ABA')
    checkpoint('consumed association ABA refuses pending acceptance, accepted fact and all screenplay remain intact');capture('manual-fact-09-relationship-stale.png')
    reject(application,window,database,relationship_pending[0])
    if material(database)!=preserved or field(database)!=after_fact:raise RuntimeError('Final association refusal/rejection changed accepted fact/material')
    require_draft(application)
    evidence['final_bible_display']=accepted_bible_display(application,window)
    capture('manual-fact-09b-preserved-bible-drafts.png')


def qualify_save_refresh(application,window,database,capture,checkpoint,evidence):
    """Actual UI and public writes; explicitly labelled transport faults only."""
    preserved=material(database);provider_count=len(Provider.records)
    before=field(database)
    ui.click_button(application,'QA hold next Bible Save acknowledgement',window)
    editor=ui.wait_for('clean right Bible editor before pending Save',lambda:driver.find_bible_editor(application,window,BLUE,'right'))
    ui.type_text(editor,window,PENDING_DRAFT)
    save=ui.wait_for('verified ordinary right Bible Save',lambda:(control if control and control.getState().contains(ui.pyatspi.STATE_ENABLED) else None) if (control:=driver.save_fact_control(application,editor)) else None)
    ui.click_control(save,window)
    ui.wait_for('held actual Save acknowledgement',lambda:receipt(application) if receipt(application)['saveAcknowledgementHeld'] else None)
    acknowledged=ui.wait_for('actual public Save committed submitted trimmed text',lambda:field(database) if field(database)[0]==PENDING_DRAFT.strip() else None)
    ui.click_button(application,'QA write newer Green fact',window)
    newer=ui.wait_for('newer public fact committed',lambda:field(database) if field(database)[0]==NEWER_FACT else None)
    viewport=driver.bible_viewport(application,window,'right')
    def right_node(name,text=None):
        return ui.reveal(application,lambda n:(n.name==name or (text is None and ui.text_of(n)==name)) and (text is None or ui.text_of(n)==text)
            and viewport[0]<=n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0]<viewport[0]+viewport[2])
    ui.wait_for('newer committed field visibly refreshed while Save ack held',lambda:right_node('Committed Bible fact',NEWER_FACT))
    ui.click_button(application,'QA release Bible Save acknowledgement',window)
    ui.wait_for('Save acknowledgement released',lambda:not receipt(application)['saveAcknowledgementHeld'])
    evidence['pending_save_conflict']={
        'before_fact':before,'acknowledged_fact':acknowledged,'newer_fact':newer,
        'ordinary_submitted_draft':driver.bible_field(application,window,PENDING_DRAFT,'right',align=True),
        'source':'ordinary native field Save with labelled delayed acknowledgement; newer public canonical write',
        'exact_original_base':BLUE,
    }
    committed=ui.wait_for('newer committed fact after late acknowledgement',lambda:right_node('Committed Bible fact',NEWER_FACT))
    base=ui.wait_for('original draft base retained',lambda:ui.reveal(application,lambda n:ui.text_of(n)==BLUE and not n.getState().contains(ui.pyatspi.STATE_EDITABLE) and viewport[0]<=n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0]<viewport[0]+viewport[2]))
    save=ui.wait_for('right conflict Save control',lambda:driver.save_fact_control(application,committed))
    if save.getState().contains(ui.pyatspi.STATE_ENABLED):raise RuntimeError('Late acknowledgement left conflicting Save enabled')
    evidence['pending_save_conflict'].update(save_disabled=True,base_native_text=ui.text_of(base),committed_native_text=ui.text_of(committed))
    checkpoint('held Save acknowledgement cannot erase exact submitted draft or newer canonical conflict')
    capture('manual-fact-11-late-save-conflict.png')

    motivation=ui.wait_for('right unconflicted motivation editor',lambda:driver.find_bible_editor(application,window,MOTIVATION,'right'))
    ui.type_text(motivation,window,RECOVERY_DRAFT)
    ui.wait_for('unconflicted motivation Save enabled before induced failure',lambda:(control if control and control.getState().contains(ui.pyatspi.STATE_ENABLED) else None) if (control:=driver.save_fact_control(application,motivation)) else None)
    ui.click_button(application,'QA fail Bible detail refresh',window)
    ui.wait_for('labelled failure actually reached the production detail read seam',lambda:receipt(application)['detailFailures']>0)
    error=ui.wait_for('visible cached-detail failure',lambda:right_node('SYNTHETIC QA transport: Bible detail unavailable.'))
    motivation=ui.wait_for('exact recovery draft during failed refresh',lambda:driver.find_bible_editor(application,window,RECOVERY_DRAFT,'right'))
    save=ui.wait_for('unconflicted field Save during failed refresh',lambda:driver.save_fact_control(application,motivation))
    if save.getState().contains(ui.pyatspi.STATE_ENABLED):raise RuntimeError('Failed cached refresh permits stale field Save')
    evidence['detail_failure']={'visible_error':ui.text_of(error),'cached_fact':field(database),
        'exact_draft':driver.bible_field(application,window,RECOVERY_DRAFT,'right',align=True),
        'unconflicted_save_enabled_before_failure':True,'unconflicted_save_disabled':True,'synthetic_transport_failure':True}
    checkpoint('cached details expose refresh failure and disable unconflicted field Save')
    capture('manual-fact-12-detail-error.png')

    ui.click_button(application,'QA hold Bible detail recovery',window)
    retry=ui.wait_for('ordinary right Retry saved facts',lambda:right_node('Retry saved facts'))
    ui.click_control(retry,window)
    ui.wait_for('actual ordinary retry read held',lambda:receipt(application)['detailReadsHeld']>0)
    motivation=ui.wait_for('exact draft during pending retry',lambda:driver.find_bible_editor(application,window,RECOVERY_DRAFT,'right'))
    save=ui.wait_for('field Save during pending verification',lambda:driver.save_fact_control(application,motivation))
    if save.getState().contains(ui.pyatspi.STATE_ENABLED):raise RuntimeError('Pending retry permits stale Save after error clearing')
    evidence['pending_detail_retry']={'ordinary_retry':True,'save_disabled_after_error_cleared':True,
        'exact_draft':driver.bible_field(application,window,RECOVERY_DRAFT,'right',align=True)}
    checkpoint('ordinary pending retry keeps Save disabled after error is cleared')
    capture('manual-fact-13-detail-retry-pending.png')

    ui.click_button(application,'QA release Bible detail recovery',window)
    motivation=ui.wait_for('exact recovery draft after admitted read',lambda:driver.find_bible_editor(application,window,RECOVERY_DRAFT,'right'))
    save=ui.wait_for('unconflicted Save enabled only after verified recovery',lambda:(control if control and control.getState().contains(ui.pyatspi.STATE_ENABLED) else None) if (control:=driver.save_fact_control(application,motivation)) else None)
    evidence['verified_detail_recovery']={'exact_draft':driver.bible_field(application,window,RECOVERY_DRAFT,'right',align=True),
        'unconflicted_save_enabled':True,'draft_not_submitted':True,'unchanged_motivation':ui.query(database,"SELECT text_value FROM bible_graph_fields WHERE id='qualification.mara.motivation'")[0][0]}
    if evidence['verified_detail_recovery']['unchanged_motivation']!=MOTIVATION or field(database)!=newer or material(database)!=preserved or len(Provider.records)!=provider_count:raise RuntimeError('Fault/recovery sequence changed unrelated canonical material or called provider')
    require_draft(application)
    evidence['verified_detail_recovery']['saved_script_timeline_and_provider_unchanged']=True
    checkpoint('verified read restores unconflicted Save without discarding exact drafts or rewriting screenplay')
    capture('manual-fact-14-detail-recovered.png')


def qualify_initial_detail_retry(application,window,database,capture,checkpoint,evidence):
    """Uncached ordinary selection/Retry; only read-fault transport is synthetic."""
    before={'material':material(database),'fields':bible_fields(database),'history':driver.canonical_state(database)}
    ui.click_button(application,'QA arm initial Bible detail failure',window)
    ui.click_button(application,'Bible',window)
    search=ui.wait_for('Bible entity search',lambda:ui.find(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and 0<=n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0]<400 and ui.text_of(n)==''))
    ui.type_text(search,window,'Mara')
    ui.click_control(ui.wait_for('Mara entity',lambda:ui.reveal(application,lambda n:n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and ui.button_label_matches(n.name,'Mara',prefix=True))),window)
    def left_node(text):
        viewport=driver.bible_viewport(application,window,'left')
        return ui.reveal(application,lambda n:(ui.text_of(n)==text or n.name==text) and driver.contained(tuple(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)),viewport))
    error=ui.wait_for('visible uncached initial detail error',lambda:left_node('SYNTHETIC QA transport: Bible detail unavailable.'))
    retry=ui.wait_for('enabled ordinary uncached Retry',lambda:left_node('Retry saved facts'))
    if not retry.getState().contains(ui.pyatspi.STATE_ENABLED):raise RuntimeError('Initial detail Retry disabled')
    if driver.find_bible_editor(application,window,RED,'left'):raise RuntimeError('Initial fault unexpectedly retained cached fields')
    evidence['initial_detail_error']={'ordinary_entity_selection':True,'no_cached_field_editor':True,'error':ui.text_of(error),'retry_enabled':True,'failures':receipt(application)['detailFailures']}
    checkpoint('uncached initial read failure exposes ordinary Retry in place')
    capture('manual-fact-15-initial-detail-error.png')
    ui.click_button(application,'QA hold Bible detail recovery',window)
    ui.click_control(retry,window)
    ui.wait_for('ordinary initial retry held',lambda:receipt(application)['detailReadsHeld']>0)
    ui.wait_for('uncached initial retry Loading',lambda:left_node('Loading'))
    if left_node('SYNTHETIC QA transport: Bible detail unavailable.') or left_node('Retry saved facts'):raise RuntimeError('Pending initial retry kept stale error or duplicate retry')
    evidence['initial_detail_pending']={'ordinary_retry':True,'loading_visible':True,'error_cleared':True,'held_reads':receipt(application)['detailReadsHeld']}
    checkpoint('same inspector ordinary retry pending; Loading and no editable fields')
    capture('manual-fact-16-initial-detail-pending.png')
    ui.click_button(application,'QA release Bible detail recovery',window)
    field_view=driver.bible_field(application,window,RED,'left',align=True)
    if left_node('SYNTHETIC QA transport: Bible detail unavailable.'):raise RuntimeError('Recovered initial detail still displays error')
    evidence['initial_detail_recovered']={'same_inspector':True,'actual_public_read':True,'canonical_editor':field_view}
    checkpoint('same inspector verified initial detail recovery displays actual saved fact')
    capture('manual-fact-17-initial-detail-recovered.png')
    editor=ui.wait_for('recovered motivation editor',lambda:driver.find_bible_editor(application,window,MOTIVATION,'left'))
    ui.type_text(editor,window,RECOVERY_DRAFT)
    draft=driver.bible_field(application,window,RECOVERY_DRAFT,'left',align=True)
    editor=ui.wait_for('exact unsubmitted draft after initial recovery',lambda:driver.find_bible_editor(application,window,RECOVERY_DRAFT,'left'))
    save=ui.wait_for('verified recovered field Save',lambda:driver.save_fact_control(application,editor))
    if not save.getState().contains(ui.pyatspi.STATE_ENABLED):raise RuntimeError('Verified initial recovery did not enable field Save')
    if before!={'material':material(database),'fields':bible_fields(database),'history':driver.canonical_state(database)}:raise RuntimeError('Read-only initial failure/retry or unsubmitted draft changed canonical material/history')
    if len(Provider.records)!=4:raise RuntimeError('Initial read retry invoked provider')
    evidence['initial_detail_draft']={'exact_unsubmitted_draft':draft,'save_enabled_after_verification':True,'saved_material_facts_history_unchanged':True,'provider_calls_unchanged':True}
    checkpoint('verified recovery permits exact manual draft; no Save submitted or canonical changes')
    capture('manual-fact-18-initial-detail-draft.png')


def qualify_timeline_notes(application,window,database,fixture,capture,checkpoint,evidence):
    b_id=fixture['b']['id'];a_id=fixture['a']['id'];f_id=fixture['f']['id']
    original={node:ui.blocks(database,node) for node in (a_id,b_id,f_id)}
    original_bible=bible_fields(database)
    old=json.loads(ui.query(database,'SELECT content_json FROM nodes WHERE id=?',(b_id,))[0][0])['notes']
    block=ui.wait_for('unrelated F block',lambda:ui.screenplay_block(application,driver.screenplay_anchor(F_TEXT)))
    ui.reveal_button(block,'Edit',window)
    ui.type_text(ui.wait_for('unrelated editor',lambda:ui.editable(application,F_TEXT)),window,DRAFT)
    ui.click_button(application,'Bible',window)
    search=ui.wait_for('Bible search',lambda:ui.find(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and 0<=n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0]<400 and ui.text_of(n)==''))
    ui.type_text(search,window,'Mara')
    ui.click_control(ui.wait_for('Mara entity',lambda:ui.reveal(application,lambda n:n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and ui.button_label_matches(n.name,'Mara',prefix=True))),window)
    driver.bible_field(application,window,RED,'left',align=True)
    clip=ui.wait_for('ordinary B timeline clip',lambda:ui.native_timeline_clip(application,fixture['b']['name'],window))
    ui.click_control(clip[0],window,clip[1])
    field_node=ui.wait_for('selected clip ordinary Notes editor',lambda:ui.reveal(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n)==old))
    ui.type_text(field_node,window,NOTES)
    def saved_notes():
        actual=json.loads(ui.query(database,'SELECT content_json FROM nodes WHERE id=?',(b_id,))[0][0])['notes']
        return actual if actual==NOTES else None
    ui.wait_for('exact public debounced Notes commit',saved_notes)
    for node,rows in original.items():
        if ui.blocks(database,node)!=rows:raise RuntimeError('Notes edit changed saved screenplay/placement')
    if bible_fields(database)!=original_bible:raise RuntimeError('Notes edit changed Bible facts')
    require_draft(application)
    evidence['ordinary_notes_edit']={'node_id':b_id,'original':old,'exact_saved_notes':NOTES,'input_route':'ordinary Notes textarea/native keys/public debounced command','saved_screenplay_and_placement_unchanged':True,'bible_unchanged':True,'unrelated_draft':DRAFT}
    ui.click_button(application,'QA read canonical impacts',window)
    def impacted():
        value=receipt(application)
        if not value or not value['impacts']:return None
        rows=json.loads(value['impacts']);target=next((r for r in rows if r['source']==b_id),None)
        if target and any(c['dependency_id'].endswith('.timeline_notes') for c in target.get('causes') or []):return rows
    impacts=ui.wait_for('canonical selected Notes review cause',impacted)
    if any(any(c['dependency_id'].endswith('.timeline_notes') for c in r.get('causes') or []) for r in impacts if r['source']!=b_id):raise RuntimeError('Unrelated scene acquired Notes cause')
    evidence['notes_impacts_before_acceptance']=impacts
    # Both generated scenes already have independent review causes. Their
    # ordinary cause disclosures are closed initially; open them natively
    # before asking for a visibly rendered Notes label. A multicolumn break
    # can place the selected scene's notice and chooser in different columns.
    notices=[n for n in ui.walk(application) if n.name=='Screenplay needs review']
    if len(notices)!=2:raise RuntimeError('Expected exactly the two generated scene notices')
    for node in notices:disclosure(node,window,'What changed')
    notice=ui.wait_for('ordinary Notes review notice visible',lambda:driver.visible_review_label(application,window,'Timeline Notes changed.'))
    evidence['visible_notes_impact']=notice
    source_review=ui.wait_for('ordinary B source review',lambda:review_for_source(application,fixture['b']['name']))
    combo=ui.wait_for('Notes source review chooser',lambda:ui.reveal(source_review,lambda n:
        n.getRole()==ui.pyatspi.ROLE_COMBO_BOX and ui.button_label_matches(n.name,'Input change',prefix=True)))
    ui.click_control(combo,window);ui.command('xdotool','key','--clearmodifiers','End','Return')
    review=ui.wait_for('ordinary explicitly selected Notes cause',lambda:notes_review(application))
    evidence['ordinary_selected_notes_cause']={'native_combo_name':combo.name,'route':'native source chooser End/Return; canonical pending binding checked'}
    checkpoint('ordinary Notes edit preserved saved material and exposed selected review cause');capture('timeline-notes-01-needs-review.png')
    count=len(Provider.records)
    ui.reveal_button(review,'Preview update',window)
    def pending():
        if len(Provider.records)>count and not Provider.records[-1]['accepted']:raise RuntimeError('Synthetic Notes provider refused prompt')
        rows=ui.query(database,"SELECT p.id,p.proposed_text,b.binding_json FROM propagation_proposals p JOIN script_impact_proposal_bindings b ON b.proposal_id=p.id WHERE p.status='pending' ORDER BY p.rowid DESC")
        return rows[0] if rows else None
    proposed=ui.wait_for('canonical pending Notes preview',pending)
    binding=json.loads(proposed[2])
    if proposed[1]!=NOTES_PREVIEW or binding['timeline_notes_previous']['notes']!=old or binding['timeline_notes_current']['notes']!=NOTES or not binding['request']['dependency_id'].endswith('.timeline_notes'):raise RuntimeError('Notes preview receipt/text/cause differs')
    if {node:ui.blocks(database,node) for node in original}!=original:raise RuntimeError('Preview replaced saved material')
    ui.wait_for('reachable exact current Notes evidence',lambda:ui.reveal(application,lambda n:n.name=='Current preview Notes' and ui.text_of(n)==NOTES))
    ui.wait_for('exact current Notes glyphs in Script pane',lambda:driver.visible_paragraph(application,window,NOTES,'Current preview Notes'))
    evidence['pending_notes_preview']={'id':proposed[0],'binding':binding,'proposed_text':proposed[1],'saved_material_unchanged':True}
    require_draft(application)
    checkpoint('synthetic targeted preview retains exact original/current Notes and requires acceptance');capture('timeline-notes-02-pending.png')
    # An ordinary second edit/restore retains identical text while advancing its clock.
    for text in ['Intervening timeline Notes decision',NOTES]:
        before_text=json.loads(ui.query(database,'SELECT content_json FROM nodes WHERE id=?',(b_id,))[0][0])['notes']
        field_node=ui.wait_for('ordinary Notes editor before ABA',lambda:ui.reveal(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n)==before_text))
        ui.type_text(field_node,window,text)
        ui.wait_for('public Notes ABA write',lambda:json.loads(ui.query(database,'SELECT content_json FROM nodes WHERE id=?',(b_id,))[0][0])['notes']==text)
    before=driver.canonical_state(database)
    ui.reveal_button(ui.wait_for('selected Notes review before stale accept',lambda:notes_review(application)),'Accept update',window)
    error=ui.wait_for('ordinary stale Notes acceptance refusal',lambda:ui.find(application,lambda n:'screenplay proposal is stale' in ui.text_of(n)))
    if driver.canonical_state(database)!=before:raise RuntimeError('Stale Notes refusal changed canonical state')
    evidence['notes_aba_refusal']={'exact_restored_notes':NOTES,'visible_error':ui.text_of(error),'all_recorded_tables_unchanged':True}
    require_draft(application);checkpoint('ordinary Notes ABA refuses old acceptance without saved writes');capture('timeline-notes-03-stale.png')
    ui.reveal_button(ui.wait_for('selected Notes review before reject',lambda:notes_review(application)),'Reject',window)
    ui.wait_for('ordinary reject stale Notes preview',lambda:ui.query(database,"SELECT id FROM propagation_proposals WHERE id=? AND status='rejected'",(proposed[0],)))
    # Allow one explicitly requested fresh synthetic response after the first is rejected.
    Provider.allow_fresh_notes=True
    ui.reveal_button(ui.wait_for('selected Notes review before fresh preview',lambda:notes_review(application)),'Preview update',window)
    fresh=ui.wait_for('fresh canonical pending Notes preview',pending)
    if fresh[0]==proposed[0]:raise RuntimeError('Fresh review reused rejected proposal')
    ui.reveal_button(ui.wait_for('selected Notes review before fresh accept',lambda:notes_review(application)),'Accept update',window)
    accepted=ui.wait_for('explicit targeted Notes update',lambda:next((r for r in ui.blocks(database,b_id) if r[1]==NOTES_PREVIEW and r[2]!=original[b_id][0][2]),None))
    for node in (a_id,f_id):
        if ui.blocks(database,node)!=original[node]:raise RuntimeError('Acceptance changed unrelated saved material')
    if accepted[3:]!=original[b_id][0][3:] or bible_fields(database)!=original_bible:raise RuntimeError('Acceptance changed placement or Bible')
    ui.click_button(application,'QA read canonical impacts',window)
    def clean():
        value=receipt(application)
        if not value or not value['impacts']:return None
        rows=json.loads(value['impacts']);target=next((r for r in rows if r['source']==b_id),None)
        return rows if target and not any(c['dependency_id'].endswith('.timeline_notes') for c in target.get('causes') or []) else None
    evidence['notes_impacts_after_acceptance']=ui.wait_for('fresh Notes consumption clears its cause',clean)
    evidence['explicit_notes_acceptance']={'proposal_id':fresh[0],'saved_selected_block':accepted,'unrelated_saved_material_preserved':True,'placement_and_bible_preserved':True}
    require_draft(application)
    ui.wait_for('visible saved synthetic Notes update',lambda:ui.screenplay_block(application,driver.screenplay_anchor(NOTES_PREVIEW)))
    if len(Provider.records)!=6 or not all(r['accepted'] for r in Provider.records):raise RuntimeError('Expected four fixture and two targeted synthetic provider calls')
    checkpoint('explicit fresh acceptance updates only chosen screenplay and refreshes Notes lineage');capture('timeline-notes-04-accepted.png')

def verify_ancestor_fixture_hierarchy(database,fixture):
    def row(node):
        rows=ui.query(database,'SELECT parent_id,level,content_json FROM nodes WHERE id=?',(node,))
        if len(rows)!=1:raise RuntimeError('Missing actual public fixture node: '+node)
        return rows[0]
    f_parent,f_level,_=row(fixture['f']['id'])
    seq_parent,seq_level,_=row(f_parent)
    _,act_level,_=row(seq_parent)
    if (f_level,seq_level,act_level)!=('Scene','Sequence','Act') or f_parent!=fixture['unrelated_sequence']['id'] or seq_parent!=fixture['unrelated_act']['id']:
        raise RuntimeError('Unrelated F does not use the required public Act/Sequence/Scene hierarchy')
    ancestor_id=fixture['ancestor']['id']
    for source in ('a','b'):
        parent,level,_=row(fixture[source]['id'])
        act,seq_level,_=row(parent)
        if (level,seq_level)!=('Scene','Sequence') or act!=ancestor_id:raise RuntimeError('Consumer is outside the declared ancestor hierarchy')
    _,level,content=row(ancestor_id)
    if level!='Act' or json.loads(content)['notes']!=ANCESTOR_OLD:raise RuntimeError('Actual ancestor Notes differ')
    revisions=ui.query(database,"SELECT r.change_event_id FROM object_revisions r JOIN object_revision_fields f ON f.revision_id=r.id WHERE r.object_kind='timeline_node' AND r.object_id=? AND f.field_key='notes' AND f.new_type='text' AND f.new_text=?",(ancestor_id,ANCESTOR_OLD))
    if not revisions:raise RuntimeError('Fixture ancestor Notes lack owned public field history')
    return {'public_hierarchy_verified':True,'unrelated_sequence':f_parent,'unrelated_act':seq_parent,'consumed_act':ancestor_id,'owned_notes_revision':revisions[-1][0]}


def qualify_ancestor_notes(application,window,database,fixture,capture,checkpoint,evidence):
    b_id=fixture['b']['id'];a_id=fixture['a']['id'];f_id=fixture['f']['id'];ancestor_id=fixture['ancestor']['id']
    save_script(application,window,database,b_id,GENERATED_B,ANCESTOR_MANUAL)
    original={node:ui.blocks(database,node) for node in (a_id,b_id,f_id)}
    original_bible=bible_fields(database)
    old=json.loads(ui.query(database,'SELECT content_json FROM nodes WHERE id=?',(ancestor_id,))[0][0])['notes']
    if old!=ANCESTOR_OLD:raise RuntimeError('Public fixture ancestor Notes differs')
    original_material=material(database)
    block=ui.wait_for('unrelated F block',lambda:ui.screenplay_block(application,driver.screenplay_anchor(F_TEXT)))
    ui.reveal_button(block,'Edit',window)
    ui.type_text(ui.wait_for('unrelated editor',lambda:ui.editable(application,F_TEXT)),window,DRAFT)
    ui.click_button(application,'Bible',window)
    search=ui.wait_for('Bible search',lambda:ui.find(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and 0<=n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0]<400 and ui.text_of(n)==''))
    ui.type_text(search,window,'Mara')
    ui.click_control(ui.wait_for('Mara entity',lambda:ui.reveal(application,lambda n:n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and ui.button_label_matches(n.name,'Mara',prefix=True))),window)
    driver.bible_field(application,window,RED,'left',align=True)
    unrelated_id=fixture['unrelated_act']['id']
    clip=ui.wait_for('ordinary unrelated Act timeline clip',lambda:ui.native_timeline_clip(application,fixture['unrelated_act']['name'],window))
    ui.click_control(clip[0],window,clip[1])
    editor=ui.wait_for('ordinary unrelated Act Notes editor',lambda:ui.reveal(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n)==UNRELATED_ACT_OLD))
    unrelated_new='Unrelated Act: Eli keeps his brass whistle.\n\n'
    ui.type_text(editor,window,unrelated_new)
    ui.wait_for('exact unrelated Act public Notes write',lambda:json.loads(ui.query(database,'SELECT content_json FROM nodes WHERE id=?',(unrelated_id,))[0][0])['notes']==unrelated_new)
    ui.click_button(application,'QA read canonical impacts',window)
    def no_unrelated_impacts():
        value=receipt(application)
        if not value or not value['impacts']:return None
        rows=json.loads(value['impacts'])
        if any('.ancestor_notes.' in c['dependency_id'] for row in rows for c in row.get('causes') or []):raise RuntimeError('Unconsumed Act Notes created a consumer cause')
        return rows
    control_impacts=ui.wait_for('unrelated Act edit has no ancestor consumer cause',no_unrelated_impacts)
    if material(database)!=original_material or bible_fields(database)!=original_bible:raise RuntimeError('Unrelated Notes edit changed saved material')
    require_draft(application)
    evidence['ordinary_unrelated_act_control']={'node_id':unrelated_id,'exact_notes':unrelated_new,'impacts':control_impacts,'saved_material_and_unrelated_draft_preserved':True}
    checkpoint('ordinary unrelated Act Notes edit preserves consumers and saved material');capture('ancestor-notes-00-unrelated.png')
    clip=ui.wait_for('ordinary consumed Act timeline clip',lambda:ui.native_timeline_clip(application,fixture['ancestor']['name'],window))
    ui.click_control(clip[0],window,clip[1])
    field_node=ui.wait_for('selected clip ordinary Notes editor',lambda:ui.reveal(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n)==old))
    ui.type_text(field_node,window,NOTES)
    def saved_notes():
        actual=json.loads(ui.query(database,'SELECT content_json FROM nodes WHERE id=?',(ancestor_id,))[0][0])['notes']
        return actual if actual==NOTES else None
    ui.wait_for('exact public debounced Notes commit',saved_notes)
    for node,rows in original.items():
        if ui.blocks(database,node)!=rows:raise RuntimeError('Notes edit changed saved screenplay/placement')
    if bible_fields(database)!=original_bible:raise RuntimeError('Notes edit changed Bible facts')
    if material(database)!=original_material:raise RuntimeError('Ancestor edit changed saved blocks/spans/locks/placement')
    require_draft(application)
    evidence['ordinary_notes_edit']={'node_id':ancestor_id,'original':old,'exact_saved_notes':NOTES,'input_route':'ordinary Notes textarea/native keys/public debounced command','saved_screenplay_and_placement_unchanged':True,'bible_unchanged':True,'unrelated_draft':DRAFT}
    ui.click_button(application,'QA read canonical impacts',window)
    def impacted():
        value=receipt(application)
        if not value or not value['impacts']:return None
        rows=json.loads(value['impacts']);target=next((r for r in rows if r['source']==b_id),None)
        if target and any(c['dependency_id'].endswith('.ancestor_notes.'+ancestor_id) for c in target.get('causes') or []):return rows
    impacts=ui.wait_for('canonical selected Notes review cause',impacted)
    for source in (a_id,b_id):
        row=next((r for r in impacts if r['source']==source),None)
        if not row or not any(c['dependency_id'].endswith('.ancestor_notes.'+ancestor_id) and c['input']['node_id']==ancestor_id for c in row.get('causes') or []):raise RuntimeError('Actual ancestor consumer lacks exact owned Notes cause')
    if any(any(c['dependency_id'].endswith('.ancestor_notes.'+ancestor_id) for c in r.get('causes') or []) for r in impacts if r['source'] not in (a_id,b_id)):raise RuntimeError('Unrelated Act material acquired ancestor Notes cause')
    evidence['notes_impacts_before_acceptance']=impacts
    # Both generated scenes already have independent review causes. Their
    # ordinary cause disclosures are closed initially; open them natively
    # before asking for a visibly rendered Notes label. A multicolumn break
    # can place the selected scene's notice and chooser in different columns.
    notices=[n for n in ui.walk(application) if n.name=='Screenplay needs review']
    if len(notices)!=2:raise RuntimeError('Expected exactly the two generated scene notices')
    for node in notices:disclosure(node,window,'What changed')
    notice=ui.wait_for('ordinary Notes review notice visible',lambda:driver.visible_review_label(application,window,'Ancestor timeline Notes changed.'))
    evidence['visible_notes_impact']=notice
    source_review=ui.wait_for('ordinary B source review',lambda:review_for_source(application,fixture['b']['name']))
    combo=ui.wait_for('Notes source review chooser',lambda:ui.reveal(source_review,lambda n:
        n.getRole()==ui.pyatspi.ROLE_COMBO_BOX and ui.button_label_matches(n.name,'Input change',prefix=True)))
    ui.click_control(combo,window);ui.command('xdotool','key','--clearmodifiers','End','Return')
    review=ui.wait_for('ordinary explicitly selected Notes cause',lambda:review_for_source(application,fixture['b']['name']))
    evidence['ordinary_selected_notes_cause']={'native_combo_name':combo.name,'route':'native source chooser End/Return; canonical pending binding checked'}
    checkpoint('ordinary Notes edit preserved saved material and exposed selected review cause');capture('ancestor-notes-01-needs-review.png')
    count=len(Provider.records)
    ui.reveal_button(review,'Preview update',window)
    def pending():
        if len(Provider.records)>count and not Provider.records[-1]['accepted']:raise RuntimeError('Synthetic Notes provider refused prompt')
        rows=ui.query(database,"SELECT p.id,p.proposed_text,b.binding_json FROM propagation_proposals p JOIN script_impact_proposal_bindings b ON b.proposal_id=p.id WHERE p.status='pending' ORDER BY p.rowid DESC")
        return rows[0] if rows else None
    proposed=ui.wait_for('canonical pending Notes preview',pending)
    binding=json.loads(proposed[2])
    if proposed[1]!=NOTES_PREVIEW or binding['ancestor_notes_previous']!=[{'node_id':ancestor_id,'notes':old,'revision_event_id':binding['cause']['consumed_revision_event_id']}] or binding['ancestor_notes_current'][0]['notes']!=NOTES or not binding['request']['dependency_id'].endswith('.ancestor_notes.'+ancestor_id):raise RuntimeError('Notes preview receipt/text/cause differs')
    if {node:ui.blocks(database,node) for node in original}!=original or material(database)!=original_material:raise RuntimeError('Preview replaced exact saved manual material')
    ui.wait_for('reachable exact current Notes evidence',lambda:ui.reveal(application,lambda n:n.name=='Current preview Notes' and ui.text_of(n)==NOTES))
    ui.wait_for('exact current Notes glyphs in Script pane',lambda:driver.visible_paragraph(application,window,NOTES,'Current preview Notes'))
    evidence['ancestor_original_visible']=ui.wait_for('exact original ancestor Notes glyphs',lambda:driver.visible_paragraph(application,window,old,'Originally consumed Notes'))
    evidence['ancestor_current_visible']=ui.wait_for('exact current ancestor Notes glyphs',lambda:driver.visible_paragraph(application,window,NOTES,'Current preview Notes'))
    evidence['pending_notes_preview']={'id':proposed[0],'binding':binding,'proposed_text':proposed[1],'saved_material_unchanged':True}
    require_draft(application)
    checkpoint('synthetic targeted preview retains exact original/current Notes and requires acceptance');capture('ancestor-notes-02-pending.png')
    # An ordinary second edit/restore retains identical text while advancing its clock.
    for text in ['Intervening timeline Notes decision',NOTES]:
        before_text=json.loads(ui.query(database,'SELECT content_json FROM nodes WHERE id=?',(ancestor_id,))[0][0])['notes']
        field_node=ui.wait_for('ordinary Notes editor before ABA',lambda:ui.reveal(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n)==before_text))
        ui.type_text(field_node,window,text)
        ui.wait_for('public Notes ABA write',lambda:json.loads(ui.query(database,'SELECT content_json FROM nodes WHERE id=?',(ancestor_id,))[0][0])['notes']==text)
    before=driver.canonical_state(database)
    ui.reveal_button(ui.wait_for('selected Notes review before stale accept',lambda:review_for_source(application,fixture['b']['name'])),'Accept update',window)
    error=ui.wait_for('ordinary stale Notes acceptance refusal',lambda:ui.find(application,lambda n:'screenplay proposal is stale' in ui.text_of(n)))
    if driver.canonical_state(database)!=before:raise RuntimeError('Stale Notes refusal changed canonical state')
    evidence['notes_aba_refusal']={'exact_restored_notes':NOTES,'visible_error':ui.text_of(error),'all_recorded_tables_unchanged':True}
    require_draft(application);checkpoint('ordinary Notes ABA refuses old acceptance without saved writes');capture('ancestor-notes-03-stale.png')
    ui.reveal_button(ui.wait_for('selected Notes review before reject',lambda:review_for_source(application,fixture['b']['name'])),'Reject',window)
    ui.wait_for('ordinary reject stale Notes preview',lambda:ui.query(database,"SELECT id FROM propagation_proposals WHERE id=? AND status='rejected'",(proposed[0],)))
    # Allow one explicitly requested fresh synthetic response after the first is rejected.
    Provider.allow_fresh_notes=True
    ui.reveal_button(ui.wait_for('selected Notes review before fresh preview',lambda:review_for_source(application,fixture['b']['name'])),'Preview update',window)
    fresh=ui.wait_for('fresh canonical pending Notes preview',pending)
    if fresh[0]==proposed[0]:raise RuntimeError('Fresh review reused rejected proposal')
    ui.reveal_button(ui.wait_for('selected Notes review before fresh accept',lambda:review_for_source(application,fixture['b']['name'])),'Accept update',window)
    accepted=ui.wait_for('explicit targeted Notes update',lambda:next((r for r in ui.blocks(database,b_id) if r[1]==NOTES_PREVIEW and r[2]!=original[b_id][0][2]),None))
    for node in (a_id,f_id):
        if ui.blocks(database,node)!=original[node]:raise RuntimeError('Acceptance changed unrelated saved material')
    if accepted[3:]!=original[b_id][0][3:] or bible_fields(database)!=original_bible:raise RuntimeError('Acceptance changed placement or Bible')
    ui.click_button(application,'QA read canonical impacts',window)
    def clean():
        value=receipt(application)
        if not value or not value['impacts']:return None
        rows=json.loads(value['impacts']);target=next((r for r in rows if r['source']==b_id),None)
        return rows if target and not any(c['dependency_id'].endswith('.ancestor_notes.'+ancestor_id) for c in target.get('causes') or []) else None
    evidence['notes_impacts_after_acceptance']=ui.wait_for('fresh Notes consumption clears its cause',clean)
    other=next(r for r in evidence['notes_impacts_after_acceptance'] if r['source']==a_id)
    if not any(c['dependency_id'].endswith('.ancestor_notes.'+ancestor_id) for c in other.get('causes') or []):raise RuntimeError('Chosen-block acceptance cleared an unaccepted consumer')
    evidence['explicit_notes_acceptance']={'proposal_id':fresh[0],'saved_selected_block':accepted,'unrelated_saved_material_preserved':True,'placement_and_bible_preserved':True}
    require_draft(application)
    ui.wait_for('visible saved synthetic Notes update',lambda:ui.screenplay_block(application,driver.screenplay_anchor(NOTES_PREVIEW)))
    if len(Provider.records)!=6 or not all(r['accepted'] for r in Provider.records):raise RuntimeError('Expected four fixture and two targeted synthetic provider calls')
    checkpoint('explicit fresh acceptance updates only chosen screenplay and refreshes Notes lineage');capture('ancestor-notes-04-accepted.png')


def verify_arc_fixture(database,fixture):
    arc=fixture['arc']['id'];other=fixture['unrelated_arc']['id']
    for key,expected in [('a',arc),('b',arc),('f',other)]:
        tags=ui.query(database,'SELECT arc_id FROM node_arcs WHERE node_id=?',(fixture[key]['id'],))
        if tags!=[(expected,)]:raise RuntimeError('Fixture does not retain the actual declared template tag')
    if ui.query(database,'SELECT description FROM arcs WHERE id=?',(arc,))!=[('',)]:raise RuntimeError('Fixture description is not empty')
    revisions=ui.query(database,"SELECT r.change_event_id FROM object_revisions r JOIN object_revision_fields f ON f.revision_id=r.id WHERE r.object_kind='story_arc' AND r.object_id=? AND f.field_key='description' AND f.new_type='text' AND f.new_text='' ORDER BY r.rowid",(arc,))
    if not revisions:raise RuntimeError('Fixture empty Description lacks public owned history')
    if fixture.get('setup_only'):return {'template_tags_verified':True,'owned_empty_revision':revisions[-1][0],'provider_calls':0}
    receipts=[]
    for key in ('a','b'):
        rows=ui.query(database,"SELECT c.payload_json,e.id FROM commands c JOIN change_events e ON e.command_id=c.id WHERE c.payload_type='script.generate_block' ORDER BY e.rowid")
        command,event=next((json.loads(raw),event) for raw,event in rows if json.loads(raw)['block']['source_node_id']==fixture[key]['id'])
        expected={'arc_id':arc,'field':'description','value':'','revision_event_id':revisions[-1][0]}
        if command.get('arc_description_applicability')!=[expected] or any(i['field']=='description' for i in command['arc_inputs']):raise RuntimeError('Generation omitted description is not an honest applicability read')
        dep='generation.'+event+'.arc_description_applicability.'+arc
        if ui.query(database,'SELECT count(*) FROM semantic_dependencies WHERE id=?',(dep,))!=[(1,)]:raise RuntimeError('Applicability receipt lacks actual graph dependency')
        receipts.append({'source':fixture[key]['id'],'generation_event':event,'applicability':expected,'dependency_id':dep})
    return {'template_tags_verified':True,'owned_empty_revision':revisions[-1][0],'generation_receipts':receipts}


def arc_editor(application,window,old):
    return ui.wait_for('ordinary left ArcDetail Description textarea',lambda:ui.reveal(application,lambda n:
        n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n)==old
        and 0<=n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0]<400
        and (n.name=='Description' or n.getRole() in (ui.pyatspi.ROLE_TEXT,ui.pyatspi.ROLE_ENTRY))))


def open_arc(application,window,name):
    ui.click_button(application,'Arcs',window)
    node=ui.wait_for('ordinary named ArcList button',lambda:ui.reveal(application,lambda n:
        n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and ui.button_label_matches(n.name,name,prefix=True)))
    ui.click_control(node,window)


def qualify_arc_description(application,window,database,fixture,capture,checkpoint,evidence):
    a_id=fixture['a']['id'];b_id=fixture['b']['id'];f_id=fixture['f']['id'];arc=fixture['arc']['id']
    suffix='.arc_description_applicability.'+arc
    save_script(application,window,database,b_id,GENERATED_B,ANCESTOR_MANUAL)
    original={node:ui.blocks(database,node) for node in (a_id,b_id,f_id)}
    original_material=material(database);original_bible=bible_fields(database)
    block=ui.wait_for('unrelated saved F block',lambda:ui.screenplay_block(application,driver.screenplay_anchor(F_TEXT)))
    ui.reveal_button(block,'Edit',window)
    ui.type_text(ui.wait_for('unrelated screenplay editor',lambda:ui.editable(application,F_TEXT)),window,DRAFT)
    # Ordinary Bible entity selection leaves its real right inspector visible
    # while the left sidebar switches to Arcs. No layout/source injection.
    ui.click_button(application,'Bible',window)
    search=ui.wait_for('Bible search',lambda:ui.find(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and 0<=n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0]<400 and ui.text_of(n)==''))
    ui.type_text(search,window,'Mara')
    ui.click_control(ui.wait_for('Mara entity',lambda:ui.reveal(application,lambda n:n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and ui.button_label_matches(n.name,'Mara',prefix=True))),window)
    driver.bible_field(application,window,RED,'right',align=True)
    def preserved():
        if material(database)!=original_material or bible_fields(database)!=original_bible:raise RuntimeError('Arc action changed saved screenplay, spans, locks, placement or Bible')
        require_draft(application)
    def impacts():
        previous=receipt(application)['impactReads']
        ui.click_button(application,'QA read canonical impacts',window)
        return ui.wait_for('actual public canonical impacts',lambda:(json.loads(v['impacts']) if not v['busy'] and v['impactReads']>previous and v['impacts'] else None) if (v:=receipt(application)) else None)
    def scoped(rows):return [c for row in rows for c in row.get('causes') or [] if c['dependency_id'].endswith(suffix)]
    open_arc(application,window,fixture['unrelated_arc']['name'])
    ui.type_text(arc_editor(application,window,''),window,ARC_UNRELATED)
    ui.wait_for('exact unrelated public Description write',lambda:ui.query(database,'SELECT description FROM arcs WHERE id=?',(fixture['unrelated_arc']['id'],))==[(ARC_UNRELATED,)])
    control=impacts()
    if scoped(control):raise RuntimeError('Unrelated arc fill created target applicability cause')
    preserved();driver.bible_field(application,window,RED,'right',align=True)
    evidence['ordinary_unrelated_arc_control']={'exact_description':ARC_UNRELATED,'impacts':control,'saved_material_and_draft_preserved':True}
    checkpoint('ordinary unrelated Arc Description fill preserves consumers and draft');capture('arc-description-00-unrelated.png')
    open_arc(application,window,fixture['arc']['name'])
    ui.type_text(arc_editor(application,window,''),window,ARC_NEW)
    ui.wait_for('exact owned public ArcDetail Description commit',lambda:ui.query(database,'SELECT description FROM arcs WHERE id=?',(arc,))==[(ARC_NEW,)])
    preserved();rows=impacts()
    for source in (a_id,b_id):
        row=next(r for r in rows if r['source']==source)
        cause=next((c for c in row.get('causes') or [] if c['dependency_id'].endswith(suffix)),None)
        if not cause or cause['input']!={'kind':'story_arc_field','arc_id':arc,'field':'description'} or cause['reason']!='context_changed':raise RuntimeError('Actual tagged consumer lacks precise Description applicability cause')
    if len(scoped(rows))!=2:raise RuntimeError('Applicability escaped its two actual consumers')
    evidence['ordinary_arc_edit']={'arc_id':arc,'original_omitted_description':'','exact_saved_description':ARC_NEW,'input_route':'ordinary ArcDetail textarea/native keys/public debounced metadata command','saved_manual_text':ANCESTOR_MANUAL,'saved_material_and_draft_preserved':True,'impacts':rows}
    notices=[n for n in ui.walk(application) if n.name=='Screenplay needs review']
    if len(notices)!=2:raise RuntimeError('Expected two generated consumer notices')
    for node in notices:disclosure(node,window,'What changed')
    evidence['visible_applicability_impact']=ui.wait_for('ordinary new Description notice visible',lambda:driver.visible_review_label(application,window,'Story arc description became available.'))
    review=ui.wait_for('ordinary B source review',lambda:review_for_source(application,fixture['b']['name']))
    combo=ui.wait_for('ordinary applicability cause chooser',lambda:ui.reveal(review,lambda n:n.getRole()==ui.pyatspi.ROLE_COMBO_BOX and ui.button_label_matches(n.name,'Input change',prefix=True)))
    ui.click_control(combo,window);ui.command('xdotool','key','--clearmodifiers','End','Return')
    checkpoint('exact ArcDetail entry identifies two consumers without screenplay replacement');capture('arc-description-01-needs-review.png')
    count=len(Provider.records)
    ui.reveal_button(ui.wait_for('chosen B applicability review',lambda:review_for_source(application,fixture['b']['name'])),'Preview update',window)
    def pending():
        if len(Provider.records)>count and not Provider.records[-1]['accepted']:raise RuntimeError('Synthetic applicability provider refused actual prompt')
        rows=ui.query(database,"SELECT p.id,p.proposed_text,b.binding_json FROM propagation_proposals p JOIN script_impact_proposal_bindings b ON b.proposal_id=p.id WHERE p.status='pending' ORDER BY p.rowid DESC")
        return rows[0] if rows else None
    proposed=ui.wait_for('actual canonical pending applicability preview',pending);binding=json.loads(proposed[2])
    expected={'arc_id':arc,'field':'description','value':'','revision_event_id':binding['cause']['consumed_revision_event_id']}
    if proposed[1]!=ARC_PREVIEW or binding.get('arc_description_applicability_previous')!=[expected] or binding.get('arc_description_applicability_current')!=[] or not binding['request']['dependency_id'].endswith(suffix):raise RuntimeError('Pending original omission/source receipt differs')
    if not any(i['arc_id']==arc and i['field']=='description' and i['value']==ARC_NEW for i in binding['arc_inputs']):raise RuntimeError('Pending current supplied Description differs')
    preserved()
    ui.wait_for('reachable exact current Description evidence',lambda:ui.reveal(application,lambda n:n.name=='Current available arc description' and ui.text_of(n)==ARC_NEW))
    evidence['exact_current_description_visible']=ui.wait_for('exact current Description glyphs inside Script pane',lambda:driver.visible_paragraph(application,window,ARC_NEW,'Current available arc description'))
    evidence['original_omission_visible']=ui.wait_for('original omitted prose label inside Script pane',lambda:driver.visible_review_label(application,window,'Description prose was not supplied: the field was empty.'))
    evidence['pending_arc_preview']={'id':proposed[0],'binding':binding,'proposed_text':proposed[1],'saved_material_unchanged':True}
    driver.bible_field(application,window,RED,'right',align=True)
    checkpoint('pending synthetic review shows original omission/current exact Description and awaits acceptance');capture('arc-description-02-pending.png')
    ui.type_text(arc_editor(application,window,ARC_NEW),window,'')
    ui.wait_for('ordinary public Description clear',lambda:ui.query(database,'SELECT description FROM arcs WHERE id=?',(arc,))==[('',)])
    cleared=impacts()
    if scoped(cleared):raise RuntimeError('Clear did not withdraw newly supplied prose cause')
    preserved();before=driver.canonical_state(database)
    ui.reveal_button(ui.wait_for('retained pending proposal after clear',lambda:review_for_source(application,fixture['b']['name'])),'Accept update',window)
    error=ui.wait_for('visible cleared-source refusal',lambda:ui.find(application,lambda n:'proposal' in ui.text_of(n) and ('stale' in ui.text_of(n) or 'changed' in ui.text_of(n))))
    if driver.canonical_state(database)!=before:raise RuntimeError('Cleared-source refusal wrote state')
    preserved();evidence['clear_refusal']={'visible_error':ui.text_of(error),'cause_withdrawn':True,'all_recorded_tables_unchanged':True}
    checkpoint('ordinary clear withdraws applicability cause and refuses old pending acceptance');capture('arc-description-03-cleared-refused.png')
    ui.type_text(arc_editor(application,window,''),window,ARC_NEW)
    ui.wait_for('ordinary public exact Description restore',lambda:ui.query(database,'SELECT description FROM arcs WHERE id=?',(arc,))==[(ARC_NEW,)])
    before=driver.canonical_state(database)
    ui.reveal_button(ui.wait_for('retained old proposal after restore',lambda:review_for_source(application,fixture['b']['name'])),'Accept update',window)
    error=ui.wait_for('visible exact text ABA refusal',lambda:ui.find(application,lambda n:'screenplay proposal is stale' in ui.text_of(n)))
    if driver.canonical_state(database)!=before:raise RuntimeError('Restored-source stale refusal wrote state')
    preserved();evidence['restore_aba_refusal']={'visible_error':ui.text_of(error),'exact_restored_description':ARC_NEW,'all_recorded_tables_unchanged':True}
    checkpoint('ordinary exact clear/restore ABA refuses old preview without writes');capture('arc-description-04-stale.png')
    ui.reveal_button(ui.wait_for('old B review before Reject',lambda:review_for_source(application,fixture['b']['name'])),'Reject',window)
    ui.wait_for('ordinary rejection',lambda:ui.query(database,"SELECT id FROM propagation_proposals WHERE id=? AND status='rejected'",(proposed[0],)))
    Provider.allow_fresh_notes=True
    ui.reveal_button(ui.wait_for('fresh B review',lambda:review_for_source(application,fixture['b']['name'])),'Preview update',window)
    fresh=ui.wait_for('fresh pending applicability preview',pending)
    if fresh[0]==proposed[0]:raise RuntimeError('Fresh review reused old proposal')
    ui.reveal_button(ui.wait_for('fresh B proposal before Accept',lambda:review_for_source(application,fixture['b']['name'])),'Accept update',window)
    accepted=ui.wait_for('explicit chosen-block update',lambda:next((r for r in ui.blocks(database,b_id) if r[1]==ARC_PREVIEW and r[2]!=original[b_id][0][2]),None))
    for node in (a_id,f_id):
        if ui.blocks(database,node)!=original[node]:raise RuntimeError('Acceptance changed unselected saved material')
    if accepted[3:]!=original[b_id][0][3:] or bible_fields(database)!=original_bible:raise RuntimeError('Acceptance changed placement or Bible')
    final=impacts()
    if any(c['dependency_id'].endswith(suffix) for r in final if r['source']==b_id for c in r.get('causes') or []):raise RuntimeError('Accepted B keeps original omission cause')
    if not any(c['dependency_id'].endswith(suffix) for r in final if r['source']==a_id for c in r.get('causes') or []):raise RuntimeError('Acceptance cleared unaccepted A cause')
    evidence['explicit_arc_acceptance']={'proposal_id':fresh[0],'saved_selected_block':accepted,'unselected_material_preserved':True,'placement_and_bible_preserved':True,'impacts':final}
    require_draft(application);driver.bible_field(application,window,RED,'right',align=True)
    ui.wait_for('visible saved synthetic arc update',lambda:ui.screenplay_block(application,driver.screenplay_anchor(ARC_PREVIEW)))
    if len(Provider.records)!=6 or not all(r['accepted'] for r in Provider.records):raise RuntimeError('Expected four fixture/two targeted labelled synthetic calls')
    checkpoint('explicit fresh acceptance updates B alone and refreshes consumed Description lineage');capture('arc-description-05-accepted.png')


def verify_two_tag_fixture(database,fixture):
    primary=fixture['arc']['id'];second=fixture['second_arc']['id'];other=fixture['unrelated_arc']['id']
    receipts=[]
    for key,arcs in [('a',[primary]),('b',[primary,second]),('f',[other])]:
        actual=sorted(a for a, in ui.query(database,'SELECT arc_id FROM node_arcs WHERE node_id=?',(fixture[key]['id'],)))
        if actual!=sorted(arcs):raise RuntimeError('Synthetic two-tag fixture does not retain declared canonical tags')
        if key=='f':continue
        omissions=[]
        for arc in arcs:
            if ui.query(database,'SELECT description FROM arcs WHERE id=?',(arc,))!=[('',)]:raise RuntimeError('Tagged fixture Description is not empty')
            revisions=ui.query(database,"SELECT r.change_event_id FROM object_revisions r JOIN object_revision_fields f ON f.revision_id=r.id WHERE r.object_kind='story_arc' AND r.object_id=? AND f.field_key='description' AND f.new_type='text' AND f.new_text='' ORDER BY r.rowid",(arc,))
            if not revisions:raise RuntimeError('Synthetic two-tag fixture lacks public owned empty field history')
            omissions.append({'arc_id':arc,'field':'description','value':'','revision_event_id':revisions[-1][0]})
        if fixture.get('setup_only'):continue
        rows=ui.query(database,"SELECT c.payload_json,e.id FROM commands c JOIN change_events e ON e.command_id=c.id WHERE c.payload_type='script.generate_block' ORDER BY e.rowid")
        command,event=next((json.loads(raw),event) for raw,event in rows if json.loads(raw)['block']['source_node_id']==fixture[key]['id'])
        if sorted(command.get('arc_description_applicability') or [],key=lambda i:i['arc_id'])!=sorted(omissions,key=lambda i:i['arc_id']) or any(i['field']=='description' for i in command['arc_inputs']):raise RuntimeError('Two-tag generation did not preserve both honest omission receipts')
        for omission in omissions:
            dep='generation.'+event+'.arc_description_applicability.'+omission['arc_id']
            if ui.query(database,'SELECT count(*) FROM semantic_dependencies WHERE id=?',(dep,))!=[(1,)]:raise RuntimeError('Two-tag omission lacks actual graph dependency')
        receipts.append({'source':fixture[key]['id'],'generation_event':event,'applicability':omissions})
    return {'synthetic_project_model_and_persistence_fixture':True,'canonical_tags_verified':True,'public_owned_empty_history_verified':True,'generation_receipts':receipts}


def validate_known_empty_binding(binding,fixture,mode,deleted=False):
    primary=fixture['arc']['id'];second=fixture.get('second_arc')
    expected_previous={primary,second['id']} if second else {primary}
    previous=binding.get('arc_description_applicability_previous')
    if previous is None or len(previous)!=len(expected_previous) or {i['arc_id'] for i in previous}!=expected_previous or any(i['field']!='description' or i['value']!='' or not i['revision_event_id'] for i in previous):raise RuntimeError('Preview lost original owned omission receipts')
    expected_current=set() if deleted else {second['id']} if mode=='known-empty-second-tag' else {primary}
    current=binding.get('arc_description_applicability_current')
    if current is None or len(current)!=len(expected_current) or {i['arc_id'] for i in current}!=expected_current or any(i['field']!='description' or i['value']!='' or not i['revision_event_id'] for i in current):raise RuntimeError('Preview lost current known-empty applicability')
    if any(i!=next(before for before in previous if before['arc_id']==i['arc_id']) for i in current):raise RuntimeError('Unedited omitted Description clock changed during preview')
    inputs=binding.get('arc_inputs')
    if inputs is None:raise RuntimeError('Preview current arc consumption is unknown')
    descriptions=[i for i in inputs if i['field']=='description']
    if mode=='known-empty-second-tag':
        if len(descriptions)!=1 or descriptions[0]['arc_id']!=primary or descriptions[0]['value']!=ARC_NEW:raise RuntimeError('Second empty tag displaced the selected available Description')
    elif descriptions:raise RuntimeError('Known-empty preview falsely claims consumed Description prose')
    absent=binding.get('arc_absence_revisions') or []
    if deleted:
        if inputs or len(absent)!=1 or absent[0][0]!=primary or not absent[0][1]:raise RuntimeError('Fresh deleted-arc preview does not bind actual owned absence')
    elif absent:raise RuntimeError('Present known-empty arc was recorded as absent')
    cause=binding['cause'];expected_field='description' if mode=='known-empty-second-tag' else 'name'
    if cause['input']!={'kind':'story_arc_field','arc_id':primary,'field':expected_field}:raise RuntimeError('Preview selected a different cause')
    if deleted and cause['reason']!='deleted':raise RuntimeError('Fresh deletion preview does not record deletion')
    target=next((i for i in binding['script_inputs'] if i['block_id']==binding['request']['block_id']),None)
    if target is None or target['text']!=ANCESTOR_MANUAL:raise RuntimeError('Preview changed exact saved manual target custody')
    return {'original_omissions':previous,'current_omissions':current,'current_arc_inputs':inputs,'absence_revisions':absent,'cause':cause}


def owned_arc_deletion_receipt(database,arc):
    # Arc deletion removes the live arc and records history. Historical raw
    # node_arcs references may remain; they are not live prompt context.
    if ui.query(database,'SELECT id FROM arcs WHERE id=?',(arc,)):return None
    rows=ui.query(database,"SELECT r.id,r.change_event_id,r.operation,c.id,c.payload_type,c.payload_json FROM object_revisions r JOIN change_events e ON e.id=r.change_event_id JOIN commands c ON c.id=e.command_id WHERE r.object_kind='story_arc' AND r.object_id=? ORDER BY r.rowid DESC LIMIT 1",(arc,))
    if not rows:return None
    revision,event,operation,command,kind,payload=rows[0]
    if operation!='delete' or kind!='story_arc.delete' or json.loads(payload)!={'arc_id':arc}:return None
    fields=ui.query(database,"SELECT field_key,old_type,old_text,new_type FROM object_revision_fields WHERE revision_id=? AND field_key IN ('name','description') ORDER BY field_key",(revision,))
    if fields!=[('description','text','',None),('name','text',KNOWN_EMPTY_NAME,None)]:return None
    live=ui.query(database,'SELECT n.node_id,n.arc_id FROM node_arcs n JOIN arcs a ON a.id=n.arc_id WHERE n.arc_id=?',(arc,))
    if live:return None
    return {'arc_id':arc,'revision_id':revision,'change_event_id':event,'command_id':command,'owned_deleted_fields':fields,'live_tag_context':live,'retained_raw_references':ui.query(database,'SELECT node_id,arc_id FROM node_arcs WHERE arc_id=? ORDER BY node_id',(arc,))}


def qualify_known_empty_arc_preview(application,window,database,fixture,capture,checkpoint,evidence):
    mode=os.environ['EIDETIC_CAPTURE_SCOPE'];primary=fixture['arc']['id'];b_id=fixture['b']['id']
    save_script(application,window,database,b_id,GENERATED_B,ANCESTOR_MANUAL)
    originals={fixture[key]['id']:ui.blocks(database,fixture[key]['id']) for key in ('a','b','f')}
    original_material=material(database);original_bible=bible_fields(database);block_id=originals[b_id][0][0]
    def unselected_material():
        return {'blocks':ui.query(database,'SELECT * FROM script_blocks WHERE id!=? ORDER BY rowid',(block_id,)),
            'spans':ui.query(database,'SELECT * FROM script_spans WHERE block_id!=? ORDER BY rowid',(block_id,)),
            'locks':ui.query(database,'SELECT * FROM script_locks ORDER BY rowid'),
            'segments':ui.query(database,'SELECT * FROM script_segments WHERE source_node_id!=? ORDER BY rowid',(b_id,)),
            'placement':material(database)['timeline_placement']}
    unselected_before=unselected_material()
    block=ui.wait_for('unrelated saved F block',lambda:ui.screenplay_block(application,driver.screenplay_anchor(F_TEXT)))
    ui.reveal_button(block,'Edit',window)
    ui.type_text(ui.wait_for('unrelated screenplay editor',lambda:ui.editable(application,F_TEXT)),window,DRAFT)
    ui.click_button(application,'Bible',window)
    search=ui.wait_for('Bible search',lambda:ui.find(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and 0<=n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0]<400 and ui.text_of(n)==''))
    ui.type_text(search,window,'Mara')
    ui.click_control(ui.wait_for('Mara entity',lambda:ui.reveal(application,lambda n:n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and ui.button_label_matches(n.name,'Mara',prefix=True))),window)
    driver.bible_field(application,window,RED,'right',align=True)
    def preserved():
        if material(database)!=original_material or bible_fields(database)!=original_bible:raise RuntimeError('Known-empty preview/edit changed saved screenplay, spans, locks, placement or Bible')
        require_draft(application)
    def impacts():
        previous=receipt(application)['impactReads'];ui.click_button(application,'QA read canonical impacts',window)
        return ui.wait_for('real canonical known-empty impacts',lambda:(json.loads(v['impacts']) if not v['busy'] and v['impactReads']>previous and v['impacts'] else None) if (v:=receipt(application)) else None)
    def select_cause(deleted=False):
        rows=impacts();row=next(r for r in rows if r['source']==b_id)
        field='description' if mode=='known-empty-second-tag' else 'name'
        candidates=[c for c in row['causes'] if c['input']=={'kind':'story_arc_field','arc_id':primary,'field':field} and (not deleted or c['reason']=='deleted')]
        if len(candidates)!=1:raise RuntimeError('Known-empty native scenario lacks one precise selected cause')
        review=ui.wait_for('ordinary B known-empty review',lambda:review_for_source(application,fixture['b']['name']))
        combo=ui.wait_for('ordinary known-empty cause chooser',lambda:ui.reveal(review,lambda n:n.getRole()==ui.pyatspi.ROLE_COMBO_BOX and ui.button_label_matches(n.name,'Input change',prefix=True)))
        index=row['causes'].index(candidates[0])
        ui.click_control(combo,window);ui.command('xdotool','key','--clearmodifiers','Home',*(['Down']*index),'Return')
        return candidates[0]
    def preview(deleted=False):
        cause=select_cause(deleted);before_ids={r[0] for r in ui.query(database,'SELECT id FROM propagation_proposals')};count=len(Provider.records)
        ui.reveal_button(ui.wait_for('selected B known-empty review',lambda:review_for_source(application,fixture['b']['name'])),'Preview update',window)
        def pending():
            if len(Provider.records)>count and not Provider.records[-1]['accepted']:raise RuntimeError('Synthetic known-empty provider refused actual prompt')
            rows=ui.query(database,"SELECT p.id,p.proposed_text,b.binding_json FROM propagation_proposals p JOIN script_impact_proposal_bindings b ON b.proposal_id=p.id WHERE p.status='pending' ORDER BY p.rowid DESC")
            return next((row for row in rows if row[0] not in before_ids),None)
        proposed=ui.wait_for('new canonical known-empty pending proposal',pending)
        if proposed[1]!=KNOWN_EMPTY_PREVIEW:raise RuntimeError('Unexpected known-empty synthetic response')
        binding=json.loads(proposed[2]);proof=validate_known_empty_binding(binding,fixture,mode,deleted)
        if binding['request']['dependency_id']!=cause['dependency_id']:raise RuntimeError('Native cause chooser selected the wrong dependency')
        preserved();return proposed,proof
    open_arc(application,window,fixture['arc']['name'])
    if mode=='known-empty-second-tag':
        ui.type_text(arc_editor(application,window,''),window,ARC_NEW)
        ui.wait_for('exact public primary Description commit',lambda:ui.query(database,'SELECT description FROM arcs WHERE id=?',(primary,))==[(ARC_NEW,)])
    else:
        name=ui.wait_for('ordinary ArcDetail Name input',lambda:ui.reveal(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n)==fixture['arc']['name'] and 0<=n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0]<400))
        ui.type_text(name,window,KNOWN_EMPTY_NAME)
        ui.wait_for('ordinary exact arc Name commit',lambda:ui.query(database,'SELECT name,description FROM arcs WHERE id=?',(primary,))==[(KNOWN_EMPTY_NAME,'')])
    preserved();proposed,proof=preview();evidence['pending_known_empty_preview']={'id':proposed[0],**proof,'saved_material_and_draft_preserved':True}
    ui.wait_for('native still-empty evidence',lambda:ui.reveal(application,lambda n:n.name=='Current available arc description' and ui.text_of(n)=='(still empty; not supplied)'))
    evidence['still_empty_visible']=ui.wait_for('still-empty evidence glyphs inside Script pane',lambda:driver.visible_paragraph(application,window,'(still empty; not supplied)','Current available arc description'))
    if mode=='known-empty-second-tag':
        evidence['current_primary_description_visible']=ui.wait_for('available primary Description glyphs',lambda:driver.visible_paragraph(application,window,ARC_NEW,'Current available arc description'))
    checkpoint(mode+': ordinary pending preview preserves exact known-empty evidence and saved material');capture(mode+'-01-pending.png')
    if mode=='known-empty-deletion':
        delete=ui.wait_for('ordinary ArcDetail Delete button',lambda:ui.reveal(application,lambda n:n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and n.name=='Delete' and 0<=n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0]<400))
        ui.click_control(delete,window)
        deletion=ui.wait_for('live arc absent with exact owned deletion revision',lambda:owned_arc_deletion_receipt(database,primary))
        evidence['ordinary_arc_deletion']=deletion
        before=driver.canonical_state(database)
        ui.reveal_button(ui.wait_for('retained old known-empty preview',lambda:review_for_source(application,fixture['b']['name'])),'Accept update',window)
        error=ui.wait_for('visible deleted-arc stale refusal',lambda:ui.find(application,lambda n:'proposal' in ui.text_of(n) and ('stale' in ui.text_of(n) or 'changed' in ui.text_of(n))))
        if driver.canonical_state(database)!=before:raise RuntimeError('Deleted known-empty stale acceptance wrote canonical state')
        preserved();evidence['deleted_known_empty_refusal']={'visible_error':ui.text_of(error),'all_recorded_tables_unchanged':True}
        checkpoint('ordinary arc deletion refuses old known-empty preview without writes');capture(mode+'-02-deleted-refused.png')
        ui.reveal_button(ui.wait_for('old preview before rejection',lambda:review_for_source(application,fixture['b']['name'])),'Reject',window)
        ui.wait_for('old known-empty preview rejected',lambda:ui.query(database,"SELECT id FROM propagation_proposals WHERE id=? AND status='rejected'",(proposed[0],)))
        Provider.allow_fresh_notes=True
        fresh,proof=preview(deleted=True)
        if fresh[0]==proposed[0]:raise RuntimeError('Fresh absence preview reused old proposal')
        if proof['absence_revisions']!=[[primary,deletion['change_event_id']]]:raise RuntimeError('Fresh absence preview does not bind the exact ordinary deletion event')
        proposed=fresh;evidence['fresh_absence_preview']={'id':fresh[0],**proof}
        ui.wait_for('native removed-context evidence',lambda:ui.reveal(application,lambda n:n.name=='Current available arc description' and ui.text_of(n)=='(removed from current context)'))
        evidence['removed_context_visible']=ui.wait_for('removed-context glyphs inside Script pane',lambda:driver.visible_paragraph(application,window,'(removed from current context)','Current available arc description'))
        checkpoint('fresh native preview binds owned arc absence');capture(mode+'-03-fresh-absence.png')
    ui.reveal_button(ui.wait_for('review before explicit known-empty acceptance',lambda:review_for_source(application,fixture['b']['name'])),'Accept update',window)
    accepted=ui.wait_for('exact chosen-block known-empty acceptance',lambda:next((r for r in ui.blocks(database,b_id) if r[1]==KNOWN_EMPTY_PREVIEW and r[2]!=originals[b_id][0][2]),None))
    if unselected_material()!=unselected_before or bible_fields(database)!=original_bible or accepted[3:]!=originals[b_id][0][3:]:raise RuntimeError('Known-empty acceptance changed unselected material, locks, placement or Bible')
    for key in ('a','f'):
        if ui.blocks(database,fixture[key]['id'])!=originals[fixture[key]['id']]:raise RuntimeError('Known-empty acceptance changed unselected source')
    require_draft(application);final=impacts();chosen=next(r for r in final if r['source']==b_id)
    if any(c['input'].get('kind')=='story_arc_field' and c['input'].get('arc_id')==primary for c in chosen['causes']):raise RuntimeError('Acceptance retained repaired primary arc cause')
    evidence['explicit_known_empty_acceptance']={'proposal_id':proposed[0],'selected_block':accepted,'unselected_blocks_spans_locks_segments_placement_bible_and_draft_preserved':True,'impacts':final}
    ui.wait_for('visible saved known-empty synthetic text',lambda:ui.screenplay_block(application,driver.screenplay_anchor(KNOWN_EMPTY_PREVIEW)))
    checkpoint(mode+': explicit acceptance updates B alone');capture(mode+'-04-accepted.png')
    if mode!='known-empty-deletion':
        tracked=fixture['second_arc'] if mode=='known-empty-second-tag' else {'id':primary,'name':KNOWN_EMPTY_NAME}
        open_arc(application,window,tracked['name']);ui.type_text(arc_editor(application,window,''),window,ARC_UNRELATED)
        ui.wait_for('later exact tracked Description fill',lambda:ui.query(database,'SELECT description FROM arcs WHERE id=?',(tracked['id'],))==[(ARC_UNRELATED,)])
        after=impacts();chosen=next(r for r in after if r['source']==b_id)
        scoped=[c for c in chosen['causes'] if c['input']=={'kind':'story_arc_field','arc_id':tracked['id'],'field':'description'} and c['dependency_id'].endswith('.arc_description_applicability.'+tracked['id'])]
        if len(scoped)!=1:raise RuntimeError('Acceptance lost separate known-empty applicability for later fill')
        if ui.blocks(database,b_id)!=[accepted] or unselected_material()!=unselected_before or bible_fields(database)!=original_bible:raise RuntimeError('Later applicability fill changed saved material')
        require_draft(application);evidence['later_empty_fill_lineage']={'arc_id':tracked['id'],'exact_description':ARC_UNRELATED,'precise_cause':scoped[0],'saved_material_preserved':True}
        checkpoint(mode+': later fill retains the precise omitted-description dependency');capture(mode+'-05-later-fill.png')
    expected=6 if mode=='known-empty-deletion' else 5
    if len(Provider.records)!=expected or not all(r['accepted'] and r['synthetic'] and not r['real_model'] for r in Provider.records):raise RuntimeError('Wrong number or custody of known-empty synthetic calls')


def context_prompt_matches(user,notes):
    return ('SCENE NOTES:\n'+notes+'\n\n' in user and DRAFT not in user)


def context_receipt(application):
    node=ui.find(application,lambda n:n.name=='QA context receipt' and n.getRole() in (ui.pyatspi.ROLE_TEXT,ui.pyatspi.ROLE_ENTRY))
    if node is None:return None
    text=node.queryText();value=json.loads(text.getText(0,text.characterCount))
    if value['seam']!='QA held real public context return; no fabricated prompt':raise RuntimeError('Unlabelled context seam')
    return value


def raw_prompt_visible(application,window,exact,notes):
    # Compare the real native User Prompt text, excluding the JSON receipt.
    # Read complete text: prompts legitimately exceed the diagnostic helper's 4K cap.
    application.clear_cache()
    for node in ui.walk(application):
        try:text=node.queryText();actual=text.getText(0,text.characterCount)
        except NotImplementedError:continue
        if actual!=exact:continue
        offset=actual.index('SCENE NOTES:\n')+len('SCENE NOTES:\n')
        # The ordinary disclosure begins near the editor's lower edge. Scroll
        # its real text range through AT-SPI before requiring visible glyphs.
        # This changes viewport position only, never rendered text or app state.
        text.scrollSubstringTo(offset,offset+len(notes),ui.pyatspi.SCROLL_ANYWHERE)
        node.clear_cache()
        if not ui.visible(node):continue
        rect=tuple(node.queryComponent().getExtents(ui.pyatspi.XY_SCREEN))
        glyphs=[tuple(text.getCharacterExtents(offset+i,ui.pyatspi.XY_SCREEN)) for i,char in enumerate(notes) if not char.isspace()]
        top=[tuple(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)) for n in ui.walk(application) if n.name=='Resize panels' and ui.visible(n) and n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[2]>900 and n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[1]<720]
        if len(top)!=1:raise RuntimeError('Ambiguous actual editor viewport')
        x,y,width,height=top[0];viewport=(x,0,width,y)
        if glyphs and all(driver.contained(g,rect) and driver.contained(g,viewport) for g in glyphs):
            return {'exact_user_prompt':actual,'exact_notes':notes,'native_prompt_bounds':list(rect),'editor_viewport':list(viewport),'notes_character_bounds':[list(g) for g in glyphs],'native_notes_glyphs_visible':True}
    return None


def qualify_notes_prompt(application,window,database,fixture,capture,checkpoint,evidence):
    b_id=fixture['b']['id'];original=material(database);original_bible=bible_fields(database)
    def saved_notes():return json.loads(ui.query(database,'SELECT content_json FROM nodes WHERE id=?',(b_id,))[0][0])['notes']
    old=saved_notes()
    block=ui.wait_for('unrelated saved F block',lambda:ui.screenplay_block(application,driver.screenplay_anchor(F_TEXT)))
    ui.reveal_button(block,'Edit',window)
    ui.type_text(ui.wait_for('unrelated F editor',lambda:ui.editable(application,F_TEXT)),window,DRAFT)
    ui.click_button(application,'Bible',window)
    search=ui.wait_for('Bible search',lambda:ui.find(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and 0<=n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0]<400 and ui.text_of(n)==''))
    ui.type_text(search,window,'Mara')
    ui.click_control(ui.wait_for('Mara entity',lambda:ui.reveal(application,lambda n:n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and ui.button_label_matches(n.name,'Mara',prefix=True))),window)
    driver.bible_field(application,window,RED,'left',align=True)
    clip=ui.wait_for('ordinary B timeline clip',lambda:ui.native_timeline_clip(application,fixture['b']['name'],window));ui.click_control(clip[0],window,clip[1])
    splitter=ui.wait_for('existing editor splitter',lambda:ui.find(application,lambda n:n.name=='Resize panels' and ui.visible(n) and n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[2]>900 and n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[1]<720))
    ui.click_control(splitter,window)
    ui.wait_for('editor splitter native focus',lambda:splitter.getState().contains(ui.pyatspi.STATE_FOCUSED))
    ui.command('xdotool','key','--clearmodifiers','--repeat','16','--delay','40','Down')
    def open_raw():
        panel=ui.wait_for('ordinary Raw AI Prompt disclosure',lambda:ui.reveal(application,lambda n:n.name.upper().startswith('RAW AI PROMPT') and n.getRole() in (ui.pyatspi.ROLE_UNKNOWN,ui.pyatspi.ROLE_PUSH_BUTTON,ui.pyatspi.ROLE_TOGGLE_BUTTON)))
        ui.click_control(panel,window)
    def returned(notes,after=0):
        value=context_receipt(application)
        if value is None:return None
        return next((call for call in reversed(value['calls']) if call['node_id']==b_id and call['id']>after and call['stage']=='returned' and context_prompt_matches(call.get('actual_user',''),notes)),None)
    def preserved():
        if material(database)!=original or bible_fields(database)!=original_bible:raise RuntimeError('Context reads/Notes edit changed saved screenplay, placement, or Bible')
        require_draft(application)
    first=ui.wait_for('real original committed Notes context',lambda:returned(old));open_raw()
    evidence['original_context']=ui.wait_for('original Notes glyphs visible in actual User Prompt',lambda:raw_prompt_visible(application,window,first['actual_user'],old))
    preserved();checkpoint('original real public context with Bible timeline and saved screenplay');capture('notes-prompt-01-original.png')
    ui.click_button(application,'QA hold next real context response',window)
    refresh=ui.wait_for('ordinary Raw Prompt Refresh',lambda:ui.reveal(application,lambda n:n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and n.name in ('Refresh','REFRESH')))
    ui.click_control(refresh,window)
    def held():
        value=context_receipt(application)
        if value is None or not value['held']:return None
        return next((call for call in value['calls'] if call['node_id']==b_id and call['stage']=='held' and context_prompt_matches(call.get('actual_user',''),old)),None)
    old_read=ui.wait_for('held unmodified older actual context return',held)
    field_node=ui.wait_for('ordinary selected Notes editor',lambda:ui.reveal(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n)==old))
    ui.type_text(field_node,window,NOTES);ui.wait_for('exact public debounced Notes commit',lambda:saved_notes()==NOTES)
    fresh=ui.wait_for('new exact Notes public context while older read remains held',lambda:returned(NOTES,old_read['id']))
    evidence['fresh_context']=ui.wait_for('fresh Notes glyphs visible in actual User Prompt',lambda:raw_prompt_visible(application,window,fresh['actual_user'],NOTES))
    if not context_receipt(application)['held']:raise RuntimeError('Older context was not still held')
    evidence['older_held_real_read']=old_read;evidence['ordinary_notes_edit']={'original':old,'exact_saved_notes':NOTES,'native_keys_and_public_debounced_commit':True}
    preserved();checkpoint('new committed exact Notes refreshes prompt while original real response remains held');capture('notes-prompt-02-fresh-old-held.png')
    history=driver.canonical_state(database)
    ui.click_button(application,'QA release real context response',window)
    ui.wait_for('older actual response returned',lambda:not context_receipt(application)['held'] and any(c['id']==old_read['id'] and c['stage']=='returned' for c in context_receipt(application)['calls']))
    evidence['after_late_return']=ui.wait_for('late older response refused by production context owner',lambda:raw_prompt_visible(application,window,fresh['actual_user'],NOTES))
    if driver.canonical_state(database)!=history:raise RuntimeError('Read-only delayed return wrote canonical history')
    preserved();evidence['stale_checks'].append('held older real context success cannot replace fresh same-node Notes context')
    checkpoint('released older real response cannot replace latest authored Notes prompt');capture('notes-prompt-03-late-return-refused.png')
    field_node=ui.wait_for('Notes editor before clear',lambda:ui.editable(application,NOTES));ui.type_text(field_node,window,'')
    ui.wait_for('ordinary Notes clear saved',lambda:saved_notes()=='')
    ui.wait_for('empty Notes remove Raw Prompt panel',lambda:not ui.find(application,lambda n:n.name.upper().startswith('RAW AI PROMPT')))
    preserved();evidence['ordinary_notes_clear']={'exact_saved_notes':'','raw_prompt_panel_absent':True,'saved_material_and_draft_preserved':True}
    checkpoint('ordinary Notes clear commits empty text and removes Raw Prompt without saved material changes');capture('notes-prompt-03b-cleared.png')
    # The observed native label is NOTES at x296; a broad x>400 selector
    # can instead reach the empty right-hand Bible SUMMARY draft.
    field_node=ui.wait_for('empty selected Notes editor',lambda:ui.reveal(application,lambda n:n.name.upper()=='NOTES' and n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n)==''))
    ui.type_text(field_node,window,NOTES);ui.wait_for('exact restored Notes public commit',lambda:saved_notes()==NOTES)
    restored=ui.wait_for('restored Notes owns new actual context read',lambda:returned(NOTES,fresh['id']));open_raw()
    evidence['restored_context']=ui.wait_for('restored exact Notes glyphs visible',lambda:raw_prompt_visible(application,window,restored['actual_user'],NOTES))
    preserved()
    if len(Provider.records)!=4 or not all(r['accepted'] for r in Provider.records):raise RuntimeError('Context preview unexpectedly requested inference beyond four synthetic fixture calls')
    evidence['context_reads']=context_receipt(application)['calls'];evidence['saved_screenplay_placement_bible_and_unrelated_draft_preserved']=True
    checkpoint('clear and exact restore request fresh context; four labelled synthetic fixture calls only');capture('notes-prompt-04-restored.png')


def pending_script_review(database):
    rows=ui.query(database,"SELECT p.id,b.binding_json,p.proposed_text FROM propagation_proposals p JOIN script_impact_proposal_bindings b ON b.proposal_id=p.id WHERE p.status='pending' ORDER BY p.rowid DESC")
    return (rows[0][0],json.loads(rows[0][1]),rows[0][2]) if rows else None


def qualify_screenplay_removal(application,window,database,fixture,capture,checkpoint,evidence):
    a_id=fixture['a']['id'];b_id=fixture['b']['id'];f_id=fixture['f']['id']
    placement=material(database)['timeline_placement'];world=bible_fields(database)
    original={node:ui.blocks(database,node) for node in (a_id,b_id,f_id)}
    source_id=original[a_id][0][0]
    generation_rows=ui.query(database,"SELECT payload_json FROM commands WHERE payload_type='script.generate_block' ORDER BY rowid")
    consumer=next(json.loads(raw) for raw, in generation_rows if json.loads(raw)['block']['source_node_id']==b_id)
    if not any(i['block_id']==source_id and i['text']==GENERATED_A for i in consumer['script_inputs']):raise RuntimeError('B did not genuinely consume the source block in its production generation')
    save_script(application,window,database,b_id,GENERATED_B,ANCESTOR_MANUAL)
    save_script(application,window,database,a_id,GENERATED_A,SAVED)
    source=ui.blocks(database,a_id)[0];saved_b=ui.blocks(database,b_id)
    block=ui.wait_for('unrelated saved F block',lambda:ui.screenplay_block(application,driver.screenplay_anchor(F_TEXT)))
    ui.reveal_button(block,'Edit',window)
    ui.type_text(ui.wait_for('unrelated exact draft editor',lambda:ui.editable(application,F_TEXT)),window,DRAFT)
    ui.click_button(application,'Bible',window)
    search=ui.wait_for('Bible search',lambda:ui.find(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and 0<=n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0]<400 and ui.text_of(n)==''))
    ui.type_text(search,window,'Mara')
    ui.click_control(ui.wait_for('Mara entity',lambda:ui.reveal(application,lambda n:n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and ui.button_label_matches(n.name,'Mara',prefix=True))),window)
    evidence['visible_bible']=driver.bible_field(application,window,RED,'right',align=True)
    def preserved():
        if ui.blocks(database,b_id)!=saved_b or ui.blocks(database,f_id)!=original[f_id]:raise RuntimeError('Removal or preview replaced manual B/unrelated F')
        if material(database)['timeline_placement']!=placement or bible_fields(database)!=world:raise RuntimeError('Removal changed timeline placement or Bible canon')
        require_draft(application)
    def impacts():
        previous=receipt(application)['impactReads'];ui.click_button(application,'QA read canonical impacts',window)
        return ui.wait_for('fresh public impacts',lambda:(json.loads(v['impacts']) if not v['busy'] and v['impactReads']>previous and v['impacts'] else None) if (v:=receipt(application)) else None)
    def select_source():
        rows=impacts();row=next(r for r in rows if r['source']==b_id)
        index=next(i for i,c in enumerate(row['allCauses']) if c['input']=={'kind':'script_block','block_id':source_id})
        review=ui.wait_for('B review owner',lambda:review_for_source(application,fixture['b']['name']))
        dropdown=ui.wait_for('B input change selector',lambda:ui.reveal(review,lambda n:n.getRole()==ui.pyatspi.ROLE_COMBO_BOX and n.name.startswith('Input change')))
        ui.click_control(dropdown,window);keys=['Home']+['Down']*index+['Return'];ui.command('xdotool','key','--clearmodifiers',*keys)
        return rows
    def pending():return pending_script_review(database)
    evidence['impacts_before_removal']=select_source()
    ui.reveal_button(ui.wait_for('B preview control',lambda:review_for_source(application,fixture['b']['name'])),'Preview update',window)
    old=ui.wait_for('original pending targeted preview',pending)
    if old[1]['cause']['input']!={'kind':'script_block','block_id':source_id}:raise RuntimeError('Preview selected a different source')
    preserved();driver.bible_field(application,window,RED,'right',align=True)
    checkpoint('exact ordinary manual source edit and saved manual target produce pending downstream review');capture('screenplay-removal-01-pending-before.png')
    before=driver.canonical_state(database)
    block=ui.wait_for('source block for ordinary removal',lambda:ui.screenplay_block(application,driver.screenplay_anchor(SAVED)))
    ui.reveal_button(block,'Remove block',window)
    confirmation=ui.wait_for('ordinary removal confirmation',lambda:ui.reveal(application,lambda n:n.name=='Remove saved screenplay block'))
    if not any(ui.text_of(n)==SAVED for n in ui.walk(confirmation)):raise RuntimeError('Confirmation lost exact saved source text')
    if driver.canonical_state(database)!=before:raise RuntimeError('Opening confirmation wrote canon')
    checkpoint('ordinary exact-text removal confirmation awaits explicit action');capture('screenplay-removal-02-confirmation.png')
    ui.reveal_button(confirmation,'Keep block',window)
    if driver.canonical_state(database)!=before:raise RuntimeError('Cancellation wrote canon')
    block=ui.wait_for('source after cancellation',lambda:ui.screenplay_block(application,driver.screenplay_anchor(SAVED)))
    ui.reveal_button(block,'Remove block',window);ui.click_button(application,'QA lose next removal acknowledgement',window)
    ui.reveal_button(application,'Remove saved block',window)
    ui.wait_for('soft-deleted exact block',lambda:ui.query(database,'SELECT text,deleted_event_id FROM script_blocks WHERE id=? AND deleted_event_id IS NOT NULL',(source_id,)))
    ui.wait_for('visible uncertain original receipt',lambda:ui.reveal(application,lambda n:n.name=='Retry same removal' and n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON))
    ui.choose_mode(application,'Graph',window);ui.choose_mode(application,'Script',window)
    driver.enlarge_script_pane(application,window)
    ui.wait_for('orphan removal retained after navigation',lambda:ui.reveal(application,lambda n:n.name=='Retry same removal'))
    preserved();checkpoint('labelled lost acknowledgement retains exact visible retry after canonical disappearance and navigation');capture('screenplay-removal-03-uncertain.png')
    def removal_receipt():
        node=ui.find(application,lambda n:n.name=='QA removal receipt' and n.getRole() in (ui.pyatspi.ROLE_TEXT,ui.pyatspi.ROLE_ENTRY))
        if not node:return None
        text=node.queryText();return json.loads(text.getText(0,text.characterCount))
    committed=ui.wait_for('one committed public removal receipt',lambda:(v if len(v['calls'])==1 else None) if (v:=removal_receipt()) else None)
    if committed['calls'][0]['outcome']!='recorded':raise RuntimeError('Synthetic fault did not follow a genuine commit')
    before_retry=driver.canonical_state(database);ui.reveal_button(application,'Retry same removal',window)
    replay=ui.wait_for('exact acknowledged replay receipt',lambda:(v if len(v['calls'])==2 else None) if (v:=removal_receipt()) else None)
    if replay['calls'][1]['outcome']!='already_recorded' or replay['calls'][0]['payload']!=replay['calls'][1]['payload'] or replay['calls'][0]['commandId']!=replay['calls'][1]['commandId']:raise RuntimeError('Retry changed removal identity or reapplied it')
    if driver.canonical_state(database)!=before_retry:raise RuntimeError('Replay changed canonical/history tables')
    evidence['interrupted_removal']=replay
    removal_event=ui.query(database,'SELECT deleted_event_id FROM script_blocks WHERE id=?',(source_id,))[0][0]
    deleted_revision=ui.query(database,"SELECT r.operation,f.old_text,f.new_type FROM object_revisions r JOIN object_revision_fields f ON f.revision_id=r.id WHERE r.object_kind='script_block' AND r.object_id=? AND r.change_event_id=? AND f.field_key='text'",(source_id,removal_event))
    if deleted_revision!=[('delete',SAVED,None)]:raise RuntimeError('Deletion did not retain exact authored history')
    evidence['retained_history']=deleted_revision
    before_refusal=driver.canonical_state(database)
    ui.reveal_button(ui.wait_for('old proposal review',lambda:review_for_source(application,fixture['b']['name'])),'Accept update',window)
    refusal=ui.wait_for('ordinary stale acceptance error',lambda:ui.find(review,lambda n:'screenplay proposal is stale' in ui.text_of(n)) if (review:=review_for_source(application,fixture['b']['name'])) else None)
    if driver.canonical_state(database)!=before_refusal:raise RuntimeError('Old preview acceptance wrote canon')
    preserved();evidence['stale_checks'].append({'source_removed':True,'old_proposal_id':old[0],'refusal_without_any_recorded_write':True,'visible_refusal':{'text':ui.text_of(refusal),'native_role':refusal.getRoleName(),'native_bounds':list(refusal.queryComponent().getExtents(ui.pyatspi.XY_SCREEN))}})
    ui.reveal_button(ui.wait_for('old proposal before rejection',lambda:review_for_source(application,fixture['b']['name'])),'Reject',window)
    ui.wait_for('old preview rejected',lambda:ui.query(database,"SELECT id FROM propagation_proposals WHERE id=? AND status='rejected'",(old[0],)))
    evidence['impacts_after_removal']=select_source();Provider.allow_fresh_notes=True
    ui.reveal_button(ui.wait_for('fresh B review',lambda:review_for_source(application,fixture['b']['name'])),'Preview update',window)
    fresh=ui.wait_for('fresh deleted-source pending preview',pending)
    if fresh[0]==old[0] or fresh[1]['cause']['reason']!='deleted' or any(i['block_id']==source_id for i in fresh[1]['script_inputs']):raise RuntimeError('Fresh preview reused removed current source')
    if fresh[1]['cause']['input_excerpt']!=GENERATED_A:raise RuntimeError('Fresh preview lost historical consumed text')
    preserved();driver.bible_field(application,window,RED,'right',align=True)
    evidence['fresh_pending_binding']=fresh[1]
    checkpoint('old preview refused; fresh deleted-source review preserves manual target and awaits acceptance');capture('screenplay-removal-04-deleted-review.png')
    ui.reveal_button(ui.wait_for('fresh proposal acceptance owner',lambda:review_for_source(application,fixture['b']['name'])),'Accept update',window)
    accepted=ui.wait_for('explicit chosen-block acceptance',lambda:next((r for r in ui.blocks(database,b_id) if r[1]==REMOVAL_PREVIEW and r[2]!=saved_b[0][2]),None))
    if ui.blocks(database,a_id) or ui.blocks(database,f_id)!=original[f_id] or accepted[3:]!=saved_b[0][3:]:raise RuntimeError('Acceptance changed unselected material or placement')
    if material(database)['timeline_placement']!=placement or bible_fields(database)!=world:raise RuntimeError('Acceptance changed timeline or Bible canon')
    final=impacts();chosen=next(r for r in final if r['source']==b_id)
    if any(c['input']=={'kind':'script_block','block_id':source_id} for c in chosen.get('causes') or []):raise RuntimeError('Accepted lineage still consumes removed source')
    require_draft(application);driver.bible_field(application,window,RED,'right',align=True)
    ui.wait_for('visible accepted synthetic replacement',lambda:ui.screenplay_block(application,driver.screenplay_anchor(REMOVAL_PREVIEW)))
    evidence['explicit_acceptance']={'proposal_id':fresh[0],'saved_block':accepted,'manual_target_preserved_until_acceptance':True,'unrelated_saved_and_draft_preserved':True,'bible_and_timeline_unchanged':True,'final_impacts':final}
    if len(Provider.records)!=6 or not all(r['accepted'] for r in Provider.records):raise RuntimeError('Expected four genuine fixture/two targeted synthetic calls')
    checkpoint('explicit acceptance changes B alone and refreshes current memory after deletion');capture('screenplay-removal-05-accepted.png')


def qualify_timeline_title(application,window,database,fixture,capture,checkpoint,evidence):
    a_id=fixture['a']['id'];b_id=fixture['b']['id'];f_id=fixture['f']['id']
    placement=material(database)['timeline_placement'];world=bible_fields(database)
    original={node:ui.blocks(database,node) for node in (a_id,b_id,f_id)}
    consumer=next(json.loads(raw) for raw, in ui.query(database,"SELECT payload_json FROM commands WHERE payload_type='script.generate_block' ORDER BY rowid") if json.loads(raw)['block']['source_node_id']==b_id)
    old=next(i for i in consumer['timeline_title_inputs'] if i['node_id']==a_id)
    if old['name']!='SCENE A' or not old['revision_event_id']:raise RuntimeError('B did not consume the actual canonically created A title')
    evidence['original_title_consumption']=old
    save_script(application,window,database,b_id,GENERATED_B,ANCESTOR_MANUAL)
    saved_b=ui.blocks(database,b_id)
    block=ui.wait_for('unrelated saved F block',lambda:ui.screenplay_block(application,driver.screenplay_anchor(F_TEXT)))
    ui.reveal_button(block,'Edit',window)
    ui.type_text(ui.wait_for('unrelated exact draft editor',lambda:ui.editable(application,F_TEXT)),window,DRAFT)
    ui.click_button(application,'Bible',window)
    search=ui.wait_for('Bible search',lambda:ui.find(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and 0<=n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN)[0]<400 and ui.text_of(n)==''))
    ui.type_text(search,window,'Mara')
    ui.click_control(ui.wait_for('Mara entity',lambda:ui.reveal(application,lambda n:n.getRole()==ui.pyatspi.ROLE_PUSH_BUTTON and ui.button_label_matches(n.name,'Mara',prefix=True))),window)
    evidence['visible_bible']=driver.bible_field(application,window,RED,'right',align=True)
    def preserved():
        if ui.blocks(database,b_id)!=saved_b or ui.blocks(database,a_id)!=original[a_id] or ui.blocks(database,f_id)!=original[f_id]:raise RuntimeError('Title editing/preview changed manual or unrelated saved screenplay')
        if material(database)['timeline_placement']!=placement or bible_fields(database)!=world:raise RuntimeError('Title editing/preview changed placement or Bible facts')
        require_draft(application)
    editor=select_title(application,window,'SCENE A');ui.type_text(editor,window,TITLE_NEW)
    if title_input(database,a_id)!='SCENE A':raise RuntimeError('Title typing committed without Save')
    select_title(application,window,'SCENE B')
    editor=select_title(application,window,'SCENE A')
    if ui.text_of(editor)!=TITLE_NEW:raise RuntimeError('Timeline navigation lost the exact title draft')
    ui.click_button(application,'Save title',window)
    ui.wait_for('exact canonical title saved',lambda:title_input(database,a_id)==TITLE_NEW)
    ui.wait_for('renamed ordinary timeline clip',lambda:ui.native_timeline_clip(application,TITLE_NEW,window))
    ui.wait_for('saved title status',lambda:ui.find(application,lambda n:'Title saved.' in ui.text_of(n)))
    rename_rows=ui.query(database,"SELECT f.old_text,f.new_text,r.change_event_id FROM object_revisions r JOIN object_revision_fields f ON f.revision_id=r.id JOIN change_events e ON e.id=r.change_event_id WHERE r.object_kind='timeline_node' AND r.object_id=? AND f.field_key='name' ORDER BY e.rowid DESC LIMIT 1",(a_id,))
    if rename_rows[0][:2]!=('SCENE A',TITLE_NEW):raise RuntimeError('Exact title history delta differs')
    evidence['ordinary_title_save']={'node_id':a_id,'exact_title':TITLE_NEW,'old_new_owned_history':rename_rows[0],'native_input_keys_then_explicit_save':True,'draft_survived_timeline_navigation':True}
    def impacts():
        previous=receipt(application)['impactReads'];ui.click_button(application,'QA read canonical impacts',window)
        return ui.wait_for('fresh public title impacts',lambda:(json.loads(v['impacts']) if not v['busy'] and v['impactReads']>previous and v['impacts'] else None) if (v:=receipt(application)) else None)
    def select_cause():
        rows=impacts();row=next(r for r in rows if r['source']==b_id)
        index=next(i for i,c in enumerate(row['allCauses']) if c['dependency_id'].endswith('.timeline_title.'+a_id))
        review=ui.wait_for('B title review owner',lambda:review_for_source(application,fixture['b']['name']))
        dropdown=ui.wait_for('B title input change selector',lambda:ui.reveal(review,lambda n:n.getRole()==ui.pyatspi.ROLE_COMBO_BOX and n.name.startswith('Input change')))
        ui.click_control(dropdown,window);ui.command('xdotool','key','--clearmodifiers','Home',*(['Down']*index),'Return')
        return rows
    preserved();evidence['impacts_after_title_save']=select_cause()
    checkpoint('exact ordinary title save updates actual timeline and retains Bible manual screenplay and draft');capture('timeline-title-01-saved-review.png')
    ui.reveal_button(ui.wait_for('B title preview owner',lambda:review_for_source(application,fixture['b']['name'])),'Preview update',window)
    pending=ui.wait_for('targeted title pending proposal',lambda:pending_script_review(database))
    binding=pending[1]
    if not binding['cause']['dependency_id'].endswith('.timeline_title.'+a_id) or pending[2]!=TITLE_PREVIEW:raise RuntimeError('Preview bound the wrong title cause or text')
    if next(i for i in binding['timeline_title_previous'] if i['node_id']==a_id)!=old:raise RuntimeError('Original consumed title was lost')
    if next(i for i in binding['timeline_title_current'] if i['node_id']==a_id)['name']!=TITLE_NEW:raise RuntimeError('Preview did not bind current exact title')
    preserved();evidence['first_pending_binding']=binding
    ui.wait_for('reachable exact current title evidence',lambda:ui.reveal(application,lambda n:n.name=='Current preview title' and ui.text_of(n)==TITLE_NEW))
    evidence['first_visible_title']=ui.wait_for('exact current title glyphs',lambda:driver.visible_paragraph(application,window,TITLE_NEW,'Current preview title'))
    checkpoint('original and current consumed title visible while exact authored B stays saved');capture('timeline-title-02-pending.png')
    select_title(application,window,TITLE_NEW)
    ui.click_button(application,'Discard title draft and reload',window)
    editor=ui.wait_for('reloaded exact canonical title',lambda:ui.reveal(application,lambda n:n.name=='Clip title' and n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n)==TITLE_NEW))
    ui.type_text(editor,window,TITLE_LATER);ui.click_button(application,'Save title',window)
    ui.wait_for('later exact title committed',lambda:title_input(database,a_id)==TITLE_LATER)
    preserved();before=driver.canonical_state(database)
    ui.reveal_button(ui.wait_for('old B pending title proposal',lambda:review_for_source(application,fixture['b']['name'])),'Accept update',window)
    refusal=ui.wait_for('visible stale title refusal',lambda:ui.find(review,lambda n:'screenplay proposal is stale' in ui.text_of(n)) if (review:=review_for_source(application,fixture['b']['name'])) else None)
    if driver.canonical_state(database)!=before:raise RuntimeError('Stale title proposal acceptance recorded writes')
    evidence['stale_checks'].append({'old_proposal_id':pending[0],'exact_later_title':TITLE_LATER,'no_recorded_write':True,'visible_text':ui.text_of(refusal),'bounds':list(refusal.queryComponent().getExtents(ui.pyatspi.XY_SCREEN))})
    checkpoint('later ordinary title save refuses older targeted acceptance without writes');capture('timeline-title-03-stale-refusal.png')
    ui.reveal_button(ui.wait_for('old title proposal rejection owner',lambda:review_for_source(application,fixture['b']['name'])),'Reject',window)
    ui.wait_for('old title preview rejected',lambda:ui.query(database,"SELECT id FROM propagation_proposals WHERE id=? AND status='rejected'",(pending[0],)))
    evidence['fresh_impacts']=select_cause();Provider.allow_fresh_notes=True
    ui.reveal_button(ui.wait_for('fresh title preview owner',lambda:review_for_source(application,fixture['b']['name'])),'Preview update',window)
    fresh=ui.wait_for('fresh title proposal',lambda:(p if p and p[0]!=pending[0] else None) if (p:=pending_script_review(database)) else None)
    if next(i for i in fresh[1]['timeline_title_current'] if i['node_id']==a_id)['name']!=TITLE_LATER:raise RuntimeError('Fresh proposal omitted later title')
    preserved();evidence['fresh_pending_binding']=fresh[1]
    ui.wait_for('reachable later title evidence',lambda:ui.reveal(application,lambda n:n.name=='Current preview title' and ui.text_of(n)==TITLE_LATER))
    evidence['fresh_visible_title']=ui.wait_for('exact later title glyphs',lambda:driver.visible_paragraph(application,window,TITLE_LATER,'Current preview title'))
    checkpoint('fresh targeted title review preserves manual B until separate explicit acceptance');capture('timeline-title-04-fresh-review.png')
    ui.reveal_button(ui.wait_for('fresh title acceptance owner',lambda:review_for_source(application,fixture['b']['name'])),'Accept update',window)
    accepted=ui.wait_for('explicit selected B replacement',lambda:next((r for r in ui.blocks(database,b_id) if r[1]==TITLE_PREVIEW and r[2]!=saved_b[0][2]),None))
    if ui.blocks(database,a_id)!=original[a_id] or ui.blocks(database,f_id)!=original[f_id] or accepted[3:]!=saved_b[0][3:]:raise RuntimeError('Acceptance changed unrelated text or placement')
    if material(database)['timeline_placement']!=placement or bible_fields(database)!=world or title_input(database,a_id)!=TITLE_LATER:raise RuntimeError('Acceptance changed title placement or Bible canon')
    final=impacts();chosen=next(r for r in final if r['source']==b_id)
    if any(c['dependency_id'].endswith('.timeline_title.'+a_id) for c in chosen['allCauses']):raise RuntimeError('Accepted B retained stale consumed-title cause')
    require_draft(application);driver.bible_field(application,window,RED,'right',align=True)
    ui.wait_for('visible accepted synthetic title replacement',lambda:ui.screenplay_block(application,driver.screenplay_anchor(TITLE_PREVIEW)))
    evidence['explicit_acceptance']={'proposal_id':fresh[0],'saved_block':accepted,'manual_target_preserved_until_acceptance':True,'unrelated_saved_and_draft_preserved':True,'bible_and_placement_unchanged':True,'final_impacts':final}
    if len(Provider.records)!=6 or not all(r['accepted'] for r in Provider.records):raise RuntimeError('Expected four real fixture/two targeted synthetic HTTP calls')
    checkpoint('explicit acceptance changes B alone and refreshes consumed-title memory');capture('timeline-title-05-accepted.png')

def main():
    repo=Path.cwd();output=Path(os.environ['EIDETIC_CAPTURE_DIR']);output.mkdir(parents=True,exist_ok=True)
    state_root=Path(os.environ['RUNNER_TEMP'])/('eidetic-manual-fact-state-'+os.environ.get('EIDETIC_CAPTURE_SCOPE','default'));state_root.mkdir(parents=True,exist_ok=True)
    environment=os.environ.copy();environment['EIDETIC_LAUNCHER_STATE_ROOT']=str(state_root)
    for suffix in ('CACHE','CONFIG','DATA','STATE'):
        p=state_root/'dev'/('xdg-'+suffix.lower());p.mkdir(parents=True,exist_ok=True);environment['XDG_'+suffix+'_HOME']=str(p)
    environment.update(NO_AT_BRIDGE='0',GTK_MODULES='atk-bridge')
    ui.Atspi.set_timeout(1500,1500);ui.DEADLINE=time.monotonic()+540
    original_wait=ui.wait_for
    def bounded(label,check):
        overall=ui.DEADLINE;ui.DEADLINE=min(overall,time.monotonic()+45)
        try:return original_wait(label,check)
        finally:ui.DEADLINE=overall
    ui.wait_for=bounded
    config=repo/'scripts/manual-facts-vite.config.mts';host_log=state_root/'host-private.log';app_log=state_root/'app-private.log'
    host=process=application=window=None
    provider=ThreadingHTTPServer(('127.0.0.1',18080),Provider);threading.Thread(target=provider.serve_forever,daemon=True).start()
    evidence={'status':'failed','application_source_sha':SOURCE,'implementation_source_sha':SOURCE,'application_tree':SOURCE_TREE,'qualification_sha':ui.command('git','rev-parse','HEAD'),'application_binary_sha256':ui.file_hash(repo/'target/debug/eidetic-desktop'),'provider':'labelled synthetic localhost HTTP/SSE through production client','real_model_quality_qualified':False,'DOM_or_IPC_injection':False,'direct_database_writes':False,'stale_checks':[],'checkpoints':[],'qualification_config_sha256':ui.file_hash(config)}
    def checkpoint(stage):
        evidence['stage']=stage;evidence['checkpoints'].append(stage);(output/'capture-evidence.json').write_text(json.dumps(evidence,indent=2)+'\n');print('Manual fact checkpoint: '+stage,flush=True)
    def capture(name):
        pid=int(ui.command('xdotool','getwindowpid',window))
        if process.poll() is not None or os.getpgid(pid)!=process.pid:raise RuntimeError('Native window ownership changed')
        digest=ui.capture_native_window(window,output/name,screen_read=True);evidence.setdefault('captures',[]).append({'file':name,'sha256':digest,'window_id':int(window),'pid':pid})
        display=(output/name).with_suffix('.jpg');subprocess.run(['convert',str(output/name),'-quality','85',str(display)],check=True,timeout=10)
        evidence.setdefault('display_derivatives',[]).append({'file':display.name,'sha256':ui.file_hash(display),'jpeg_quality':85,'original':name,'original_sha256':digest,'lossless_integrity_original_preserved':True})
    try:
        with host_log.open('w') as stream:host=subprocess.Popen([str(repo/'ui/node_modules/.bin/vite'),'dev','--config',str(config),'--port','5173','--host','127.0.0.1','--strictPort'],cwd=repo/'ui',stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
        def ready():
            if host.poll() is not None:raise RuntimeError('Qualifier host exited')
            try:
                with urlopen('http://127.0.0.1:5173/',timeout=2) as response:return response.status==200
            except OSError:return False
        ui.wait_for('labelled native Vite host',ready)
        evidence['served_source_receipts']=[]
        for relative,marker in [('src/lib/components/editor/TimelineTitleEditor.svelte','Save title'),('src/lib/stores/timelineTitleSession.svelte.ts','invalidateScriptContext'),('src/lib/timelineCommandApi.ts','command_timeline_node_name'),('src/lib/components/editor/ScriptTimelineTitleEvidence.svelte','Originally consumed title'),('src/lib/components/editor/ScriptBlockEditor.svelte','Remove block'),('src/lib/components/editor/ScriptBlockRemoval.svelte','Retry same removal'),('src/lib/stores/scriptBlockEditSession.svelte.ts','getOrphanedScriptBlockRemovals'),('src/lib/stores/scriptDocumentProjection.svelte.ts','applyScriptBlockRemovalCommand'),('src/lib/components/editor/ScriptPanel.svelte','ScriptBlockRemoval'),('src/lib/components/editor/ScriptFactReconciliation.svelte','Analyze saved edit'),('src/lib/stores/propagationProposalProjection.svelte.ts','project changed during proposal refresh'),('src/qualification/ManualFactControls.svelte','SYNTHETIC HTTP replies'),('src/qualification/manualFacts.svelte.ts','SYNTHETIC QA transport: Bible detail unavailable.'),('src/lib/stores/bibleGraphNodeProjection.svelte.ts','setBibleGraphFieldProjection'),('src/lib/stores/bibleGraphNodeDetailProjection.svelte.ts','refreshOwnedBibleGraphNodeProjections'),('src/lib/components/sidebar/bible/bibleGraphFieldDrafts.svelte.ts','Saved fact changed while editing'),('src/lib/components/sidebar/bible/BibleGraphPartFields.svelte','Committed Bible fact'),('src/lib/components/sidebar/bible/BibleGraphNodeDetail.svelte','Retry saved facts'),('src/lib/components/sidebar/ArcDetail.svelte','Describe this story arc'),('src/lib/components/editor/ScriptArcEvidence.svelte','Originally omitted arc description'),('src/lib/components/editor/ScriptTimelineNotesEvidence.svelte','Timeline Notes used for this update'),('src/lib/components/editor/ScriptImpactReview.svelte','ScriptTimelineNotesEvidence'),('src/lib/components/editor/scriptImpactNotice.ts','Timeline Notes'),('src/lib/components/editor/contextRequestLifecycle.ts','contextNotes'),('src/lib/components/editor/BeatEditor.svelte','contextRequestLifecycle'),('src/qualification/notesPrompt.svelte.ts','QA held real public context return')]:
            source_file=repo/'ui'/relative
            with urlopen('http://127.0.0.1:5173/'+relative,timeout=10) as response:served=response.read()
            import hashlib
            if marker.encode() not in served:raise RuntimeError('Actual served module lacks admitted marker: '+relative)
            if relative=='src/lib/stores/scriptDocumentProjection.svelte.ts' and b'/src/qualification/manualFacts.svelte.ts' not in served:
                raise RuntimeError('Actual removal transport is not routed through labelled post-commit acknowledgement seam')
            if relative=='src/lib/stores/bibleGraphNodeDetailProjection.svelte.ts' and b'/src/qualification/manualFacts.svelte.ts' not in served:
                raise RuntimeError('Actual served Bible detail read is not routed through labelled fault seam')
            if relative=='src/lib/components/editor/BeatEditor.svelte' and b'/src/qualification/notesPrompt.svelte.ts' not in served:
                raise RuntimeError('Actual selected editor is not routed through labelled real context return seam')
            evidence['served_source_receipts'].append({'module':relative,'source_sha256':ui.file_hash(source_file),'served_transformed_sha256':hashlib.sha256(served).hexdigest(),'marker':marker,'observed':True,'fault_wrapper_routed':b'/src/qualification/manualFacts.svelte.ts' in served if relative=='src/lib/stores/bibleGraphNodeDetailProjection.svelte.ts' else None})
        fixture=json.loads(subprocess.check_output([str(repo/'target/debug/examples/manual_fact_capture_fixture')],env=environment,text=True,timeout=90))
        database=Path(fixture['project_path']).resolve();evidence['public_fixture']=dict(fixture,project_path='<fresh qualification project>')
        if os.environ.get('EIDETIC_CAPTURE_SCOPE')=='arc-description':evidence['verified_fixture_applicability']=verify_arc_fixture(database,fixture)
        if os.environ.get('EIDETIC_CAPTURE_SCOPE') in KNOWN_EMPTY_MODES:
            evidence['verified_fixture_applicability']=verify_two_tag_fixture(database,fixture) if os.environ['EIDETIC_CAPTURE_SCOPE']=='known-empty-second-tag' else verify_arc_fixture(database,fixture)
        if os.environ.get('EIDETIC_CAPTURE_SCOPE')=='ancestor-notes':evidence['verified_fixture_hierarchy']=verify_ancestor_fixture_hierarchy(database,fixture)
        if len(Provider.records)!=4 or not all(r['accepted'] for r in Provider.records):raise RuntimeError('Fixture did not create two genuine synthetic consumer generations')
        checkpoint('public services seed two actual consumers, unconsumed fields and unrelated saved scene')
        with app_log.open('w') as stream:process=subprocess.Popen(['./launcher.sh','--run'],env=environment,stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
        def owned_window():
            if process.poll() is not None:raise RuntimeError('Native launcher exited '+str(process.returncode))
            result=subprocess.run(['xdotool','search','--onlyvisible','--name','^Eidetic$'],capture_output=True,text=True,timeout=5)
            for candidate in result.stdout.splitlines():
                pid=int(ui.command('xdotool','getwindowpid',candidate))
                if os.getpgid(pid)==process.pid:return candidate,pid
        window,pid=ui.wait_for('owned native window',owned_window);ui.NATIVE_WINDOW_PID=pid
        evidence.update(window_id=int(window),window_pid=pid,application_binary_sha256=ui.file_hash(Path('/proc/'+str(pid)+'/exe')))
        ui.command('xdotool','windowsize','--sync',window,'1920','1440');evidence['native_window_geometry']=ui.command('xdotool','getwindowgeometry','--shell',window)
        application=ui.wait_for('native accessibility app',lambda:next((a for a in ui.pyatspi.Registry.getDesktop(0) if a.get_process_id()==pid),None))
        ui.open_project_chooser(application,window);ui.click_button(application,database.parent.name,window,prefix=True);ui.choose_mode(application,'Script',window)
        driver.enlarge_script_pane(application,window);ui.reveal_button(application,'Zoom to Fit (Ctrl+0)',window)
        for _ in range(3):ui.reveal_button(application,'+',window)
        if os.environ.get('EIDETIC_CAPTURE_SCOPE')=='initial-detail-retry':
            evidence['qualification_scope']='targeted uncached initial detail failure, ordinary Retry, pending read and verified recovery only; predecessor full reconciliation qualification remains separately source-bound'
            qualify_initial_detail_retry(application,window,database,capture,checkpoint,evidence)
            evidence['status']='passed';checkpoint('complete frozen-source targeted initial detail Retry')
            return
        ui.click_button(application,'AI',window);ui.click_button(application,'Save & Connect',window)
        ui.wait_for('connected synthetic provider',lambda:driver.visible_text(application,'Connected'))
        if os.environ.get('EIDETIC_CAPTURE_SCOPE','').startswith('timeline-title'):
            evidence['qualification_scope']='ordinary exact clip title edit/save/navigation/reload, canonical consumed-title impact, retained manual screenplay/draft/Bible/placement, pending targeted preview, stale refusal and separate acceptance; synthetic HTTP only'
            qualify_timeline_title(application,window,database,fixture,capture,checkpoint,evidence)
            evidence['status']='passed';checkpoint('complete frozen-source timeline title memory')
            return
        if os.environ.get('EIDETIC_CAPTURE_SCOPE','').startswith('screenplay-removal'):
            evidence['qualification_scope']='ordinary exact manual edits, explicit saved-block confirmation/cancel, labelled post-commit lost acknowledgement, exact replay after canonical disappearance/navigation, stale downstream refusal, fresh targeted preview and separate acceptance; synthetic HTTP only'
            qualify_screenplay_removal(application,window,database,fixture,capture,checkpoint,evidence)
            evidence['status']='passed';checkpoint('complete frozen-source saved screenplay removal')
            return
        if os.environ.get('EIDETIC_CAPTURE_SCOPE')=='notes-prompt':
            evidence['qualification_scope']='exact same-node Notes prompt refresh and held older actual public context return refusal, ordinary clear/restore, saved text and unrelated draft preservation; no new inference or real-model quality claim'
            qualify_notes_prompt(application,window,database,fixture,capture,checkpoint,evidence)
            evidence['status']='passed';checkpoint('complete frozen-source exact Notes prompt custody')
            return
        if os.environ.get('EIDETIC_CAPTURE_SCOPE')=='arc-description':
            evidence['qualification_scope']='ordinary ArcDetail exact Description entry on actual tagged Scenes; known-empty applicability, unrelated control, manual saved text and draft preservation, pending synthetic review, clear/restore refusal and explicit chosen-block acceptance'
            qualify_arc_description(application,window,database,fixture,capture,checkpoint,evidence)
            evidence['status']='passed';checkpoint('complete frozen-source arc Description applicability review')
            return
        if os.environ.get('EIDETIC_CAPTURE_SCOPE') in KNOWN_EMPTY_MODES:
            evidence['qualification_scope']=os.environ['EIDETIC_CAPTURE_SCOPE']+': real native ArcDetail edit and production Preview/Accept; exact known-empty or absence binding, saved material/draft preservation; synthetic model replies only'
            qualify_known_empty_arc_preview(application,window,database,fixture,capture,checkpoint,evidence)
            evidence['status']='passed';checkpoint('complete frozen-source '+os.environ['EIDETIC_CAPTURE_SCOPE'])
            return
        if os.environ.get('EIDETIC_CAPTURE_SCOPE')=='ancestor-notes':
            evidence['qualification_scope']='ordinary consumed ancestor Notes edit; two exact dependent Scenes, unrelated Act saved text/draft preservation, pending synthetic review, stale ABA refusal and explicit selected-block acceptance only'
            qualify_ancestor_notes(application,window,database,fixture,capture,checkpoint,evidence)
            evidence['status']='passed';checkpoint('complete frozen-source ancestor Notes targeted review')
            return
        if os.environ.get('EIDETIC_CAPTURE_SCOPE')=='timeline-notes':
            evidence['qualification_scope']='ordinary selected clip Notes edit, explicit Notes cause selection, pending synthetic preview, Notes ABA refusal and explicit fresh selected-block acceptance; ancestor/sibling Notes and real-model quality unqualified'
            qualify_timeline_notes(application,window,database,fixture,capture,checkpoint,evidence)
            evidence['status']='passed';checkpoint('complete frozen-source timeline Notes targeted review')
            return
        qualify(application,window,database,fixture,capture,checkpoint,evidence)
        qualify_save_refresh(application,window,database,capture,checkpoint,evidence)
        # The draft-preservation action sequence has completed. A fresh native
        # process now checks saved material/proposal reconstruction only; this
        # does not qualify unsaved draft lifetime or project-switch recovery.
        saved_before_reopen=material(database)
        proposals_before_reopen=ui.query(database,'SELECT * FROM propagation_proposals ORDER BY rowid')
        history_before_reopen=driver.canonical_state(database)
        os.killpg(process.pid,signal.SIGTERM)
        try:process.wait(timeout=10)
        except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait(timeout=5)
        ui.NATIVE_WINDOW_PID=None
        with app_log.open('a') as stream:process=subprocess.Popen(['./launcher.sh','--run'],env=environment,stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
        window,pid=ui.wait_for('reopened owned native window',owned_window);ui.NATIVE_WINDOW_PID=pid
        if ui.file_hash(Path('/proc/'+str(pid)+'/exe'))!=evidence['application_binary_sha256']:raise RuntimeError('Reopen changed native binary custody')
        ui.command('xdotool','windowsize','--sync',window,'1920','1440')
        application=ui.wait_for('reopened native accessibility app',lambda:next((a for a in ui.pyatspi.Registry.getDesktop(0) if a.get_process_id()==pid),None))
        ui.open_project_chooser(application,window);ui.click_button(application,database.parent.name,window,prefix=True);ui.choose_mode(application,'Script',window)
        driver.enlarge_script_pane(application,window)
        ui.click_button(application,'Bible',window)
        reconstructed=ui.wait_for('reconstructed accepted fact article',lambda:fact_article(application,'accepted'))
        reconstructed_view=visible_proposal(application,window,BLUE,status='accepted')
        if saved_before_reopen!=material(database) or proposals_before_reopen!=ui.query(database,'SELECT * FROM propagation_proposals ORDER BY rowid') or history_before_reopen!=driver.canonical_state(database):raise RuntimeError('Reopen changed saved material, immutable proposal/binding or history')
        evidence['reopened_persisted_review']={'same_project':True,'native_process_restarted':True,'saved_material_and_proposal_binding_columns_unchanged':True,'canonical_history_unchanged':True,'accepted_proposal_visible':reconstructed_view,'scope':'saved material/proposal reconstruction after draft-preservation sequence; unsaved draft lifetime and project-switch recovery unqualified'}
        checkpoint('same project reopened; persisted accepted/rejected fact review and saved material reconstruct unchanged')
        capture('manual-fact-10-reopened.png')
        if len(Provider.records)!=9 or not all(r['accepted'] and not r['real_model'] for r in Provider.records):raise RuntimeError('Missing exact synthetic HTTP phases')
        evidence['status']='passed';checkpoint('complete frozen-source native saved edit to Bible reconciliation')
    except Exception as error:
        evidence['failure']=str(error)[:1000]
        if application:
            try:evidence['native_diagnostics']=[{'role':n.getRoleName(),'name':n.name,'text':ui.text_of(n),'bounds':tuple(n.queryComponent().getExtents(ui.pyatspi.XY_SCREEN))} for n in ui.walk(application) if ui.visible(n)][:200]
            except Exception as diagnostic:evidence['diagnostic_error']=str(diagnostic)
        if window:
            try:capture('manual-fact-failure.png')
            except Exception as capture_error:evidence['capture_error']=str(capture_error)
        raise
    finally:
        evidence['provider_records']=Provider.records;evidence['typed_text_observations']=ui.TYPED_TEXT_OBSERVATIONS
        evidence['accessibility_retirements']=ui.ACCESSIBILITY_RETIREMENTS
        (output/'capture-evidence.json').write_text(json.dumps(evidence,indent=2)+'\n')
        # Readable receipt survives artifact-transfer restrictions; fixtures
        # contain only authored QA text. Originals remain the uploaded PNGs.
        keys=('status','application_source_sha','application_tree','qualification_sha','application_binary_sha256','failure','checkpoints','captures','display_derivatives','stale_checks','saved_screenplay_placement_bible_and_unrelated_draft_preserved','ordinary_notes_edit','ordinary_notes_clear')
        summary={key:evidence[key] for key in keys if key in evidence}
        summary['synthetic_fixture_calls']=len(Provider.records)
        summary['synthetic_calls_accepted']=all(r['accepted'] for r in Provider.records)
        summary['real_model_quality_qualified']=False
        for key in ('original_context','fresh_context','after_late_return','restored_context'):
            if key in evidence:summary[key]=evidence[key]
        print('NATIVE_CAPTURE_RECEIPT '+json.dumps(summary,ensure_ascii=False),flush=True)
        for p in (process,host):
            if p and p.poll() is None:
                os.killpg(p.pid,signal.SIGTERM)
                try:p.wait(timeout=10)
                except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL)
        provider.shutdown();provider.server_close()
        for private,name in [(app_log,'app-sanitized.log'),(host_log,'qualification-host-sanitized.log')]:
            raw=private.read_text(errors='replace') if private.exists() else 'No launch log.'
            (output/name).write_text(ui.sanitized_log(raw,[(str(state_root),'<qualification state>'),(str(repo),'<repo>')]))
if __name__=='__main__':main()
