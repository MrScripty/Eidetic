"""No-display regression checks for the synthetic provider's exact fact custody."""
import ast
import json
import subprocess
from pathlib import Path
import unittest

source=Path(__file__).with_name('qualify-manual-fact-reconciliation.py').read_text()
module=ast.parse(source)
# Load only constants and the pure prompt predicate, without pyatspi/native imports.
selected=[n for n in module.body if isinstance(n,ast.Assign) and all(isinstance(t,ast.Name) and t.id in {'RED','BLUE','SAVED','DRAFT','BIBLE_DRAFT','MOTIVATION_DRAFT','NOTES','GENERATED_B','ANCESTOR_OLD','ANCESTOR_MANUAL','UNRELATED_ACT_OLD','ARC_NEW','ARC_PREVIEW','ARC_UNRELATED'} for t in n.targets) or isinstance(n,ast.FunctionDef) and n.name in ('fact_prompt','notes_prompt','context_prompt_matches','ancestor_notes_prompt','arc_description_prompt')]
namespace={'json':json};exec(compile(ast.Module(body=selected,type_ignores=[]),'<pure qualifier>','exec'),namespace)

class NativeInputTests(unittest.TestCase):
    def test_native_key_steps_preserve_exact_unicode_whitespace_and_newlines(self):
        helper=ast.parse(Path(__file__).with_name('manual-fact-native-helpers.py').read_text())
        function=next(n for n in helper.body if isinstance(n,ast.FunctionDef) and n.name=='native_input_steps')
        local={};exec(compile(ast.Module(body=[function],type_ignores=[]),'<native input>','exec'),local)
        text=namespace['SAVED'];steps=local['native_input_steps'](text)
        restored=''.join(value if kind=='type' else '\n' if kind=='newline' else chr(int(value,16)) for kind,value in steps)
        self.assertEqual(restored,text)
        self.assertIn(('unicode','2014'),steps)
        self.assertIn(('unicode','96e8'),steps)

class PromptTests(unittest.TestCase):
    def evidence(self):
        return {'text':namespace['SAVED'],'before_revision_event_id':'before','revision_event_id':'saved','facts':[{'field_id':'qualification.mara.tagline','text':namespace['RED'],'consumed_text':namespace['RED']}]}
    def test_exact_one_consumed_field(self):
        self.assertTrue(namespace['fact_prompt'](json.dumps(self.evidence()),0))
    def test_other_field_and_missing_changed_revision_refuse(self):
        e=self.evidence();e['facts'][0]['field_id']='qualification.eli.tagline'
        self.assertFalse(namespace['fact_prompt'](json.dumps(e),0))
        e=self.evidence();e['revision_event_id']='before'
        self.assertFalse(namespace['fact_prompt'](json.dumps(e),0))
    def test_unselected_field_or_draft_leak_refuses(self):
        for extra in [{'text':'UNCONSUMED'}, {'text':namespace['DRAFT']}]:
            e=self.evidence();e['facts'].append(extra)
            self.assertFalse(namespace['fact_prompt'](json.dumps(e),0))
    def test_unsaved_bible_drafts_cannot_leak_even_outside_selected_fact(self):
        for draft in (namespace['BIBLE_DRAFT'],namespace['MOTIVATION_DRAFT']):
            e=self.evidence();e['unexpected_local_draft']=draft
            self.assertFalse(namespace['fact_prompt'](json.dumps(e),0))
    def test_later_proposal_uses_new_current_and_original_consumed_fact(self):
        e=self.evidence();e['facts'][0]['text']=namespace['BLUE']
        self.assertTrue(namespace['fact_prompt'](json.dumps(e),4))
        self.assertFalse(namespace['fact_prompt'](json.dumps(e),3))
    def test_saved_whitespace_is_not_normalized(self):
        e=self.evidence();e['text']=e['text'].strip()
        self.assertFalse(namespace['fact_prompt'](json.dumps(e),0))

class NotesPromptTests(unittest.TestCase):
    def prompt(self):
        return 'CURRENT SELECTED CLIP NOTES:\n'+namespace['NOTES']+'\n'+namespace['GENERATED_B']+namespace['RED']
    def test_targeted_preview_receives_exact_saved_notes_and_original_target(self):
        self.assertTrue(namespace['notes_prompt']('Draft a targeted screenplay update',self.prompt()))
        self.assertFalse(namespace['notes_prompt']('ordinary generation',self.prompt()))
    def test_normalized_notes_missing_target_and_unsaved_draft_leaks_refuse(self):
        for text in [self.prompt().replace(namespace['NOTES'],namespace['NOTES'].strip()),self.prompt().replace(namespace['GENERATED_B'],''),self.prompt()+namespace['DRAFT']]:
            self.assertFalse(namespace['notes_prompt']('Draft a targeted screenplay update',text))

