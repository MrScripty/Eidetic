#!/usr/bin/env python3
"""Native lifecycle addendum: real component/store effects, labelled test-build seams.

Reachable sidebar removal/remount uses ordinary application controls. A compiled
QA wrapper controls final disposal without clearing selection and holds completion
of actual production domain reads; rejection is a labelled synthetic error.
No DOM/IPC injection or database writes. The ordinary authoring setup retains
manual B, unsaved F and a pending proposal; its HTTP/SSE replies are synthetic.
"""
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import time
from urllib.request import urlopen

spec=importlib.util.spec_from_file_location('recall_capture',Path(__file__).with_name('qualify-bible-recall.py'))
driver=importlib.util.module_from_spec(spec)
spec.loader.exec_module(driver)
ui=driver.ui
SOURCE='4c468d6386bcad44117fe7e4439202cdde52a0f7'
COPPER="Mara's umbrella is copper."
NOTICE='Facts changed. Recall again'


def receipt(application):
    field=ui.find(application,lambda n:n.name=='QA lifecycle receipt' and n.getRole() in (ui.pyatspi.ROLE_TEXT,ui.pyatspi.ROLE_ENTRY))
    if field is None: return None
    value=json.loads(ui.text_of(field))
    if not value['fixture'].startswith('qualifier-only native lifecycle controls'):
        raise RuntimeError('Native QA receipt is not labelled')
    return value


def wait_receipt(application,description,predicate):
    def read():
        value=receipt(application)
        return value if value is not None and predicate(value) else None
    return ui.wait_for(description,read)


def observe(application,label,predicate,duration=0.75):
    """Require sustained client state after effects/microtasks, not one lucky frame."""
    until=time.monotonic()+duration
    observations=[]
    while time.monotonic()<until:
        value=receipt(application)
        if value is None or not predicate(value):
            raise RuntimeError('Native lifecycle invariant failed: '+label)
        observations.append(value)
        time.sleep(0.1)
    return {'label':label,'duration_seconds':duration,'samples':observations}


def clean(value):
    state=value['productionState']
    return state['anchor']=='qualification.mara' and not state['pending'] and not state['invalidated'] and state['error'] is None and state['projectionVersion'] is None


def open_right_recall(application,window):
    control=ui.wait_for('remounted right production recall summary',lambda:driver.recall_disclosure(application))
    ui.click_control(control,window)


def preserved(application,database,original,pending,history=None):
    driver.require_preserved(database,original,history)
    ui.wait_for('preserved exact unsaved F draft',lambda:ui.editable(application,driver.DRAFT))
    current=ui.query(database,'SELECT * FROM propagation_proposals ORDER BY rowid')
    if current != pending:
        raise RuntimeError('Lifecycle qualification changed pending proposal material/status')


