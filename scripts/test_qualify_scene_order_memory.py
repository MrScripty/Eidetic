"""Exact-window HTTP admission; native main() is never stubbed or executed here."""
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
spec = importlib.util.spec_from_file_location('scene_order_capture', Path(__file__).with_name('qualify-scene-order-memory.py'))
driver = importlib.util.module_from_spec(spec)
with patch.dict(sys.modules, {'pyatspi': ModuleType('pyatspi'), 'gi': ModuleType('gi'), 'gi.repository': repository}):
    spec.loader.exec_module(driver)


class SceneOrderProviderTests(unittest.TestCase):
    def test_actual_http_preview_requires_entered_E_excludes_displaced_A_and_preserves_human_B(self):
        provider = driver.SceneOrderProvider
        provider.records.clear()
        server = ThreadingHTTPServer(('127.0.0.1', 0), provider)
        thread = threading.Thread(target=server.serve_forever, kwargs={'poll_interval': 0.01}, daemon=True)
        thread.start()
        try:
            def send(kind, letters, target):
                user = ''.join(f'Exact authored scene {letter}.\n\n' for letter in letters)
                user += "profile.tagline: Mara's umbrella is blue.\n" + target
                body = {'stream': True, 'messages': [
                    {'role': 'system', 'content': 'targeted screenplay update' if kind == 'preview' else 'Generate'},
                    {'role': 'user', 'content': user},
                ]}
                connection = HTTPConnection(*server.server_address, timeout=5)
                connection.request('POST', '/v1/chat/completions', json.dumps(body), {'Content-Type': 'application/json'})
                response = connection.getresponse()
                data = response.read().decode()
                connection.close()
                return response.status, data

            self.assertEqual(send('generation', 'AFCD', '')[0], 200)
            self.assertEqual(send('preview', 'AFCD', driver.MANUAL + 'context_changed')[0], 422)
            self.assertEqual(send('preview', 'AFECD', driver.MANUAL + 'context_changed')[0], 422)
            self.assertEqual(send('preview', 'FECD', driver.GENERATED + 'context_changed')[0], 422)
            status, data = send('preview', 'FECD', driver.MANUAL + 'context_changed')
            self.assertEqual(status, 200)
            text = ''.join(json.loads(event.removeprefix('data: '))['choices'][0]['delta']['content'] for event in data.strip().split('\n\n')[:-1])
            self.assertEqual(text, driver.PROPOSED)
            driver.ui.require_accepted_preview(provider.records)
            self.assertTrue(all(r['real_model'] is False for r in provider.records))
            self.assertEqual(send('preview', 'FECD', driver.MANUAL + 'context_changed')[0], 422)
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=2)


if __name__ == '__main__':
    unittest.main()