class ActualContextPromptTests(unittest.TestCase):
    def test_exact_authored_notes_with_unicode_and_whitespace(self):
        notes=namespace['NOTES'];user='SCENE: SCENE B\nSCENE NOTES:\n'+notes+'\n\nCONTEXT HIERARCHY:\n'
        self.assertTrue(namespace['context_prompt_matches'](user,notes))
        self.assertFalse(namespace['context_prompt_matches'](user,notes.strip()))
    def test_draft_leak_and_stale_notes_refuse(self):
        user='SCENE NOTES:\nold saved notes\n\n'
        self.assertFalse(namespace['context_prompt_matches'](user,namespace['NOTES']))
        self.assertFalse(namespace['context_prompt_matches'](user+namespace['DRAFT'],'old saved notes'))



class AncestorNotesPromptTests(unittest.TestCase):
    def prompt(self):
        return ('CURRENT CONSUMED ANCESTOR NOTES (owned.act):\n'+namespace['NOTES']+'\n'
            +'TARGET BLOCK TO UPDATE:\n'+namespace['ANCESTOR_MANUAL']+namespace['RED'])
    def test_exact_ancestor_and_saved_manual_target_are_admitted(self):
        self.assertTrue(namespace['ancestor_notes_prompt']('targeted screenplay update',self.prompt()))
    def test_trimmed_notes_missing_manual_target_and_unrelated_or_draft_leak_are_refused(self):
        for text in [self.prompt().replace(namespace['NOTES'],namespace['NOTES'].strip()),
            self.prompt().replace(namespace['ANCESTOR_MANUAL'],namespace['GENERATED_B']),
            self.prompt()+namespace['UNRELATED_ACT_OLD'],self.prompt()+namespace['DRAFT']]:
            self.assertFalse(namespace['ancestor_notes_prompt']('targeted screenplay update',text))

class CanonicalImpactReceiptTests(unittest.TestCase):
    def test_actual_read_seam_retains_exact_ancestor_and_existing_review_causes(self):
        # Execute the actual QA public-read wrapper with a mixed projection. Only
        # the transport/state primitives are replaced; its filtering is unchanged.
        root=Path(__file__).resolve().parents[1]
        script=r'''
const fs=require('node:fs'),vm=require('node:vm');
const ts=require('./ui/node_modules/typescript');
const causes=[
  {dependency_id:'generation.event.ancestor_notes.owned-act',input:{kind:'timeline_node',node_id:'owned-act'},input_excerpt:'  exact — 雨.\n\n  '},
  {dependency_id:'generation.event.timeline_notes',input:{kind:'timeline_node',node_id:'scene'}},
  {dependency_id:'generation.event.field',input:{kind:'bible_field'}},
  {dependency_id:'generation.event.edge',input:{kind:'bible_edge'}},
  {dependency_id:'generation.event.arc',input:{kind:'story_arc_field'}},
  {dependency_id:'generation.event.node',input:{kind:'timeline_node',node_id:'other'}}
];
const projection={payload:{segments:[{segment:{source_node_id:'scene',id:'segment'},impact:{needs_review:true,causes}}]}};
const exports={};
const context={exports,$state:x=>x,require:name=>name==='svelte'?{untrack:f=>f()}:name.endsWith('projectionApi.js')?{getScriptDocumentProjection:async()=>projection}:{}};
const source=fs.readFileSync('ui/src/qualification/manualFacts.svelte.ts','utf8');
vm.runInNewContext(ts.transpileModule(source,{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022}}).outputText,context);
exports.readImpacts().then(()=>process.stdout.write(exports.receipt())).catch(error=>{console.error(error);process.exitCode=1;});
'''
        result=subprocess.run(['node','-e',script],cwd=root,text=True,capture_output=True,check=True)
        receipt=json.loads(result.stdout)
        self.assertEqual(receipt['error'],'')
        row=json.loads(receipt['impacts'])[0]
        self.assertEqual(row['source'],'scene')
        self.assertTrue(row['needsReview'])
        self.assertEqual([c['dependency_id'] for c in row['causes']],[
            'generation.event.ancestor_notes.owned-act','generation.event.timeline_notes',
            'generation.event.field','generation.event.edge','generation.event.arc'])
        self.assertEqual(row['causes'][0]['input'],{'kind':'timeline_node','node_id':'owned-act'})
        self.assertEqual(row['causes'][0]['input_excerpt'],'  exact — 雨.\n\n  ')


