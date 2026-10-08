"""No-display regression checks for the synthetic provider's exact fact custody."""
import ast
import json
from pathlib import Path
import unittest

source=Path(__file__).with_name('qualify-manual-fact-reconciliation.py').read_text()
module=ast.parse(source)
# Load only constants and the pure prompt predicate, without pyatspi/native imports.
selected=[n for n in module.body if isinstance(n,ast.Assign) and all(isinstance(t,ast.Name) and t.id in {'RED','BLUE','SAVED','DRAFT','BIBLE_DRAFT','MOTIVATION_DRAFT','NOTES','GENERATED_B'} for t in n.targets) or isinstance(n,ast.FunctionDef) and n.name in ('fact_prompt','notes_prompt','context_prompt_matches')]
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

if __name__=='__main__':unittest.main()
