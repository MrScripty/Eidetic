import importlib.util
import json
from pathlib import Path
import sys
from types import ModuleType
import unittest
from unittest.mock import patch

repository = ModuleType('gi.repository')
repository.Atspi = ModuleType('Atspi')
repository.GLib = ModuleType('GLib')
spec = importlib.util.spec_from_file_location('selected_capture', Path(__file__).with_name('qualify-recalled-facts-preview.py'))
driver = importlib.util.module_from_spec(spec)
with patch.dict(sys.modules, {'pyatspi': ModuleType('pyatspi'), 'gi': ModuleType('gi'), 'gi.repository': repository}):
    spec.loader.exec_module(driver)


class SelectedCaptureTests(unittest.TestCase):
    def prompt(self, chosen=''):
        return driver.driver.A_TEXT + driver.driver.F_TEXT + driver.driver.MANUAL + driver.driver.AMBER + ('profile.tagline: ' + chosen + ' Mara trusts the Keeper' if chosen else '')

    def test_selected_prompt_accepts_exact_value_and_refuses_unselected_or_unsaved_text(self):
        prompt = self.prompt(driver.SELECTED)
        self.assertTrue(driver.prompt_checks(prompt, 'preview', 3))
        self.assertTrue(driver.prompt_checks(prompt+'PROVEN INPUT CHANGE: '+driver.driver.BLUE, 'preview', 3))
        for contaminant in (driver.UNSELECTED, driver.driver.DRAFT, 'profile.tagline: '+driver.driver.BLUE, 'environment.weather: Rain'):
            self.assertFalse(driver.prompt_checks(prompt + contaminant, 'preview', 3))

    def test_fresh_prompt_rejects_old_selected_value_and_missing_path(self):
        fresh = self.prompt(driver.FRESH)
        self.assertTrue(driver.prompt_checks(fresh, 'preview', 4))
        self.assertFalse(driver.prompt_checks(self.prompt(driver.SELECTED), 'preview', 4))
        self.assertFalse(driver.prompt_checks(fresh + driver.SELECTED, 'preview', 4))
        self.assertFalse(driver.prompt_checks(fresh.replace('Mara trusts the Keeper', ''), 'preview', 4))

    def test_initial_automatic_context_does_not_consume_selected_peer(self):
        initial = driver.driver.A_TEXT + driver.driver.F_TEXT + driver.driver.BLUE
        self.assertTrue(driver.prompt_checks(initial, 'generation', 0))
        self.assertFalse(driver.prompt_checks(initial + driver.SELECTED, 'generation', 0))
        self.assertTrue(driver.prompt_checks(self.prompt(), 'preview', 2))
        self.assertFalse(driver.prompt_checks(self.prompt(driver.SELECTED), 'preview', 2))

    def binding(self):
        return {'request': {'recall_selection': {'query': {'story_time_ms': None}, 'facts': [{'field_id': 'qualification.keeper.tagline', 'revision_event_id': 'source'}],
                'paths': [{'relationship': {'edge': {'edge_id': 'qualification.mara.keeper'}}}]}},
                'bible_inputs': [{'field_id': 'qualification.keeper.tagline', 'revision_event_id': 'source', 'value': {'type': 'text', 'value': driver.SELECTED}}]}

    def test_exact_consumed_revision_is_required_without_client_values(self):
        value = self.binding()
        driver.require_selection_binding(value, (driver.SELECTED, 'source'))
        with self.assertRaisesRegex(RuntimeError, 'custody'):
            driver.require_selection_binding(value, (driver.SELECTED, 'ABA-new-source'))
        value['request']['recall_selection']['facts'][0]['value'] = driver.SELECTED
        with self.assertRaisesRegex(RuntimeError, 'custody'):
            driver.require_selection_binding(value, (driver.SELECTED, 'source'))

    def test_canonical_binding_refuses_unselected_consumption_or_wrong_value(self):
        value = self.binding()
        value['bible_inputs'].append({'field_id': 'qualification.keeper.motivation'})
        with self.assertRaisesRegex(RuntimeError, 'Unselected'):
            driver.require_selection_binding(value, (driver.SELECTED, 'source'))
        value = self.binding()
        value['bible_inputs'][0]['value']['value'] = driver.FRESH
        with self.assertRaisesRegex(RuntimeError, 'Consumed'):
            driver.require_selection_binding(value, (driver.SELECTED, 'source'))

    def test_unspecified_story_time_uses_native_deletion_before_recall(self):
        class Field:
            name = 'Recall story time (ms)'
            text = '1000'
            def clear_cache(self): pass
            def getState(self): return self
            def contains(self, state): return True
            def queryComponent(self): return self
            def getExtents(self, coordinates): return (1600, 200, 280, 28)
        field = Field()
        events = []
        def command(*args):
            events.append(args)
            if args[-1] == 'BackSpace': field.text = ''
        with patch.object(driver.ui.pyatspi, 'STATE_EDITABLE', 1, create=True), patch.object(driver.ui.pyatspi, 'STATE_FOCUSED', 2, create=True), patch.object(driver.ui.pyatspi, 'XY_SCREEN', 0, create=True):
            with patch.object(driver.ui, 'wait_for', side_effect=lambda label, check: check()), patch.object(driver.ui, 'reveal', side_effect=lambda app, check: field if check(field) else None), patch.object(driver.ui, 'click_control'), patch.object(driver.ui, 'command', side_effect=command), patch.object(driver.ui, 'text_of', side_effect=lambda node: node.text), patch.object(driver.ui, 'reveal_button') as recall:
                driver.recall_unspecified(None, 'window')
                self.assertEqual(field.text, '')
                self.assertEqual([e[-1] for e in events], ['ctrl+a', 'BackSpace'])
                recall.assert_called_once_with(None, 'Recall related story facts', 'window')

    def test_native_disclosure_click_admits_unknown_summary_and_excludes_landmark(self):
        class Node:
            name = 'Author-selected recalled facts used for this preview'
            def __init__(self, role): self.role = role
            def getRole(self): return self.role
        landmark, summary = Node(9), Node(1)
        with patch.object(driver.ui.pyatspi, 'ROLE_UNKNOWN', 1, create=True), patch.object(driver.ui.pyatspi, 'ROLE_PUSH_BUTTON', 2, create=True), patch.object(driver.ui.pyatspi, 'ROLE_TOGGLE_BUTTON', 3, create=True):
            with patch.object(driver.ui, 'wait_for', side_effect=lambda label, check: check()), patch.object(driver.ui, 'reveal', side_effect=lambda app, check: next(n for n in [landmark, summary] if check(n))), patch.object(driver.ui, 'click_control') as click:
                self.assertIs(driver.click_disclosure(None, 'window', summary.name), summary)
                click.assert_called_once_with(summary, 'window')

    def test_workflow_freezes_source_and_limits_delta_to_qualification_files(self):
        root = Path(__file__).resolve().parent.parent
        workflow = (root/'.github/workflows/recalled-facts-native.yml').read_text()
        self.assertIn(driver.SOURCE, workflow)
        self.assertIn('git merge-base --is-ancestor', workflow)
        self.assertIn('changed - allowed', workflow)
        config = (root/'scripts/recalled-facts-vite.config.mts').read_text()
        self.assertIn('/propagationProposalProjection.svelte.ts', config)
        self.assertIn('importer?.endsWith', config)
        self.assertIn('recalledFacts.svelte.ts', config)
        self.assertNotIn('invokeDesktop', config)


if __name__ == '__main__':
    unittest.main()
