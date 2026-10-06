"""Qualification boundary tests. GUI main is never simulated or stubbed here."""
from http.client import HTTPConnection
from http.server import ThreadingHTTPServer
import importlib.util
import json
from pathlib import Path
import sys
import threading
from types import ModuleType
import unittest
from unittest.mock import patch

repository = ModuleType('gi.repository')
repository.Atspi = ModuleType('Atspi')
repository.GLib = ModuleType('GLib')
spec = importlib.util.spec_from_file_location('authoring_walkthrough_capture', Path(__file__).with_name('qualify-story-authoring-walkthrough.py'))
driver = importlib.util.module_from_spec(spec)
with patch.dict(sys.modules, {'pyatspi': ModuleType('pyatspi'), 'gi': ModuleType('gi'), 'gi.repository': repository}):
    spec.loader.exec_module(driver)


class NativeAuthoringProviderTests(unittest.TestCase):
    def test_actual_http_sequence_consumes_only_exact_saved_changed_context(self):
        provider=driver.AuthoringProvider
        provider.records.clear()
        server=ThreadingHTTPServer(('127.0.0.1',0),provider)
        thread=threading.Thread(target=server.serve_forever,kwargs={'poll_interval':0.01},daemon=True)
        thread.start()
        try:
            def send(kind,user):
                body={'stream':True,'messages':[{'role':'system','content':'targeted screenplay update' if kind=='preview' else 'Generate'}, {'role':'user','content':user}]}
                connection=HTTPConnection(*server.server_address,timeout=5)
                connection.request('POST','/v1/chat/completions',json.dumps(body),{'Content-Type':'application/json'})
                response=connection.getresponse(); data=response.read().decode(); connection.close()
                return response.status,data
            initial=driver.A_TEXT+driver.F_TEXT+driver.C_TEXT+driver.D_TEXT+f'profile.tagline: {driver.BLUE}'
            current=driver.F_TEXT+driver.E_TEXT+driver.C_EDITED+driver.D_TEXT+driver.MANUAL+f'profile.tagline: {driver.AMBER}'
            self.assertEqual(send('generation',initial)[0],200)
            self.assertEqual(send('recap','Generate a scene recap for this screenplay beat:'+driver.GENERATED)[0],200)
            for bad in (current.replace(driver.C_EDITED,driver.C_TEXT),current.replace(driver.AMBER,driver.BLUE),current+driver.A_TEXT,current+driver.DRAFT,current.replace(driver.MANUAL,driver.GENERATED)):
                self.assertEqual(send('preview',bad)[0],422)
            status,data=send('preview',current)
            self.assertEqual(status,200)
            text=''.join(json.loads(event.removeprefix('data: '))['choices'][0]['delta']['content'] for event in data.strip().split('\n\n')[:-1])
            self.assertEqual(text,driver.PROPOSED)
            driver.ui.require_accepted_preview(provider.records)
            self.assertEqual(len([r for r in provider.records if r.get('accepted')]),3)
            self.assertTrue(all(r['real_model'] is False for r in provider.records))
        finally:
            server.shutdown(); server.server_close(); thread.join(timeout=2)

    def test_generation_requires_actual_original_A_and_excludes_future_E(self):
        provider=object.__new__(driver.AuthoringProvider)
        initial=driver.A_TEXT+driver.F_TEXT+driver.C_TEXT+driver.D_TEXT+f'profile.tagline: {driver.BLUE}'
        self.assertTrue(provider.contains_expected_context(initial,'generation'))
        self.assertFalse(provider.contains_expected_context(initial.replace(driver.A_TEXT,''),'generation'))
        self.assertFalse(provider.contains_expected_context(initial+driver.E_TEXT,'generation'))


class FormattedScreenplayAnchorTests(unittest.TestCase):
    def test_raw_multiline_C_is_located_by_its_unique_rendered_action_line(self):
        # The native failure capture shows a separate heading and action <p>.
        rendered_children=['INT. TICKET OFFICE - NIGHT','The departure board reads midnight.']
        self.assertFalse(any(driver.C_TEXT.strip() in child for child in rendered_children))
        self.assertEqual(driver.screenplay_anchor(driver.C_TEXT),rendered_children[1])
        self.assertTrue(any(driver.screenplay_anchor(driver.C_TEXT) in child for child in rendered_children))
        self.assertEqual(driver.screenplay_anchor(driver.F_TEXT),'Eli keeps the gate open.')
        self.assertEqual(driver.screenplay_anchor(driver.GENERATED),driver.GENERATED.strip())
        with self.assertRaisesRegex(RuntimeError,'no authored text'):
            driver.screenplay_anchor('\n\n')


class NativeReviewLabelTests(unittest.TestCase):
    def test_hidden_select_option_cannot_hide_a_visible_bounded_cause(self):
        class State:
            def __init__(self,showing):self.showing=showing
            def contains(self,kind):return self.showing
        class Node:
            name='Source screenplay text changed.'
            def __init__(self,showing,rect):self.showing=showing;self.rect=rect
            def getState(self):return State(self.showing)
            def queryComponent(self):return self
            def getExtents(self,coords):return self.rect
            def getRoleName(self):return 'combo box'
        class Root:
            def clear_cache(self):pass
        nodes=[Node(False,(0,0,0,0)),Node(True,(300,600,370,20))]
        with patch.object(driver.ui,'walk',return_value=iter(nodes)),patch.object(driver.ui,'text_of',side_effect=lambda n:n.name),patch.object(driver,'script_viewport',return_value=(280,200,1320,840)),patch.object(driver.ui.pyatspi,'STATE_SHOWING',1,create=True),patch.object(driver.ui.pyatspi,'XY_SCREEN',0,create=True):
            shown=driver.visible_review_label(Root(),'owned-window','Source screenplay text changed.')
        self.assertEqual(shown['bounds'],[300,600,370,20])


class VisibleProposalGeometryTests(unittest.TestCase):
    def test_showing_or_window_bounds_cannot_substitute_for_actual_script_viewport(self):
        viewport=(280,200,1320,840)
        self.assertTrue(driver.contained((700,300,10,20),viewport))
        for offscreen in ((270,300,10,20),(700,190,10,20),(1600,300,10,20),(700,1040,10,20),(700,300,0,20)):
            self.assertFalse(driver.contained(offscreen,viewport))

    def test_exact_text_geometry_refuses_changed_native_paragraph(self):
        class Text:
            characterCount=7
            def getText(self,start,end): return 'Changed'
        class Node:
            def queryText(self): return Text()
        with self.assertRaisesRegex(RuntimeError,'Native text changed'):
            driver.text_rectangles(Node(),driver.PROPOSED)

    def test_canonical_preservation_checks_all_saved_scenes(self):
        expected={'A':[('a','manual','rev',0,100)],'F':[('f','saved','rev',100,200)]}
        with patch.object(driver.ui,'blocks',side_effect=lambda db,node:expected[node]):
            driver.require_canonical_unchanged(None,expected)
        with patch.object(driver.ui,'blocks',side_effect=lambda db,node:[] if node=='F' else expected[node]):
            with self.assertRaisesRegex(RuntimeError,'unrelated canonical screenplay: F'):
                driver.require_canonical_unchanged(None,expected)


if __name__=='__main__':
    unittest.main()
