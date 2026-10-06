"""Exercise the native qualification's actual synthetic HTTP evidence boundary."""
import importlib.util
import json
from pathlib import Path
import sys
import threading
from types import ModuleType
import unittest
from unittest.mock import patch
from http.server import ThreadingHTTPServer
from urllib.error import HTTPError
from urllib.request import Request, urlopen

repository = ModuleType('gi.repository')
repository.Atspi = ModuleType('Atspi')
repository.GLib = ModuleType('GLib')
spec = importlib.util.spec_from_file_location(
    'relationship_capture', Path(__file__).with_name('qualify-bible-relationship.py'))
driver = importlib.util.module_from_spec(spec)
with patch.dict(sys.modules, {'pyatspi': ModuleType('pyatspi'), 'gi': ModuleType('gi'),
                             'gi.repository': repository}):
    spec.loader.exec_module(driver)


class RelationshipProviderTests(unittest.TestCase):
    def setUp(self):
        driver.RelationshipProvider.records = []
        self.server = ThreadingHTTPServer(('127.0.0.1', 0), driver.RelationshipProvider)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.addCleanup(self.stop)

    def stop(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=2)

    def prompt(self):
        return '\n'.join((driver.TRUST, driver.DOUBT, driver.EDGE_ID,
            '"kind":"bible_edge"', f'profile.tagline: {driver.bible.BLUE}',
            driver.ui.MANUAL_TEXT, driver.ui.PROPOSED_TEXT))

    def request(self, user, system='targeted screenplay update', stream=True):
        request = Request(f'http://127.0.0.1:{self.server.server_port}/v1/chat/completions',
            data=json.dumps({'stream': stream, 'messages': [
                {'role': 'system', 'content': system}, {'role': 'user', 'content': user}]}).encode(),
            headers={'Content-Type': 'application/json'})
        with urlopen(request, timeout=2) as response:
            return response.headers['Content-Type'], response.read()

    def test_two_exact_canonical_previews_use_real_loopback_and_record_synthetic_provenance(self):
        for _ in range(2):
            content_type, payload = self.request(self.prompt())
            self.assertEqual(content_type, 'text/event-stream')
            chunks = [json.loads(line[6:])['choices'][0]['delta']['content']
                for line in payload.decode().splitlines() if line.startswith('data: {')]
            self.assertEqual(''.join(chunks), driver.REPLACEMENT)
        self.assertEqual([r['kind'] for r in driver.RelationshipProvider.records],
                         ['preview_relationship_1', 'preview_relationship_2'])
        self.assertTrue(all(r['accepted'] and not r['real_model'] for r in driver.RelationshipProvider.records))
        with self.assertRaises(HTTPError):
            self.request(self.prompt())

    def test_missing_relationship_or_exact_manual_evidence_cannot_qualify(self):
        for absent in [driver.TRUST, driver.DOUBT, driver.EDGE_ID, driver.ui.MANUAL_TEXT,
                       driver.ui.PROPOSED_TEXT]:
            with self.assertRaises(HTTPError):
                self.request(self.prompt().replace(absent, ''))
        self.assertFalse(any(r['accepted'] for r in driver.RelationshipProvider.records))

    def test_provider_requires_stream_and_targeted_request(self):
        with self.assertRaises(HTTPError):
            self.request(self.prompt(), stream=False)
        with self.assertRaises(HTTPError):
            self.request(self.prompt(), system='unrelated task')


