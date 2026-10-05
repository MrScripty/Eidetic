"""Exercise exact saved-context admission through the synthetic HTTP boundary."""
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
spec = importlib.util.spec_from_file_location('child_capture', Path(__file__).with_name('qualify-child-plan-memory.py'))
driver = importlib.util.module_from_spec(spec)
with patch.dict(sys.modules, {'pyatspi': ModuleType('pyatspi'), 'gi': ModuleType('gi'), 'gi.repository': repository}):
    spec.loader.exec_module(driver)


class ChildProviderTests(unittest.TestCase):
    def setUp(self):
        driver.ChildProvider.records.clear()
        self.server = ThreadingHTTPServer(('127.0.0.1', 0), driver.ChildProvider)
        self.thread = threading.Thread(target=self.server.serve_forever, kwargs={'poll_interval': 0.01}, daemon=True)
        self.thread.start()

    def tearDown(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=2)

    def request(self, text, stream=False, system='Generate'):
        connection = HTTPConnection(*self.server.server_address, timeout=5)
        body = {'stream': stream, 'messages': [{'role': 'system', 'content': system}, {'role': 'user', 'content': text}]}
        connection.request('POST', '/v1/chat/completions', json.dumps(body), {'Content-Type': 'application/json'})
        response = connection.getresponse()
        status, data = response.status, response.read().decode()
        connection.close()
        return status, data

    def test_exact_current_saved_blocks_and_bible_required_once_per_plan(self):
        fact = f'profile.tagline: {driver.bible.BLUE}'
        for manual, name in [(driver.MIDNIGHT, 'Midnight departure'), (driver.MORNING, 'Morning departure')]:
            for missing in [manual, fact, driver.ui.PROPOSED_TEXT]:
                text = '\n'.join(value for value in [manual, fact, driver.ui.PROPOSED_TEXT] if value != missing)
                self.assertEqual(self.request(text)[0], 422)
            text = '\n'.join([manual, fact, driver.ui.PROPOSED_TEXT])
            status, data = self.request(text)
            self.assertEqual(status, 200)
            children = json.loads(json.loads(data)['choices'][0]['message']['content'])
            self.assertEqual(children[0]['name'], name)
            self.assertEqual(self.request(text)[0], 422)
        self.assertEqual(sum(record['accepted'] for record in driver.ChildProvider.records), 2)
        self.assertTrue(all(record['real_model'] is False for record in driver.ChildProvider.records))

    def test_prior_bible_stream_fixture_is_preserved(self):
        context = driver.ui.MANUAL_TEXT + '\nprofile.tagline: ' + driver.bible.RED
        status, data = self.request(context, stream=True)
        self.assertEqual(status, 200)
        self.assertIn('data: [DONE]', data)
        self.assertEqual(driver.ChildProvider.records[-1]['kind'], 'generation')
        self.assertFalse(driver.ChildProvider.records[-1]['real_model'])

    def test_exact_retired_native_child_count_reacquires_on_bounded_poll(self):
        calls = []
        def check():
            calls.append(True)
            if len(calls) == 1:
                raise ValueError('__len__() should return >= 0')
            return 'fresh native root'
        with patch.object(driver.ui.GLib, 'Error', type('NativeError', (Exception,), {}), create=True), patch.object(driver.ui.time, 'sleep'):
            self.assertEqual(driver.ui.wait_for('retired root', check), 'fresh native root')
        self.assertEqual(len(calls), 2)

    def test_unrelated_native_enumeration_failure_is_not_suppressed(self):
        def check():
            raise ValueError('unrelated failure')
        with patch.object(driver.ui.GLib, 'Error', type('NativeError', (Exception,), {}), create=True):
            with self.assertRaisesRegex(ValueError, 'unrelated failure'):
                driver.ui.wait_for('root', check)


if __name__ == '__main__':
    unittest.main()
