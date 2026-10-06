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


if __name__ == '__main__':
    unittest.main()
