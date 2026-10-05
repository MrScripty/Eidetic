"""Test exact fact/manual-context admission in the real synthetic HTTP handler."""
from http.client import HTTPConnection
from http.server import ThreadingHTTPServer
import importlib.util
import json
from pathlib import Path
import sys
import sqlite3
import tempfile
import threading
from types import ModuleType
import unittest
from unittest.mock import patch

repository = ModuleType('gi.repository')
repository.Atspi = ModuleType('Atspi')
repository.GLib = ModuleType('GLib')
spec = importlib.util.spec_from_file_location('bible_fact_capture', Path(__file__).with_name('qualify-bible-fact-propagation.py'))
driver = importlib.util.module_from_spec(spec)
with patch.dict(sys.modules, {'pyatspi': ModuleType('pyatspi'), 'gi': ModuleType('gi'), 'gi.repository': repository}):
    spec.loader.exec_module(driver)


class BibleFactProviderTests(unittest.TestCase):
    def test_readonly_fact_checkpoint_works_before_screenplay_schema_exists(self):
        with tempfile.TemporaryDirectory() as directory:
            database = Path(directory) / 'fixture.db'
            with sqlite3.connect(database) as connection:
                connection.execute('CREATE TABLE bible_graph_fields (id TEXT, text_value TEXT, updated_event_id TEXT)')
                connection.execute('INSERT INTO bible_graph_fields VALUES (?, ?, ?)', ('qualification.mara.tagline', driver.RED, 'red-revision'))
            self.assertEqual(driver.fact(database), (driver.RED, 'red-revision'))
            with sqlite3.connect(database) as connection:
                self.assertEqual(connection.execute("SELECT count(*) FROM sqlite_master WHERE name='script_blocks'").fetchone()[0], 0)

    def test_generation_and_preview_require_exact_fact_and_manual_context(self):
        provider = driver.BibleFactProvider
        provider.records.clear()
        server = ThreadingHTTPServer(('127.0.0.1', 0), provider)
        thread = threading.Thread(target=server.serve_forever, kwargs={'poll_interval': 0.01}, daemon=True)
        thread.start()
        try:
            for kind, fact, expected in [('generation', driver.RED, driver.ui.GENERATED_TEXT), ('preview', driver.BLUE, driver.ui.PROPOSED_TEXT)]:
                context = driver.ui.MANUAL_TEXT + '\n' + f'profile.tagline: {fact}' + '\n' + driver.ui.GENERATED_TEXT
                for valid in [False, True, True]:
                    body = {'stream': True, 'messages': [
                        {'role': 'system', 'content': 'targeted screenplay update' if kind == 'preview' else 'Generate'},
                        {'role': 'user', 'content': context if valid else driver.ui.MANUAL_TEXT},
                    ]}
                    connection = HTTPConnection(*server.server_address, timeout=5)
                    connection.request('POST', '/v1/chat/completions', json.dumps(body), {'Content-Type': 'application/json'})
                    response = connection.getresponse()
                    data = response.read().decode()
                    connection.close()
                    accepted = provider.records[-1]['accepted']
                    self.assertFalse(provider.records[-1]['real_model'])
                    self.assertEqual(response.status, 200 if accepted else 422)
                    if accepted:
                        text = ''.join(json.loads(event.removeprefix('data: '))['choices'][0]['delta']['content'] for event in data.strip().split('\n\n')[:-1])
                        self.assertEqual(text, expected)
                self.assertEqual([r['accepted'] for r in provider.records if r['kind'] == kind], [False, True, False])
            with self.assertRaisesRegex(RuntimeError, 'production HTTP provider'):
                driver.ui.require_accepted_preview(provider.records)
            driver.ui.require_accepted_preview([r for r in provider.records if r['accepted']])
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=2)


if __name__ == '__main__':
    unittest.main()
