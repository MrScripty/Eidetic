"""Exercise native qualification evidence guards and its actual HTTP boundary."""
import importlib.util
import json
from pathlib import Path
import sqlite3
import sys
import tempfile
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
    'bible_plan_capture', Path(__file__).with_name('qualify-child-plan-bible.py'))
driver = importlib.util.module_from_spec(spec)
with patch.dict(sys.modules, {'pyatspi': ModuleType('pyatspi'), 'gi': ModuleType('gi'),
                             'gi.repository': repository}):
    spec.loader.exec_module(driver)


class BiblePlanProviderTests(unittest.TestCase):
    def setUp(self):
        driver.BiblePlanProvider.records = []
        self.server = ThreadingHTTPServer(('127.0.0.1', 0), driver.BiblePlanProvider)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.addCleanup(self.stop)

    def stop(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=2)

    def request(self, user, stream=False, system=''):
        request = Request(f'http://127.0.0.1:{self.server.server_port}/v1/chat/completions',
            data=json.dumps({'stream': stream, 'messages': [
                {'role': 'system', 'content': system}, {'role': 'user', 'content': user}]}).encode(),
            headers={'Content-Type': 'application/json'})
        with urlopen(request, timeout=2) as response:
            return response.headers['Content-Type'], response.read()

    def prompt(self, fact):
        return '\n'.join((f'profile.tagline: {fact}', driver.ui.MANUAL_TEXT,
            driver.ui.PROPOSED_TEXT, driver.EDGE, 'qualification.mara qualification.eli'))

    def test_real_loopback_records_exact_blue_gold_and_independent_screenplay_acceptance(self):
        blue = self.request(self.prompt(driver.bible.BLUE))
        gold = self.request(self.prompt(driver.GOLD))
        self.assertIn(b'Blue departure', blue[1])
        self.assertIn(b'Gold departure', gold[1])
        preview = self.request(self.prompt(driver.GOLD), True, 'targeted screenplay update')
        self.assertEqual(preview[0], 'text/event-stream')
        chunks = [json.loads(line[6:])['choices'][0]['delta']['content']
            for line in preview[1].decode().splitlines() if line.startswith('data: {')]
        self.assertEqual(''.join(chunks), driver.GOLD_SCREENPLAY)
        self.assertEqual([r['kind'] for r in driver.BiblePlanProvider.records],
            ['child_blue', 'child_gold', 'preview_gold'])
        self.assertTrue(all(r['accepted'] and not r['real_model'] for r in driver.BiblePlanProvider.records))

    def test_missing_relationship_or_exact_manual_text_refuses_and_duplicate_is_recorded(self):
        for missing in (driver.EDGE, driver.ui.MANUAL_TEXT):
            with self.assertRaises(HTTPError) as error:
                self.request(self.prompt(driver.GOLD).replace(missing, ''))
            self.assertEqual(error.exception.code, 422)
        self.request(self.prompt(driver.GOLD))
        with self.assertRaises(HTTPError):
            self.request(self.prompt(driver.GOLD))
        self.assertEqual([r['accepted'] for r in driver.BiblePlanProvider.records], [False, False, True, False])


class BiblePlanReceiptTests(unittest.TestCase):
    def test_original_receipt_read_only_and_missing_legacy_evidence_refuses(self):
        with tempfile.TemporaryDirectory() as directory:
            database = Path(directory) / 'story.sqlite'
            with sqlite3.connect(database) as connection:
                connection.executescript('''
                    CREATE TABLE commands(id TEXT,payload_type TEXT,payload_json TEXT);
                    CREATE TABLE change_events(id TEXT,command_id TEXT);
                    CREATE TABLE child_plans(id TEXT,created_event_id TEXT);
                    CREATE TABLE script_blocks(id TEXT,text TEXT);
                ''')
                memory = {'bible': {'receipt': {'context': {'payload': {'nodes': [
                    {'name': 'Mara', 'fields': [{'value': driver.bible.BLUE}]}]}}}}, 'script_inputs': []}
                connection.execute('INSERT INTO commands VALUES(?,?,?)',
                    ('create', 'ai.child_plan_create', json.dumps({'memory': memory})))
                connection.execute('INSERT INTO change_events VALUES(?,?)', ('event', 'create'))
                connection.execute('INSERT INTO child_plans VALUES(?,?)', ('plan', 'event'))
                connection.commit()
                before = driver.recovery.database_snapshot(database)
                self.assertEqual(driver.stored_memory(database, 'plan'), memory)
                driver.require_refusal_unchanged(before, driver.recovery.database_snapshot(database))
                connection.execute("UPDATE commands SET payload_json='{}'")
                connection.commit()
                with self.assertRaisesRegex(RuntimeError, 'Missing original Bible custody'):
                    driver.stored_memory(database, 'plan')
                with self.assertRaisesRegex(RuntimeError, 'changed canonical state or history'):
                    driver.require_refusal_unchanged(before, driver.recovery.database_snapshot(database))


if __name__ == '__main__':
    unittest.main()
