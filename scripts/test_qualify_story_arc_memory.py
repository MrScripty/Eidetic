"""Deterministic fixture admission tests; no GUI simulation or model inference."""
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
spec = importlib.util.spec_from_file_location('arc_memory_capture', Path(__file__).with_name('qualify-story-arc-memory.py'))
driver = importlib.util.module_from_spec(spec)
with patch.dict(sys.modules, {'pyatspi': ModuleType('pyatspi'), 'gi': ModuleType('gi'), 'gi.repository': repository}):
    spec.loader.exec_module(driver)


class ArcProviderTests(unittest.TestCase):
    def test_exact_canonical_context_and_bounded_two_preview_sequence(self):
        provider = driver.ArcProvider
        provider.records = []
        provider.arc_id = 'arc.fixture'
        server = ThreadingHTTPServer(('127.0.0.1', 0), provider)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()

        def send(kind, user):
            client = HTTPConnection('127.0.0.1', server.server_port, timeout=5)
            payload = {'stream': True, 'messages': [
                {'role': 'system', 'content': 'targeted screenplay update' if kind == 'preview' else 'screenwriter'},
                {'role': 'user', 'content': user},
            ]}
            client.request('POST', '/v1/chat/completions', json.dumps(payload), {'Content-Type': 'application/json'})
            response = client.getresponse()
            result = response.status, response.read().decode()
            client.close()
            return result

        try:
            initial = f'STORY ARCS: Witness (APlot) — {driver.OLD}\n{driver.F_TEXT}'
            current = f'CURRENT TAGGED STORY ARC FIELDS:\narc.fixture description: {driver.NEW}\n{driver.F_TEXT}{driver.MANUAL}'
            self.assertEqual(send('generation', initial.replace(driver.OLD, driver.NEW))[0], 422)
            self.assertEqual(send('generation', initial)[0], 200)
            self.assertEqual(send('recap', 'Generate a scene recap for this screenplay beat:' + driver.GENERATED)[0], 200)
            for bad in [current.replace(driver.NEW, driver.OLD), current.replace('arc.fixture', 'wrong.arc'), current + driver.DRAFT, current.replace(driver.MANUAL, driver.GENERATED)]:
                self.assertEqual(send('preview', bad)[0], 422)
            for expected in [driver.STALE, driver.PROPOSED]:
                status, data = send('preview', current)
                self.assertEqual(status, 200)
                result = ''.join(json.loads(event.removeprefix('data: '))['choices'][0]['delta']['content'] for event in data.strip().split('\n\n')[:-1])
                self.assertEqual(result, expected)
                driver.ui.require_accepted_preview(provider.records)
            self.assertEqual(send('preview', current)[0], 422)
            accepted = [item for item in provider.records if item['accepted']]
            self.assertEqual([item['kind'] for item in accepted], provider.sequence)
            self.assertTrue(all(item['real_model'] is False for item in accepted))
        finally:
            server.shutdown()
            server.server_close()


if __name__ == '__main__':
    unittest.main()
