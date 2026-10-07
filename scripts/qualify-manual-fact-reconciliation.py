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
SOURCE='e5a09ede159a6511694896fdb5d4382e51f4418d'
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


class Provider(ui.FixtureProvider):
    records=[]
    def do_POST(self):
        size=int(self.headers.get('Content-Length','0'))
        if self.path!='/v1/chat/completions' or not 0<size<=512_000:
            self.send_error(400);return
        body=json.loads(self.rfile.read(size));messages=body.get('messages',[])
        system=next((m['content'] for m in messages if m.get('role')=='system'),'')
        user=next((m['content'] for m in messages if m.get('role')=='user'),'')
        kind='fact' if 'Analyze one saved manual screenplay edit' in system else 'recap' if user.startswith('Generate a scene recap for this screenplay beat:') else 'generation'
        with self.records_lock:
            phase=sum(r.get('accepted') and r['kind']==kind for r in self.records)
            valid=body.get('stream') is True
            if kind=='generation':valid=valid and phase<2 and RED in user and 'UNCONSUMED' not in user
            elif kind=='recap':valid=valid and phase<2 and (GENERATED_A if phase==0 else GENERATED_B).strip() in user
            else:valid=valid and phase<5 and fact_prompt(user,phase)
            record={'kind':kind,'phase':phase,'accepted':valid,'real_model':False,'synthetic':True,'exact_system_prompt':system,'exact_user_prompt':user}
            self.records.append(record)
        if not valid:self.send_error(422,'Synthetic qualification prompt/phase mismatch');return
        text=(GENERATED_A if phase==0 else GENERATED_B) if kind=='generation' else 'Synthetic recap: Mara carries red.' if kind=='recap' else json.dumps({'value':GREEN if phase==4 else BLUE,'rationale':'SYNTHETIC QA response: human acceptance required; no model quality claim.'})
        record['synthetic_response']=text
        parts=[text[:len(text)//2],text[len(text)//2:]]
        data=''.join('data: '+json.dumps({'choices':[{'delta':{'content':p}}]})+'\n\n' for p in parts).encode()+b'data: [DONE]\n\n'
        self.send_response(200);self.send_header('Content-Type','text/event-stream');self.send_header('Content-Length',str(len(data)));self.end_headers();self.wfile.write(data)


def receipt(application):
    node=ui.find(application,lambda n:n.name=='QA saved-edit receipt' and n.getRole() in (ui.pyatspi.ROLE_TEXT,ui.pyatspi.ROLE_ENTRY))
    if not node:return None
    # The receipt can exceed the generic text helper's short diagnostic bound.
    text=node.queryText();value=json.loads(text.getText(0,text.characterCount))
    if not value['fixture'].startswith('qualifier-only public commands'):raise RuntimeError('Missing explicit QA label')
    if value['error']:raise RuntimeError('Public QA command failure: '+value['error'])
    return value


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


def main():
    repo=Path.cwd();output=Path(os.environ['EIDETIC_CAPTURE_DIR']);output.mkdir(parents=True,exist_ok=True)
    state_root=Path(os.environ['RUNNER_TEMP'])/'eidetic-manual-fact-state';state_root.mkdir(parents=True,exist_ok=True)
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
    evidence={'status':'failed','application_source_sha':SOURCE,'application_tree':'be0acb7d8db1402a57939dda16820ac7558126a5','qualification_sha':ui.command('git','rev-parse','HEAD'),'application_binary_sha256':ui.file_hash(repo/'target/debug/eidetic-desktop'),'provider':'labelled synthetic localhost HTTP/SSE through production client','real_model_quality_qualified':False,'DOM_or_IPC_injection':False,'direct_database_writes':False,'stale_checks':[],'checkpoints':[],'qualification_config_sha256':ui.file_hash(config)}
    def checkpoint(stage):
        evidence['stage']=stage;evidence['checkpoints'].append(stage);(output/'capture-evidence.json').write_text(json.dumps(evidence,indent=2)+'\n');print('Manual fact checkpoint: '+stage,flush=True)
    def capture(name):
        pid=int(ui.command('xdotool','getwindowpid',window))
        if process.poll() is not None or os.getpgid(pid)!=process.pid:raise RuntimeError('Native window ownership changed')
        digest=ui.capture_native_window(window,output/name,screen_read=True);evidence.setdefault('captures',[]).append({'file':name,'sha256':digest,'window_id':int(window),'pid':pid})
    try:
        with host_log.open('w') as stream:host=subprocess.Popen([str(repo/'ui/node_modules/.bin/vite'),'dev','--config',str(config),'--port','5173','--host','127.0.0.1','--strictPort'],cwd=repo/'ui',stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
        def ready():
            if host.poll() is not None:raise RuntimeError('Qualifier host exited')
            try:
                with urlopen('http://127.0.0.1:5173/',timeout=2) as response:return response.status==200
            except OSError:return False
        ui.wait_for('labelled native Vite host',ready)
        evidence['served_source_receipts']=[]
        for relative,marker in [('src/lib/components/editor/ScriptFactReconciliation.svelte','Analyze saved edit'),('src/lib/stores/propagationProposalProjection.svelte.ts','project changed during proposal refresh'),('src/qualification/ManualFactControls.svelte','SYNTHETIC HTTP replies'),('src/qualification/manualFacts.svelte.ts','SYNTHETIC QA transport: Bible detail unavailable.'),('src/lib/stores/bibleGraphNodeProjection.svelte.ts','setBibleGraphFieldProjection'),('src/lib/stores/bibleGraphNodeDetailProjection.svelte.ts','refreshOwnedBibleGraphNodeProjections'),('src/lib/components/sidebar/bible/bibleGraphFieldDrafts.svelte.ts','Saved fact changed while editing'),('src/lib/components/sidebar/bible/BibleGraphPartFields.svelte','Committed Bible fact')]:
            source_file=repo/'ui'/relative
            with urlopen('http://127.0.0.1:5173/'+relative,timeout=10) as response:served=response.read()
            import hashlib
            if marker.encode() not in served:raise RuntimeError('Actual served module lacks admitted marker: '+relative)
            if relative=='src/lib/stores/bibleGraphNodeDetailProjection.svelte.ts' and b'/src/qualification/manualFacts.svelte.ts' not in served:
                raise RuntimeError('Actual served Bible detail read is not routed through labelled fault seam')
            evidence['served_source_receipts'].append({'module':relative,'source_sha256':ui.file_hash(source_file),'served_transformed_sha256':hashlib.sha256(served).hexdigest(),'marker':marker,'observed':True,'fault_wrapper_routed':b'/src/qualification/manualFacts.svelte.ts' in served if relative=='src/lib/stores/bibleGraphNodeDetailProjection.svelte.ts' else None})
        fixture=json.loads(subprocess.check_output([str(repo/'target/debug/examples/manual_fact_capture_fixture')],env=environment,text=True,timeout=90))
        database=Path(fixture['project_path']).resolve();evidence['public_fixture']=dict(fixture,project_path='<fresh qualification project>')
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
        ui.click_button(application,'AI',window);ui.click_button(application,'Save & Connect',window)
        ui.wait_for('connected synthetic provider',lambda:driver.visible_text(application,'Connected'))
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
        (output/'capture-evidence.json').write_text(json.dumps(evidence,indent=2)+'\n')
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