class ArcDescriptionPromptTests(unittest.TestCase):
    def prompt(self):
        return ('ORIGINAL KNOWN-EMPTY ARC DESCRIPTION APPLICABILITY (arc): description prose was not supplied; field revision owned\n'
            +namespace['ARC_NEW']+'TARGET BLOCK TO UPDATE:\n'+namespace['ANCESTOR_MANUAL']+namespace['RED'])
    def test_exact_new_direction_original_omission_and_manual_target(self):
        self.assertTrue(namespace['arc_description_prompt']('targeted screenplay update',self.prompt()))
    def test_missing_omission_trimmed_description_or_draft_or_unrelated_leaks_refuse(self):
        for text in [self.prompt().replace('description prose was not supplied; field revision ','unknown '),
            self.prompt().replace(namespace['ARC_NEW'],namespace['ARC_NEW'].strip()),
            self.prompt().replace(namespace['ANCESTOR_MANUAL'],namespace['GENERATED_B']),
            self.prompt()+namespace['ARC_UNRELATED'],self.prompt()+namespace['DRAFT']]:
            self.assertFalse(namespace['arc_description_prompt']('targeted screenplay update',text))

new_names={'KNOWN_EMPTY_MODES','KNOWN_EMPTY_NAME','KNOWN_EMPTY_PREVIEW'}
new_nodes=[n for n in module.body if isinstance(n,ast.Assign) and all(isinstance(t,ast.Name) and t.id in new_names for t in n.targets) or isinstance(n,ast.FunctionDef) and n.name in ('known_empty_arc_prompt','validate_known_empty_binding')]
exec(compile(ast.Module(body=new_nodes,type_ignores=[]),'<known-empty qualifier>','exec'),namespace)


class KnownEmptyPromptTests(unittest.TestCase):
    def prompt(self,mode,deleted=False):
        primary='primary';cause={'input':{'kind':'story_arc_field','arc_id':primary,'field':'description' if mode=='known-empty-second-tag' else 'name'},'reason':'deleted' if deleted else 'context_changed' if mode=='known-empty-second-tag' else 'changed'}
        text='ORIGINAL KNOWN-EMPTY ARC DESCRIPTION APPLICABILITY (primary): description prose was not supplied; field revision owned\n'
        if mode=='known-empty-second-tag':text+='ORIGINAL KNOWN-EMPTY ARC DESCRIPTION APPLICABILITY (second): description prose was not supplied; field revision owned-second\n'
        if not deleted:
            text+='CURRENT TAGGED STORY ARC FIELDS:\nprimary name: '+namespace['KNOWN_EMPTY_NAME']+'\n'
            if mode=='known-empty-second-tag':text+='primary description: '+namespace['ARC_NEW']+'\nsecond name: Still-empty supporting arc\n'
        return text+namespace['RED']+'\nTARGET BLOCK TO UPDATE:\n'+namespace['ANCESTOR_MANUAL']+'\n\nPROVEN INPUT CHANGE:\n'+json.dumps(cause)+'\nReturn only the complete replacement text.'
    def test_each_exact_native_case_prompt_is_admitted(self):
        for mode in namespace['KNOWN_EMPTY_MODES']:
            self.assertTrue(namespace['known_empty_arc_prompt']('targeted screenplay update',self.prompt(mode),mode,0))
        self.assertTrue(namespace['known_empty_arc_prompt']('targeted screenplay update',self.prompt('known-empty-deletion',True),'known-empty-deletion',1))
    def test_wrong_phase_unknown_scope_and_non_targeted_requests_refuse(self):
        for mode in namespace['KNOWN_EMPTY_MODES']:
            self.assertFalse(namespace['known_empty_arc_prompt']('generation',self.prompt(mode),mode,0))
            self.assertFalse(namespace['known_empty_arc_prompt']('targeted screenplay update',self.prompt(mode),mode,2))
        self.assertFalse(namespace['known_empty_arc_prompt']('targeted screenplay update',self.prompt('known-empty-name'),'unknown',0))
        self.assertFalse(namespace['known_empty_arc_prompt']('targeted screenplay update',self.prompt('known-empty-name'),'known-empty-name',1))
    def test_exact_manual_text_omissions_and_draft_exclusion_are_required(self):
        mode='known-empty-second-tag';text=self.prompt(mode)
        for changed in (text.replace(namespace['ANCESTOR_MANUAL'],namespace['ANCESTOR_MANUAL'].strip()),text.replace(namespace['ARC_NEW'],namespace['ARC_NEW'].strip()),text.replace('ORIGINAL KNOWN-EMPTY ARC DESCRIPTION APPLICABILITY (second)','missing omission'),text+namespace['DRAFT'],text+namespace['ARC_UNRELATED']):
            self.assertFalse(namespace['known_empty_arc_prompt']('targeted screenplay update',changed,mode,0))
    def test_falsely_supplied_empty_description_wrong_cause_and_missing_current_name_refuse(self):
        mode='known-empty-name';text=self.prompt(mode)
        for changed in (text+'\nprimary description: invented prose\n',text.replace(namespace['KNOWN_EMPTY_NAME'],'wrong current name'),text.replace('"field": "name"','"field": "description"')):
            self.assertFalse(namespace['known_empty_arc_prompt']('targeted screenplay update',changed,mode,0))
    def test_deleted_preview_cannot_retain_current_arc_context(self):
        text=self.prompt('known-empty-deletion',True)
        self.assertFalse(namespace['known_empty_arc_prompt']('targeted screenplay update',text+'CURRENT TAGGED STORY ARC FIELDS:','known-empty-deletion',1))
        self.assertFalse(namespace['known_empty_arc_prompt']('targeted screenplay update',text.replace('"deleted"','"changed"'),'known-empty-deletion',1))


