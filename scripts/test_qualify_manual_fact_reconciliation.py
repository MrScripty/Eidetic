"""No-display regression checks for the synthetic provider's exact fact custody."""
import ast
import json
from pathlib import Path
import unittest

source=Path(__file__).with_name('qualify-manual-fact-reconciliation.py').read_text()
module=ast.parse(source)
# Load only constants and the pure prompt predicate, without pyatspi/native imports.
selected=[n for n in module.body if isinstance(n,ast.Assign) and all(isinstance(t,ast.Name) and t.id in {'RED','BLUE','SAVED','DRAFT'} for t in n.targets) or isinstance(n,ast.FunctionDef) and n.name=='fact_prompt']
namespace={'json':json};exec(compile(ast.Module(body=selected,type_ignores=[]),'<pure qualifier>','exec'),namespace)

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
    def test_later_proposal_uses_new_current_and_original_consumed_fact(self):
        e=self.evidence();e['facts'][0]['text']=namespace['BLUE']
        self.assertTrue(namespace['fact_prompt'](json.dumps(e),4))
        self.assertFalse(namespace['fact_prompt'](json.dumps(e),3))
    def test_saved_whitespace_is_not_normalized(self):
        e=self.evidence();e['text']=e['text'].strip()
        self.assertFalse(namespace['fact_prompt'](json.dumps(e),0))

if __name__=='__main__':unittest.main()
