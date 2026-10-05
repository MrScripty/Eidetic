"""Actual HTTP fixture tests; gi imports only are stubbed, native main never runs."""
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
spec = importlib.util.spec_from_file_location('canonical_capture', Path(__file__).with_name('qualify-canonical-scene-generation.py'))
driver = importlib.util.module_from_spec(spec)
with patch.dict(sys.modules, {'pyatspi': ModuleType('pyatspi'), 'gi': ModuleType('gi'), 'gi.repository': repository}):
    spec.loader.exec_module(driver)


def row(id, text):
    return ('script.document.main', 'segment.' + id, 'block.' + id,
            '00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000002', 0, 1000, text)


def user(rows, extra=''):
    return (driver.NOTES + "profile.tagline: Mara's umbrella is blue.\n" + extra
            + driver.HEADER + driver.serialized_blocks(rows) + driver.SUFFIX)


class CanonicalProviderTests(unittest.TestCase):
    def test_exact_section_refuses_swapped_extra_duplicate_and_obsolete_text(self):
        expected = [row('B', driver.ANCHOR), row('A', driver.A)]
        self.assertTrue(driver.exact_context(user(expected), expected))
        for bad in [user(expected[::-1]), user(expected + [expected[0]]), user(expected + [row('old', driver.OUTPUT)]),
                    user(expected, driver.OUTPUT), user(expected, driver.ANCHOR), user(expected).replace('block.A', 'block.stale'),
                    user(expected).replace('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000009'),
                    user(expected).replace(driver.ANCHOR, driver.ANCHOR.rstrip()), user(expected).replace(driver.NOTES, 'Old scene notes\n')]:
            with self.subTest(bad=bad[:100]):
                self.assertFalse(driver.exact_context(bad, expected))

    def test_http_admission_keeps_invalid_requests_out_of_slots_and_pauses_second_generation(self):
        provider = driver.CanonicalProvider
        provider.records.clear(); provider.delayed_seen.clear(); provider.release.clear()
        first = [row('Bmanual', driver.ANCHOR), row('A', driver.A)]
        second = [row('Bgenerated', driver.OUTPUT)] + first
        provider.expected = {'generation1': first, 'generation2': second}
        server = ThreadingHTTPServer(('127.0.0.1', 0), provider)
        thread = threading.Thread(target=server.serve_forever, kwargs={'poll_interval': 0.01}, daemon=True); thread.start()
        def send(text, stream=True):
            connection = HTTPConnection('127.0.0.1', server.server_port, timeout=5)
            connection.request('POST', '/v1/chat/completions', json.dumps({'stream': stream, 'messages': [
                {'role': 'system', 'content': 'Synthetic fixture'}, {'role': 'user', 'content': text}]}), {'Content-Type': 'application/json'})
            response = connection.getresponse(); status, body = response.status, response.read().decode(); connection.close(); return status, body
        try:
            for bad in [user(first[::-1]), user(first + [row('old', driver.OUTPUT)]), user(first, driver.OUTPUT)]:
                self.assertEqual(send(bad)[0], 422)
            self.assertEqual(send(user(first), stream=False)[0], 422)
            self.assertEqual(send(user(first))[0], 200)
            recap = f'Generate a scene recap for this screenplay beat:\n\nSCRIPT:\n{driver.OUTPUT}\n\nProduce the scene recap now. Use the exact format specified. Be concise — aim for 100-150 tokens.'
            self.assertEqual(send(recap.replace(driver.OUTPUT, driver.HUMAN))[0], 422)
            self.assertEqual(send(recap)[0], 200)
            self.assertEqual(send(recap)[0], 422)
            self.assertEqual(send(user(second[::-1]))[0], 422)
            received = []
            pending = threading.Thread(target=lambda: received.append(send(user(second)))); pending.start()
            self.assertTrue(provider.delayed_seen.wait(timeout=2)); self.assertEqual(received, [])
            provider.release.set(); pending.join(timeout=2)
            self.assertEqual(received[0][0], 200)
            self.assertIn(driver.DELAYED.rstrip(), received[0][1])
            self.assertEqual(send(user(second))[0], 422)
            self.assertEqual([record['phase'] for record in provider.records if record['accepted']], ['generation1', 'recap', 'generation2'])
            self.assertTrue(all(not record['real_model'] for record in provider.records))
        finally:
            provider.release.set(); server.shutdown(); server.server_close(); thread.join(timeout=2)


if __name__ == '__main__':
    unittest.main()
