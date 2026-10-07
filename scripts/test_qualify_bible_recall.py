import importlib.util
from pathlib import Path
import sqlite3
import sys
import tempfile
from types import ModuleType
import unittest
from unittest.mock import patch

repository=ModuleType('gi.repository')
repository.Atspi=ModuleType('Atspi')
repository.GLib=ModuleType('GLib')
spec=importlib.util.spec_from_file_location('recall_capture',Path(__file__).with_name('qualify-bible-recall.py'))
driver=importlib.util.module_from_spec(spec)
with patch.dict(sys.modules,{'pyatspi':ModuleType('pyatspi'),'gi':ModuleType('gi'),'gi.repository':repository}):
    spec.loader.exec_module(driver)


class RecallCaptureTests(unittest.TestCase):
    def test_result_text_is_scrolled_into_view_and_normalizes_native_line_wrapping(self):
        class Node:
            def __init__(self,role):self.role=role
            def getRole(self):return self.role
        paragraph,hidden_section=Node(1),Node(0)
        with patch.object(driver.ui.pyatspi,'ROLE_COMBO_BOX',2,create=True), patch.object(driver.ui.pyatspi,'ROLE_PARAGRAPH',1,create=True), patch.object(driver.ui,'text_of',return_value='Unresolved:\n environment.weather — no established value'), patch.object(driver.ui,'find',return_value=None):
            with patch.object(driver.ui,'reveal',side_effect=lambda app,predicate:next((n for n in [hidden_section,paragraph] if predicate(n)),None)) as reveal:
                self.assertIs(driver.visible_text(None,'Unresolved: environment.weather'),paragraph)
                reveal.assert_called_once()

    def test_visible_connection_status_is_preferred_without_scrolling_hidden_ancestors(self):
        status=object()
        with patch.object(driver.ui,'find',return_value=status), patch.object(driver.ui,'reveal') as reveal:
            self.assertIs(driver.visible_text(None,'Connected'),status)
            reveal.assert_not_called()

    def test_synthetic_context_requires_saved_inputs_and_exact_current_fact_excludes_draft_and_unrecalled_neighbor(self):
        provider=object.__new__(driver.RecallProvider)
        before=driver.A_TEXT+driver.F_TEXT+'profile.tagline: '+driver.BLUE
        self.assertTrue(provider.contains_expected_context(before,'generation'))
        self.assertFalse(provider.contains_expected_context(before+driver.DRAFT,'generation'))
        self.assertFalse(provider.contains_expected_context(before+'environment.weather: Rain','generation'))
        current=driver.A_TEXT+driver.F_TEXT+driver.MANUAL+'profile.tagline: '+driver.AMBER
        self.assertTrue(provider.contains_expected_context(current,'preview'))
        self.assertFalse(provider.contains_expected_context(current+'profile.tagline: '+driver.BLUE,'preview'))
        self.assertFalse(provider.contains_expected_context(current+'Flooded road','preview'))
        self.assertEqual(provider.response_text('preview'),driver.PROPOSED)

    def test_read_only_history_and_text_checks_detect_any_replacement_or_new_history(self):
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'fixture.db'
            with sqlite3.connect(path) as conn:
                conn.executescript("""
                CREATE TABLE script_blocks(id TEXT,text TEXT,updated_event_id TEXT,segment_id TEXT,sort_order INTEGER,deleted_event_id TEXT);
                CREATE TABLE script_segments(id TEXT,source_node_id TEXT,start_ms INTEGER,end_ms INTEGER,deleted_event_id TEXT);
                CREATE TABLE commands(id TEXT);
                INSERT INTO script_segments VALUES('s','B',1,2,NULL);
                INSERT INTO script_blocks VALUES('b','Exact saved manual B','saved','s',0,NULL);
                INSERT INTO commands VALUES('saved');
                """)
            original={'B':driver.ui.blocks(path,'B')}
            history=driver.canonical_state(path)
            driver.require_preserved(path,original,history)
            with sqlite3.connect(path) as conn:
                conn.execute("INSERT INTO commands VALUES('unexpected')")
            with self.assertRaisesRegex(RuntimeError,'history'):
                driver.require_preserved(path,original,history)
            with sqlite3.connect(path) as conn:
                conn.execute("UPDATE script_blocks SET text='replaced'")
            with self.assertRaisesRegex(RuntimeError,'screenplay'):
                driver.require_preserved(path,original)

    def test_disclosure_locator_excludes_the_same_named_noninteractive_section(self):
        class Node:
            name = 'Related story facts'
            def __init__(self,role,x=1600):self.role,self.x=role,x
            def getRole(self):return self.role
            def queryComponent(self):return self
            def getExtents(self,coordinates):return (self.x,140,250,20)
        region=Node(0)
        with patch.object(driver.ui.pyatspi,'ROLE_PUSH_BUTTON',1,create=True), patch.object(driver.ui.pyatspi,'ROLE_TOGGLE_BUTTON',2,create=True), patch.object(driver.ui.pyatspi,'ROLE_UNKNOWN',3,create=True), patch.object(driver.ui.pyatspi,'XY_SCREEN',0,create=True):
            for role in (1,2,3):
                summary,left_summary=Node(role),Node(role,x=12)
                with patch.object(driver.ui,'reveal',side_effect=lambda app,predicate:next(n for n in [region,left_summary,summary] if predicate(n))):
                    self.assertIs(driver.recall_disclosure(None),summary)

    def test_proposed_text_must_be_wholly_visible_inside_the_script_viewport(self):
        self.assertTrue(driver.contained((10,10,20,20),(0,0,50,50)))
        self.assertFalse(driver.contained((10,10,60,20),(0,0,50,50)))
        self.assertFalse(driver.contained((10,-1,20,20),(0,0,50,50)))


if __name__=='__main__':
    unittest.main()