class KnownEmptyBindingTests(unittest.TestCase):
    def binding(self,mode='known-empty-name',deleted=False):
        omission=lambda arc:{'arc_id':arc,'field':'description','value':'','revision_event_id':'owned-'+arc}
        fixture={'arc':{'id':'primary'}}
        previous=[omission('primary')];current=[omission('primary')];inputs=[{'arc_id':'primary','field':'name','value':namespace['KNOWN_EMPTY_NAME'],'revision_event_id':'name-new'}]
        if mode=='known-empty-second-tag':
            fixture['second_arc']={'id':'second'};previous.append(omission('second'));current=[omission('second')]
            inputs.append({'arc_id':'primary','field':'description','value':namespace['ARC_NEW'],'revision_event_id':'description-new'})
        return fixture,{'arc_description_applicability_previous':previous,'arc_description_applicability_current':[] if deleted else current,'arc_inputs':[] if deleted else inputs,'arc_absence_revisions':[['primary','delete-event']] if deleted else [],'cause':{'input':{'kind':'story_arc_field','arc_id':'primary','field':'description' if mode=='known-empty-second-tag' else 'name'},'reason':'deleted' if deleted else 'changed'},'request':{'block_id':'block'},'script_inputs':[{'block_id':'block','text':namespace['ANCESTOR_MANUAL']}]}
    def test_all_three_binding_shapes_preserve_owned_evidence(self):
        for mode in namespace['KNOWN_EMPTY_MODES']:
            fixture,binding=self.binding(mode)
            self.assertTrue(namespace['validate_known_empty_binding'](binding,fixture,mode))
        fixture,binding=self.binding('known-empty-deletion',True)
        self.assertTrue(namespace['validate_known_empty_binding'](binding,fixture,'known-empty-deletion',True))
    def test_unknown_forged_duplicate_or_lost_omission_refuses(self):
        import copy
        fixture,binding=self.binding()
        for key in ('arc_description_applicability_previous','arc_description_applicability_current'):
            for invalid in (None,[],binding[key]*2,[dict(binding[key][0],value='not empty')],[dict(binding[key][0],revision_event_id=None)],[dict(binding[key][0],arc_id='other')]):
                mutated=copy.deepcopy(binding);mutated[key]=invalid
                with self.assertRaises(RuntimeError):namespace['validate_known_empty_binding'](mutated,fixture,'known-empty-name')
    def test_second_tag_cannot_be_replaced_by_primary_or_lost(self):
        fixture,binding=self.binding('known-empty-second-tag')
        binding['arc_description_applicability_current'][0]['arc_id']='primary'
        with self.assertRaises(RuntimeError):namespace['validate_known_empty_binding'](binding,fixture,'known-empty-second-tag')
        fixture,binding=self.binding('known-empty-second-tag');binding['arc_inputs']=[i for i in binding['arc_inputs'] if i['field']!='description']
        with self.assertRaises(RuntimeError):namespace['validate_known_empty_binding'](binding,fixture,'known-empty-second-tag')
    def test_absence_requires_exact_owned_deleted_arc_and_no_current_context(self):
        fixture,binding=self.binding('known-empty-deletion',True)
        for invalid in ([],[['other','delete-event']],[['primary',None]],[['primary','delete-event']]*2):
            binding['arc_absence_revisions']=invalid
            with self.assertRaises(RuntimeError):namespace['validate_known_empty_binding'](binding,fixture,'known-empty-deletion',True)
    def test_wrong_target_or_selected_cause_refuses(self):
        fixture,binding=self.binding();binding['script_inputs'][0]['text']=namespace['ANCESTOR_MANUAL'].strip()
        with self.assertRaises(RuntimeError):namespace['validate_known_empty_binding'](binding,fixture,'known-empty-name')
        fixture,binding=self.binding();binding['cause']['input']['field']='arc_type'
        with self.assertRaises(RuntimeError):namespace['validate_known_empty_binding'](binding,fixture,'known-empty-name')