def qualify(application,window,database,original,capture,checkpoint,evidence):
    if evidence['application_source_sha'] != SOURCE:
        raise RuntimeError('Lifecycle application source is not frozen successor')
    evidence['lifecycle_test_build']={
        'label':'qualifier-only compiled lifecycle wrappers; exact production BibleRecall/store/transport',
        'seams':['wrap production inspector with native show/dispose controls','hold completion after actual production domain read','labelled synthetic delayed completion error'],
        'DOM_or_IPC_injection':False,'database_writes':False,'synthetic_recall_values':False,
        'original_evidence_source':'8bd0da0daa22897996b8e60976bf2de63fffe4eb'}
    checks=[]
    evidence['native_lifecycle_checks']=checks
    pending=ui.query(database,'SELECT * FROM propagation_proposals ORDER BY rowid')
    if not pending: raise RuntimeError('No canonical pending proposal to preserve')
    history=driver.canonical_state(database)
    initial=wait_receipt(application,'both production inspector hosts',lambda r:r['hosts']==2 and r['productionState']['projectionVersion'] is not None)
    version=initial['productionState']['projectionVersion']
    # This is the normal app lifecycle: Sidebar tab destroys only its inline inspector.
    ui.click_button(application,'Arcs',window)
    wait_receipt(application,'one surviving inspector',lambda r:r['hosts']==1)
    checks.append(observe(application,'ordinary sidebar removal preserves the other inspector',lambda r:r['hosts']==1 and r['productionState']['projectionVersion']==version and not r['productionState']['invalidated']))
    ui.wait_for('surviving actual related fact',lambda:driver.visible_text(application,'environment.weather: Rain'))
    ui.click_button(application,'Bible',window)
    wait_receipt(application,'both inspectors remounted on unchanged selection',lambda r:r['hosts']==2)
    checks.append(observe(application,'ordinary same-anchor sidebar remount has no false notice',lambda r:r['hosts']==2 and r['productionState']['projectionVersion']==version and not r['productionState']['invalidated']))
    preserved(application,database,original,pending,history)
    checkpoint('ordinary concurrent inspector removal/remount preserves shared read and manual work')
    capture('eidetic-bible-recall-lifecycle-concurrent.png')

    for failure in (False,True):
        ui.click_button(application,'QA hold next recall completion',window)
        ui.reveal_button(application,'Recall related story facts',window)
        held=wait_receipt(application,'actual domain read completion held behind QA control',lambda r:r['held'] and r['productionState']['pending'])
        if json.loads(held['actualReadQuery'])['anchor_node_id']!='qualification.mara':
            raise RuntimeError('Held response query does not belong to selected actual anchor')
        checks.append({'label':'held actual read before final disposal','failure':failure,'receipt':held})
        ui.click_button(application,'QA dispose all recall inspectors',window)
        wait_receipt(application,'last production inspector effects disposed',lambda r:not r['inspectorsVisible'] and clean(r))
        ui.click_button(application,'QA remount recall inspectors',window)
        wait_receipt(application,'clean same-anchor native remount',lambda r:r['inspectorsVisible'] and clean(r))
        open_right_recall(application,window)
        ui.click_button(application,'QA release labelled synthetic error' if failure else 'QA release actual read success',window)
        checks.append(observe(application,'final disposal/remount rejects delayed '+('synthetic error' if failure else 'actual success'),lambda r:not r['held'] and clean(r)))
        preserved(application,database,original,pending,history)
        checkpoint('actual last-inspector effects revoke delayed '+('labelled synthetic error' if failure else 'actual read success'))
        capture('eidetic-bible-recall-lifecycle-delayed-'+('error' if failure else 'success')+'.png')

    # Genuine canonical fact Save through ordinary app controls, with actual evidence active.
    driver.recall_time(application,window,'1000')
    wait_receipt(application,'fresh real recall before canonical mutation',lambda r:r['productionState']['projectionVersion'] is not None and not r['productionState']['pending'])
    field=ui.wait_for('existing exact amber Bible field',lambda:ui.reveal(application,lambda n:n.getState().contains(ui.pyatspi.STATE_EDITABLE) and ui.text_of(n)==driver.AMBER))
    ui.type_text(field,window,COPPER)
    save=ui.wait_for('associated canonical fact Save',lambda:driver.save_fact_control(application,field))
    before=driver.fact(database)
    ui.click_control(save,window)
    after=ui.wait_for('exact canonical copper fact write',lambda:driver.fact(database) if driver.fact(database)[0]==COPPER and driver.fact(database)[1]!=before[1] else None)
    wait_receipt(application,'genuine event invalidation',lambda r:r['productionState']['invalidated'] and r['productionState']['projectionVersion'] is None)
    ui.click_button(application,'QA dispose all recall inspectors',window)
    wait_receipt(application,'genuine invalidation retained without an inspector',lambda r:not r['inspectorsVisible'] and r['productionState']['invalidated'])
    ui.click_button(application,'QA remount recall inspectors',window)
    wait_receipt(application,'genuine invalidation retained on same-anchor remount',lambda r:r['inspectorsVisible'] and r['productionState']['invalidated'])
    open_right_recall(application,window)
    ui.wait_for('actual native fact-change notice after remount',lambda:driver.visible_text(application,NOTICE))
    checks.append(observe(application,'genuine mutation survives actual final disposal/remount',lambda r:r['productionState']['invalidated'] and r['productionState']['projectionVersion'] is None))
    evidence['lifecycle_canonical_fact_edit']={'before':before,'after':after,'exact_before':driver.AMBER,'exact_after':COPPER}
    preserved(application,database,original,pending)
    checkpoint('genuine canonical mutation survives final disposal/remount and keeps pending manual work')
    capture('eidetic-bible-recall-lifecycle-genuine-invalidation.png')
    changed_history=driver.canonical_state(database)
    driver.recall_time(application,window,'1000')
    ui.wait_for('explicit real current copper evidence',lambda:driver.visible_text(application,'profile.tagline: '+COPPER))
    wait_receipt(application,'fresh read clears genuine mutation notice',lambda r:not r['productionState']['invalidated'] and r['productionState']['projectionVersion'] is not None)
    preserved(application,database,original,pending,changed_history)
    checks.append(observe(application,'explicit real fresh recall alone clears mutation notice',lambda r:not r['productionState']['invalidated'] and r['productionState']['projectionVersion'] is not None))
    evidence['pending_proposal_rows_before_and_after']=pending
    evidence['retained_pending_proposal_unchanged']=True
    checkpoint('native lifecycle addendum complete; exact screenplay, draft and pending proposal preserved')
    capture('eidetic-bible-recall-lifecycle-fresh.png')


def main():
    repo=Path.cwd()
    output=Path(os.environ['EIDETIC_CAPTURE_DIR'])
    output.mkdir(parents=True,exist_ok=True)
    config=repo/'scripts/recall-lifecycle-vite.config.mts'
    log=Path(os.environ['RUNNER_TEMP'])/'recall-lifecycle-vite-private.log'
    process=None
    try:
        with log.open('w') as stream:
            args=[str(repo/'ui/node_modules/.bin/vite'),'dev','--config',str(config),'--port','5173','--host','127.0.0.1','--strictPort']
            process=subprocess.Popen(args,cwd=repo/'ui',stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
        def ready():
            if process.poll() is not None:raise RuntimeError('Qualifier-only Vite host exited')
            try:
                with urlopen('http://127.0.0.1:5173/',timeout=2) as response:
                    return response.status==200
            except OSError:return False
        ui.wait_for('isolated lifecycle Vite host',ready)
        host={'label':'explicit qualifier-only Vite host reused by unchanged desktop launcher',
            'pid':process.pid,'args':['vite','dev','--config','scripts/recall-lifecycle-vite.config.mts','--port','5173','--host','127.0.0.1','--strictPort'],
            'config_sha256':ui.file_hash(config),'cwd':'<repo>/ui','bind_url':'http://127.0.0.1:5173',
            'no_alternate_runtime_or_dependency':True}
        driver.main(qualify,host)
    finally:
        if process and process.poll() is None:
            os.killpg(process.pid,signal.SIGTERM)
            try:process.wait(timeout=10)
            except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL)
        raw=log.read_text(errors='replace') if log.exists() else 'No lifecycle Vite host started.'
        (output/'qualification-host-sanitized.log').write_text(ui.sanitized_log(raw,[(str(repo),'<repo>')]))


if __name__=='__main__':main()
