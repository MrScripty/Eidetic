"""Stdlib-only HTTP fixture regressions, not native GUI qualification.

Only the driver import's unavailable accessibility modules are stubbed. These
tests exercise its real localhost HTTP handler and evidence guard, never main().
"""
from concurrent.futures import ThreadPoolExecutor
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


def load_driver():
    """Load fixture helpers without installing or exercising native libraries."""
    repository = ModuleType('gi.repository')
    repository.Atspi = ModuleType('Atspi')
    repository.GLib = ModuleType('GLib')
    modules = {'pyatspi': ModuleType('pyatspi'), 'gi': ModuleType('gi'),
               'gi.repository': repository}
    path = Path(__file__).with_name('qualify-screenplay-authoring.py')
    spec = importlib.util.spec_from_file_location('qualify_screenplay_authoring', path)
    driver = importlib.util.module_from_spec(spec)
    with patch.dict(sys.modules, modules):
        spec.loader.exec_module(driver)
    return driver


driver = load_driver()


class FixtureProviderTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.server = ThreadingHTTPServer(('127.0.0.1', 0), driver.FixtureProvider)
        cls.thread = threading.Thread(
            target=cls.server.serve_forever, kwargs={'poll_interval': 0.01}, daemon=True)
        cls.thread.start()

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()
        cls.thread.join(timeout=2)

    def setUp(self):
        driver.FixtureProvider.records.clear()

    def post(self, kind, context=None, stream=True):
        """Send production-shaped prompts to the actual HTTP fixture handler."""
        expected = {'generation': driver.MANUAL_TEXT, 'recap': driver.GENERATED_TEXT,
                    'preview': driver.EDITED_TEXT}
        if context is None:
            context = expected[kind]
        system = 'Make a targeted screenplay update.' if kind == 'preview' else 'Write a scene.'
        prefix = 'Generate a scene recap for this screenplay beat:\n' if kind == 'recap' else ''
        body = {'stream': stream, 'messages': [
            {'role': 'system', 'content': system},
            {'role': 'user', 'content': prefix + context},
        ]}
        connection = HTTPConnection(*self.server.server_address, timeout=5)
        try:
            connection.request('POST', '/v1/chat/completions', json.dumps(body),
                               {'Content-Type': 'application/json'})
            response = connection.getresponse()
            return response.status, response.getheader('Content-Type'), response.read().decode()
        finally:
            connection.close()

    def test_happy_path_streams_exact_fixture_text_and_records_acceptance(self):
        expected = {'generation': driver.GENERATED_TEXT,
                    'recap': 'Eli waits at the station with Mara and the red umbrella.',
                    'preview': driver.PROPOSED_TEXT}
        for kind, text in expected.items():
            with self.subTest(kind=kind):
                status, content_type, body = self.post(kind)
                self.assertEqual(status, 200)
                self.assertEqual(content_type, 'text/event-stream')
                events = body.strip().split('\n\n')
                self.assertEqual(events[-1], 'data: [DONE]')
                actual = ''.join(json.loads(event.removeprefix('data: '))
                                 ['choices'][0]['delta']['content'] for event in events[:-1])
                self.assertEqual(actual, text)
                self.assertEqual(driver.FixtureProvider.records[-1], {
                    'kind': kind, 'contains_exact_expected_context': True,
                    'stream': True, 'real_model': False, 'accepted': True,
                })
        driver.require_accepted_preview(driver.FixtureProvider.records)

    def test_rejected_context_does_not_consume_kind_or_three_acceptance_limit(self):
        for kind in ('generation', 'recap', 'preview'):
            with self.subTest(kind=kind):
                self.assertEqual(self.post(kind, context='Incorrect context')[0], 422)
                self.assertEqual(self.post(kind)[0], 200)
        records = driver.FixtureProvider.records
        self.assertEqual(len(records), 6)
        self.assertEqual([r['accepted'] for r in records], [False, True] * 3)
        self.assertEqual([r['contains_exact_expected_context'] for r in records],
                         [False, True] * 3)
        driver.require_accepted_preview(records)

    def test_non_streaming_requests_are_rejected_without_poisoning_valid_retry(self):
        for stream in (False, 1, 'true', None):
            with self.subTest(stream=stream):
                self.assertEqual(self.post('generation', stream=stream)[0], 422)
                self.assertIs(driver.FixtureProvider.records[-1]['accepted'], False)
                self.assertIs(driver.FixtureProvider.records[-1]['stream'], False)
                self.assertIs(driver.FixtureProvider.records[-1]['contains_exact_expected_context'], True)
        self.assertEqual(self.post('generation')[0], 200)
        self.assertEqual(len(driver.FixtureProvider.records), 5)
        self.assertIs(driver.FixtureProvider.records[-1]['accepted'], True)

    def test_accepted_kind_duplicates_stay_rejected_and_preserved(self):
        for kind in ('generation', 'recap', 'preview'):
            with self.subTest(kind=kind):
                self.assertEqual(self.post(kind)[0], 200)
                self.assertEqual(self.post(kind)[0], 422)
        records = driver.FixtureProvider.records
        self.assertEqual([r['accepted'] for r in records], [True, False] * 3)
        self.assertTrue(all(r['contains_exact_expected_context'] for r in records))
        with self.assertRaisesRegex(RuntimeError, 'production HTTP provider client'):
            driver.require_accepted_preview(records)

    def test_rejected_preview_cannot_qualify_without_an_accepted_preview(self):
        self.assertEqual(self.post('preview', context=driver.MANUAL_TEXT)[0], 422)
        with self.assertRaisesRegex(RuntimeError, 'production HTTP provider client'):
            driver.require_accepted_preview(driver.FixtureProvider.records)
        self.assertEqual(self.post('preview')[0], 200)
        driver.require_accepted_preview(driver.FixtureProvider.records)

    def test_preview_guard_requires_explicit_acceptance_on_latest_record(self):
        invalid_records = ([], [{'kind': 'preview'}],
                           [{'kind': 'preview', 'accepted': False}],
                           [{'kind': 'preview', 'accepted': 1}],
                           [{'kind': 'generation', 'accepted': True}],
                           [{'kind': 'preview', 'accepted': True},
                            {'kind': 'recap', 'accepted': True}])
        for records in invalid_records:
            with self.subTest(records=records):
                with self.assertRaisesRegex(RuntimeError, 'production HTTP provider client'):
                    driver.require_accepted_preview(records)
        driver.require_accepted_preview([{'kind': 'preview', 'accepted': True}])

    def test_concurrent_duplicates_accept_only_one_request(self):
        with ThreadPoolExecutor(max_workers=6) as pool:
            statuses = list(pool.map(lambda _: self.post('generation')[0], range(6)))
        self.assertEqual(sorted(statuses), [200] + [422] * 5)
        records = driver.FixtureProvider.records
        self.assertEqual(len(records), 6)
        self.assertEqual(sum(r['accepted'] for r in records), 1)


if __name__ == '__main__':
    unittest.main()