class RemovalWorkflowTests(unittest.TestCase):
    def test_original_native_assertions_are_preserved_and_removal_repetition_is_isolated(self):
        root=Path(__file__).resolve().parents[1]
        workflow=(root/'.github/workflows/manual-fact-native.yml').read_text()
        self.assertIn('branches: [test/screenplay-removal-native]',workflow)
        self.assertIn('EIDETIC_CAPTURE_SOURCE: d025abc67cbd82413c2ec896fb6ab01b1ea13402',workflow)
        self.assertIn('if changed != allowed:',workflow)
        self.assertIn('EIDETIC_CAPTURE_SCOPE: screenplay-removal-repeat',workflow)
        self.assertIn('eidetic-capture-artifact/screenplay-removal-repeat',workflow)
        self.assertIn("if len(Provider.records)!=6 or not all(r['accepted'] for r in Provider.records)",source)
        self.assertIn("if len(notices)!=2:raise RuntimeError('Expected two generated consumer notices')",source)
        self.assertIn("driver.canonical_state(database)!=before",source)
    def test_two_tag_fixture_uses_model_writer_and_public_load_without_sql(self):
        root=Path(__file__).resolve().parents[1]
        fixture=(root/'crates/server/examples/manual_fact_capture_fixture.rs').read_text()
        self.assertIn('project.timeline.tag_node(consumer, second)',fixture)
        self.assertIn('project_service::save_project(',fixture)
        self.assertIn('two-tag fixture requires a fresh application state',fixture)
        self.assertIn('state.project_database.active_path().is_some()',fixture)
        self.assertNotIn('state.project_path',fixture)
        self.assertNotIn('eidetic_server::persistence::',fixture)
        self.assertIn('project_service::load_project(',fixture)
        self.assertNotIn('INSERT INTO',fixture)
        self.assertIn('no tagging UI claim',fixture)


class KnownEmptyActualProviderTests(unittest.TestCase):
    def test_actual_http_routes_all_cases_and_requires_explicit_fresh_phase(self):
        import http.client
        import os
        import threading
        import types
        from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
        from unittest.mock import patch
        class Base(BaseHTTPRequestHandler):
            records_lock=threading.Lock()
            def log_message(self,*args):pass
        provider_nodes=[n for n in module.body if isinstance(n,ast.Assign) and all(isinstance(t,ast.Name) and t.id.isupper() for t in n.targets) or isinstance(n,ast.FunctionDef) and n.name in ('fact_prompt','notes_prompt','ancestor_notes_prompt','arc_description_prompt','known_empty_arc_prompt') or isinstance(n,ast.ClassDef) and n.name=='Provider']
        prompts=KnownEmptyPromptTests()
        for mode in namespace['KNOWN_EMPTY_MODES']:
            with self.subTest(mode=mode),patch.dict(os.environ,{'EIDETIC_CAPTURE_SCOPE':mode}):
                actual={'json':json,'os':os,'ui':types.SimpleNamespace(FixtureProvider=Base)}
                exec(compile(ast.Module(body=provider_nodes,type_ignores=[]),'<actual provider>','exec'),actual)
                provider=actual['Provider'];server=ThreadingHTTPServer(('127.0.0.1',0),provider)
                thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
                def post(system,user):
                    body=json.dumps({'stream':True,'messages':[{'role':'system','content':system},{'role':'user','content':user}]})
                    connection=http.client.HTTPConnection('127.0.0.1',server.server_port,timeout=5)
                    try:
                        connection.request('POST','/v1/chat/completions',body,{'Content-Type':'application/json'})
                        response=connection.getresponse();return response.status,response.read().decode()
                    finally:connection.close()
                try:
                    for generated in (actual['GENERATED_A'],actual['GENERATED_B']):
                        self.assertEqual(post('generation',actual['RED']+" Mara's choice")[0],200)
                        self.assertEqual(post('recap','Generate a scene recap for this screenplay beat: '+generated)[0],200)
                    self.assertEqual(len(provider.records),4)
                    status,stream=post('targeted screenplay update',prompts.prompt(mode))
                    self.assertEqual(status,200)
                    chunks=[json.loads(line[6:])['choices'][0]['delta']['content'] for line in stream.splitlines() if line.startswith('data: ') and line!='data: [DONE]']
                    self.assertEqual(''.join(chunks),actual['KNOWN_EMPTY_PREVIEW'])
                    self.assertTrue(all(r['accepted'] and r['synthetic'] and not r['real_model'] for r in provider.records))
                    if mode=='known-empty-deletion':
                        self.assertEqual(post('targeted screenplay update',prompts.prompt(mode,True))[0],422)
                        provider.allow_fresh_notes=True
                        self.assertEqual(post('targeted screenplay update',prompts.prompt(mode,True))[0],200)
                    else:
                        self.assertEqual(post('targeted screenplay update',prompts.prompt(mode))[0],422)
                    self.assertEqual(sum(r['accepted'] for r in provider.records),6 if mode=='known-empty-deletion' else 5)
                finally:
                    server.shutdown();server.server_close();thread.join(timeout=5)