class NativeControlTests(unittest.TestCase):
    def test_exact_stale_observation_keeps_showing_zero_child_accessible_leaf(self):
        from unittest.mock import MagicMock
        root, leaf = MagicMock(), MagicMock()
        leaf.__len__.return_value = 0
        leaf.__bool__.return_value = False
        leaf.childCount = 0
        leaf.getRole.return_value = 42
        leaf.getRoleName.return_value = 'paragraph'
        leaf.queryComponent.return_value.getExtents.return_value = (20, 30, 400, 20)
        observations = []
        with patch.object(driver.ui.pyatspi, 'SCROLL_ANYWHERE', 'any', create=True), \
                patch.object(driver.ui.pyatspi, 'XY_SCREEN', 'screen', create=True), \
                patch.object(driver.ui, 'walk', return_value=[root, leaf]), \
                patch.object(driver.ui, 'text_of', side_effect=lambda n: driver.STALE if n is leaf else 'Aggregate text'), \
                patch.object(driver.ui, 'visible', return_value=True) as visible:
            self.assertFalse(bool(leaf))
            self.assertTrue(driver.stale_alert(root, observations))
            self.assertEqual(observations[0][0]['child_count'], 0)
            self.assertEqual(observations[0][0]['exact_text'], driver.STALE)
            visible.return_value = False
            self.assertFalse(driver.stale_alert(root))
        leaf.queryComponent.return_value.scrollTo.assert_called_with('any')

    def test_exact_pending_disclosure_aligns_before_strict_native_pointer_input(self):
        from unittest.mock import Mock
        region, disclosure = object(), Mock()
        order = []
        disclosure.queryComponent.return_value.scrollTo.side_effect = lambda direction: order.append(('scroll', direction))
        disclosure.queryComponent.return_value.getExtents.return_value = (100, 100, 200, 20)
        with patch.object(driver.ui.pyatspi, 'SCROLL_TOP_LEFT', 'top-left', create=True), \
                patch.object(driver.ui.pyatspi, 'XY_SCREEN', 'screen', create=True), \
                patch.object(driver.ui, 'wait_for', side_effect=lambda _, check: check()), \
                patch.object(driver.ui, 'reveal', return_value=disclosure), \
                patch.object(driver.ui, 'command', return_value='X=0\nY=0\nWIDTH=1440\nHEIGHT=960'), \
                patch.object(driver.ui, 'click_control', side_effect=lambda node, window: order.append(('click', node, window))):
            driver.open_relationship_evidence(region, 'owned-window')
        self.assertEqual(order, [('scroll', 'top-left'), ('click', disclosure, 'owned-window')])
        disclosure.clear_cache.assert_called_once()

    def test_pending_review_scopes_exact_text_and_enabled_decisions_not_accepted_history(self):
        from unittest.mock import Mock
        article, history = Mock(), Mock()
        def node(name, role=0, enabled=True):
            result = Mock()
            result.name = name
            result.getRole.return_value = role
            result.getState.return_value.contains.return_value = enabled
            return result
        proposed = node('Proposed text')
        accepted = node('Proposed text')
        accept, reject = node('Accept update', 42), node('Reject', 42)
        with patch.object(driver.ui.pyatspi, 'ROLE_PUSH_BUTTON', 42, create=True), \
                patch.object(driver.ui.pyatspi, 'STATE_ENABLED', 'enabled', create=True), \
                patch.object(driver.ui, 'walk', return_value=[proposed, accept, reject]) as walk, \
                patch.object(driver.ui, 'text_of', return_value=driver.REPLACEMENT):
            self.assertTrue(driver.pending_review(article, driver.REPLACEMENT))
            walk.return_value = [proposed, accepted, accept, reject]
            self.assertFalse(driver.pending_review(history, driver.REPLACEMENT))
            walk.return_value = [proposed]
            self.assertFalse(driver.pending_review(article, driver.REPLACEMENT))
            walk.return_value = [proposed, accept, reject]
            self.assertFalse(driver.pending_review(article, 'Another screenplay'))
            accept.getState.return_value.contains.return_value = False
            self.assertFalse(driver.pending_review(article, driver.REPLACEMENT))

    def test_manual_card_edit_reveals_its_own_button_before_exact_draft_input(self):
        application, window, block, field = object(), 'owned-window', object(), object()
        with patch.object(driver.ui, 'wait_for', side_effect=lambda _, check: check()), \
                patch.object(driver.ui, 'screenplay_block', return_value=block), \
                patch.object(driver.ui, 'editable', return_value=field), \
                patch.object(driver.ui, 'reveal_button') as reveal, \
                patch.object(driver.ui, 'click_button', side_effect=AssertionError('Offscreen input is forbidden')), \
                patch.object(driver.ui, 'type_text') as type_text:
            driver.begin_manual_draft(application, window)
        reveal.assert_called_once_with(block, 'Edit', window)
        type_text.assert_called_once_with(field, window, driver.bible.DRAFT)

    def test_draft_cancel_requires_exact_draft_and_unique_enabled_native_control(self):
        from unittest.mock import Mock
        application, field = Mock(), Mock()
        field.getState.return_value.contains.return_value = True
        cancel = Mock(name='native-cancel')
        cancel.name = 'Cancel'
        cancel.getRole.return_value = 42
        cancel.getState.return_value.contains.return_value = True
        with patch.object(driver.ui.pyatspi, 'ROLE_PUSH_BUTTON', 42, create=True), \
                patch.object(driver.ui.pyatspi, 'STATE_ENABLED', 'enabled', create=True), \
                patch.object(driver.ui.pyatspi, 'STATE_EDITABLE', 'editable', create=True), \
                patch.object(driver.ui, 'wait_for', side_effect=lambda _, check: check()), \
                patch.object(driver.ui, 'editable', side_effect=AssertionError('Visible-only draft lookup is forbidden')), \
                patch.object(driver.ui, 'text_of', return_value=driver.bible.DRAFT) as text_of, \
                patch.object(driver.ui, 'reveal', side_effect=lambda _, predicate: field if predicate(field) else None) as draft_reveal, \
                patch.object(driver.ui, 'walk', return_value=[cancel]) as walk, \
                patch.object(driver.ui, 'reveal_button') as reveal:
            driver.cancel_manual_draft(application, 'owned-window')
            draft_reveal.assert_called_once()
            predicate = draft_reveal.call_args.args[1]
            text_of.return_value = 'Lookalike draft'
            self.assertFalse(predicate(field))
            text_of.return_value = driver.bible.DRAFT
            reveal.assert_called_once_with(application, 'Cancel', 'owned-window')
            reveal.reset_mock()
            walk.return_value = [cancel, cancel]
            with self.assertRaisesRegex(RuntimeError, 'unambiguous'):
                driver.cancel_manual_draft(application, 'owned-window')
            reveal.assert_not_called()


if __name__ == '__main__':
    unittest.main()