class NativeReceiptRetirementTests(unittest.TestCase):
    def harness(self,sequence,process_alive=True):
        import types
        class Error(Exception):
            def __init__(self,code=0):
                self.domain='atspi_error';self.code=code;self.message='The application no longer exists'
        class Clock:
            now=0
            def monotonic(self):return self.now
            def sleep(self,seconds):self.now+=seconds
        class NativeText:
            def __init__(self,value):self.raw=value;self.characterCount=len(value)
            def getText(self,start,end):return self.raw[start:end]
        class Node:
            def __init__(self,value):self.value=value
            def queryText(self):return NativeText(self.value)
        reads=[];items=list(sequence)
        def find(root,predicate):
            reads.append(root)
            value=items.pop(0) if items else None
            if value=='retired':raise Error()
            if value=='unexpected':raise Error(9)
            if isinstance(value,Exception):raise value
            return Node(value) if value is not None else None
        wait_source=ast.parse(Path(__file__).with_name('qualify-screenplay-authoring.py').read_text())
        wait_node=next(n for n in wait_source.body if isinstance(n,ast.FunctionDef) and n.name=='wait_for')
        wait_namespace={'GLib':types.SimpleNamespace(Error=Error),'time':Clock(),'DEADLINE':1,'NATIVE_WINDOW_PID':None if process_alive else 123,'ACCESSIBILITY_RETIREMENTS':0,'Path':lambda _:types.SimpleNamespace(exists=lambda:False)}
        exec(compile(ast.Module(body=[wait_node],type_ignores=[]),'<existing native wait>','exec'),wait_namespace)
        actual_ui=types.SimpleNamespace(find=find,wait_for=wait_namespace['wait_for'])
        receipt_node=next(n for n in module.body if isinstance(n,ast.FunctionDef) and n.name=='receipt')
        receipt_namespace={'ui':actual_ui,'json':json}
        exec(compile(ast.Module(body=[receipt_node],type_ignores=[]),'<actual receipt>','exec'),receipt_namespace)
        return receipt_namespace['receipt'],reads,wait_namespace,Error
    def valid(self,**overrides):
        return json.dumps({'fixture':'qualifier-only public commands and read-only impacts; synthetic replies','error':'','impactReads':7,**overrides})
    def test_actual_receipt_reacquires_retired_node_without_replaying_action(self):
        receipt,reads,wait,_=self.harness(['retired',self.valid()])
        self.assertEqual(receipt('same-native-root')['impactReads'],7)
        self.assertEqual(reads,['same-native-root','same-native-root'])
        self.assertEqual(wait['ACCESSIBILITY_RETIREMENTS'],1)
    def test_other_accessibility_errors_are_not_swallowed(self):
        receipt,reads,_,Error=self.harness(['unexpected',self.valid()])
        with self.assertRaises(Error):receipt('root')
        self.assertEqual(len(reads),1)
    def test_missing_receipt_fails_at_existing_deadline(self):
        receipt,reads,_,_=self.harness([None])
        with self.assertRaisesRegex(RuntimeError,'Timed out waiting'):receipt('root')
        self.assertEqual(len(reads),4)
    def test_dead_native_process_is_not_mistaken_for_retired_child(self):
        receipt,reads,_,_=self.harness([self.valid()],process_alive=False)
        with self.assertRaisesRegex(RuntimeError,'Native application exited'):receipt('root')
        self.assertEqual(reads,[])
    def test_malformed_or_failed_receipt_still_refuses(self):
        for value,error in [('not JSON',ValueError),(self.valid(error='actual public command failed'),RuntimeError),(self.valid(fixture='unlabelled'),RuntimeError)]:
            receipt,reads,_,_=self.harness([value,self.valid()])
            with self.assertRaises(error):receipt('root')
            self.assertEqual(len(reads),1)


class OwnedArcDeletionReceiptTests(unittest.TestCase):
    def setUp(self):
        import sqlite3,types
        self.conn=sqlite3.connect(':memory:')
        self.conn.executescript('''
            CREATE TABLE arcs(id TEXT PRIMARY KEY);
            CREATE TABLE node_arcs(node_id TEXT,arc_id TEXT);
            CREATE TABLE commands(id TEXT,payload_type TEXT,payload_json TEXT);
            CREATE TABLE change_events(id TEXT,command_id TEXT);
            CREATE TABLE object_revisions(id TEXT,change_event_id TEXT,operation TEXT,object_kind TEXT,object_id TEXT);
            CREATE TABLE object_revision_fields(revision_id TEXT,field_key TEXT,old_type TEXT,old_text TEXT,new_type TEXT);
            INSERT INTO node_arcs VALUES ('consumer','primary');
            INSERT INTO commands VALUES ('delete-command','story_arc.delete','{"arc_id":"primary"}');
            INSERT INTO change_events VALUES ('delete-event','delete-command');
            INSERT INTO object_revisions VALUES ('delete-revision','delete-event','delete','story_arc','primary');
        ''')
        self.conn.executemany('INSERT INTO object_revision_fields VALUES (?,?,?,?,?)',[('delete-revision','description','text','',None),('delete-revision','name','text',namespace['KNOWN_EMPTY_NAME'],None)])
        node=next(n for n in module.body if isinstance(n,ast.FunctionDef) and n.name=='owned_arc_deletion_receipt')
        local={'json':json,'KNOWN_EMPTY_NAME':namespace['KNOWN_EMPTY_NAME'],'ui':types.SimpleNamespace(query=lambda _,sql,args=():self.conn.execute(sql,args).fetchall())}
        exec(compile(ast.Module(body=[node],type_ignores=[]),'<actual deletion read>','exec'),local);self.read=local['owned_arc_deletion_receipt']
    def tearDown(self):self.conn.close()
    def test_owned_delete_with_retained_raw_reference_has_no_live_context(self):
        before='\n'.join(self.conn.iterdump());receipt=self.read(None,'primary')
        self.assertEqual(receipt['change_event_id'],'delete-event')
        self.assertEqual(receipt['retained_raw_references'],[('consumer','primary')])
        self.assertEqual(receipt['live_tag_context'],[])
        self.assertEqual('\n'.join(self.conn.iterdump()),before)
    def test_live_arc_or_missing_owned_history_is_not_deletion(self):
        self.conn.execute("INSERT INTO arcs VALUES ('primary')")
        self.assertIsNone(self.read(None,'primary'))
        self.conn.execute('DELETE FROM arcs');self.conn.execute('DELETE FROM object_revisions')
        self.assertIsNone(self.read(None,'primary'))
    def test_other_arc_or_other_command_cannot_prove_deletion(self):
        self.conn.execute('UPDATE commands SET payload_json=?',(json.dumps({'arc_id':'other'}),))
        self.assertIsNone(self.read(None,'primary'))
        self.conn.execute('UPDATE commands SET payload_json=?,payload_type=?',(json.dumps({'arc_id':'primary'}),'story_arc.set_metadata'))
        self.assertIsNone(self.read(None,'primary'))
    def test_latest_non_delete_revision_and_wrong_deleted_values_refuse(self):
        self.conn.execute("UPDATE object_revision_fields SET old_text='other name' WHERE field_key='name'")
        self.assertIsNone(self.read(None,'primary'))
        self.conn.execute("UPDATE object_revision_fields SET old_text=? WHERE field_key='name'",(namespace['KNOWN_EMPTY_NAME'],))
        self.conn.execute("UPDATE object_revisions SET operation='update'")
        self.assertIsNone(self.read(None,'primary'))
    def test_native_fresh_absence_binding_must_equal_the_actual_delete_clock(self):
        self.assertIn("proof['absence_revisions']!=[[primary,deletion['change_event_id']]]",source)
        self.assertIn("Deleted known-empty stale acceptance wrote canonical state",source)
        self.assertIn("exact chosen-block known-empty acceptance",source)


if __name__=='__main__':unittest.main()


class RemovalPromptTests(unittest.TestCase):
    def harness(self):
        nodes=[n for n in module.body if isinstance(n,ast.Assign) and all(isinstance(t,ast.Name) and t.id.isupper() for t in n.targets) or isinstance(n,ast.FunctionDef) and n.name=='removal_prompt']
        local={'json':json};exec(compile(ast.Module(body=nodes,type_ignores=[]),'<removal predicate>', 'exec'),local)
        return local
    def prompt(self,local,deleted):
        cause={'input':{'kind':'script_block','block_id':'source'},'reason':'deleted' if deleted else 'changed','input_excerpt':local['GENERATED_A'],'current_revision_event_id':None if deleted else 'current'}
        return ('CANONICAL SCREENPLAY CONTEXT:\n'+('' if deleted else local['SAVED'])+'\n'+local['RED']+'\nTARGET BLOCK TO UPDATE:\n'+local['ANCESTOR_MANUAL']+'\nPROVEN INPUT CHANGE:\n'+json.dumps(cause)+'\nReturn only replacement text')
    def test_deleted_source_is_historical_only_and_exact_manual_target_is_current(self):
        local=self.harness()
        self.assertTrue(local['removal_prompt']('targeted screenplay update',self.prompt(local,False),0))
        self.assertTrue(local['removal_prompt']('targeted screenplay update',self.prompt(local,True),1))
    def test_removed_current_text_or_draft_leaks_wrong_cause_and_missing_target_refuse(self):
        local=self.harness();prompt=self.prompt(local,True)
        for bad in [local['SAVED']+prompt,local['GENERATED_A']+prompt,prompt+local['DRAFT'],prompt.replace(local['ANCESTOR_MANUAL'],''),prompt.replace('"deleted"','"changed"'),prompt.replace('"script_block"','"bible_field"')]:
            self.assertFalse(local['removal_prompt']('targeted screenplay update',bad,1))

class RemovalActualProviderTests(unittest.TestCase):
    def test_actual_synthetic_http_enforces_old_then_explicit_fresh_deleted_source_phase(self):
        import http.client, os, threading, types
        from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
        from unittest.mock import patch
        class Base(BaseHTTPRequestHandler):
            records_lock=threading.Lock()
            def log_message(self,*args):pass
        nodes=[n for n in module.body if isinstance(n,ast.Assign) and all(isinstance(t,ast.Name) and t.id.isupper() for t in n.targets) or isinstance(n,ast.FunctionDef) and n.name=='removal_prompt' or isinstance(n,ast.ClassDef) and n.name=='Provider']
        prompts=RemovalPromptTests();local={'json':json,'os':os,'ui':types.SimpleNamespace(FixtureProvider=Base)}
        exec(compile(ast.Module(body=nodes,type_ignores=[]),'<actual removal provider>','exec'),local)
        provider=local['Provider'];server=ThreadingHTTPServer(('127.0.0.1',0),provider);thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
        def post(deleted):
            body=json.dumps({'stream':True,'messages':[{'role':'system','content':'targeted screenplay update'},{'role':'user','content':prompts.prompt(local,deleted)}]})
            connection=http.client.HTTPConnection('127.0.0.1',server.server_port,timeout=5)
            try:
                connection.request('POST','/v1/chat/completions',body,{'Content-Type':'application/json'});response=connection.getresponse();return response.status,response.read().decode()
            finally:connection.close()
        try:
            with patch.dict(os.environ,{'EIDETIC_CAPTURE_SCOPE':'screenplay-removal'}):
                status,stream=post(False);self.assertEqual(status,200)
                chunks=[json.loads(line[6:])['choices'][0]['delta']['content'] for line in stream.splitlines() if line.startswith('data: ') and line!='data: [DONE]']
                self.assertEqual(''.join(chunks),local['REMOVAL_PREVIEW'])
                self.assertEqual(post(True)[0],422)
                provider.allow_fresh_notes=True
                self.assertEqual(post(True)[0],200)
                self.assertEqual(post(True)[0],422)
                self.assertEqual(sum(r['accepted'] for r in provider.records),2)
                self.assertTrue(all(r['synthetic'] and not r['real_model'] for r in provider.records))
        finally:server.shutdown();server.server_close();thread.join(timeout=5)


class RemovalPendingStorageTests(unittest.TestCase):
    def test_actual_application_schema_selects_latest_bound_pending_review_without_writes(self):
        import re,sqlite3,types
        root=Path(__file__).resolve().parents[1]
        store=(root/'crates/server/src/propagation_proposal_store.rs').read_text()
        schema=re.search(r'const PROPAGATION_PROPOSAL_SCHEMA_SQL: &str = r#"(.*?)"#;',store,re.S).group(1)
        conn=sqlite3.connect(':memory:');conn.executescript(schema)
        helper=next(n for n in module.body if isinstance(n,ast.FunctionDef) and n.name=='pending_script_review')
        local={'json':json,'ui':types.SimpleNamespace(query=lambda db,sql:db.execute(sql).fetchall())}
        exec(compile(ast.Module(body=[helper],type_ignores=[]),'<actual pending review read>','exec'),local)
        read=local['pending_script_review'];self.assertIsNone(read(conn))
        for pid,status,bound,text in [('old','pending',True,'Old preview'),('latest','pending',True,'  Exact synthetic preview — 雨\n\n  '),('fact','pending',False,'Unbound fact proposal'),('accepted','accepted',True,'Accepted preview')]:
            conn.execute("INSERT INTO propagation_proposals(id,action,target_kind,target_id,status,summary,proposed_text,created_at_ms,created_event_id) VALUES (?,'update','script_block','B',?,'Fixture',?,1,'fixture.event')",(pid,status,text))
            if bound:conn.execute('INSERT INTO script_impact_proposal_bindings VALUES (?,?)',(pid,json.dumps({'request':{'block_id':'B'},'fixture_id':pid})))
        before='\n'.join(conn.iterdump());conn.execute('PRAGMA query_only=ON')
        selected=read(conn)
        self.assertEqual(selected,('latest',{'request':{'block_id':'B'},'fixture_id':'latest'},'  Exact synthetic preview — 雨\n\n  '))
        self.assertEqual('\n'.join(conn.iterdump()),before)
        conn.close()
