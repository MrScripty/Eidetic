"""Feature-union regressions for the no-download build admission gate."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('gate', Path(__file__).with_name('check-onnx-no-download.py'))
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


def metadata(ort, ort_sys):
    return {
        'packages': [{'id': name, 'name': name, 'version': '2.0.0-rc.12'} for name in ('ort', 'ort-sys')],
        'resolve': {'nodes': [{'id': 'ort', 'features': ort}, {'id': 'ort-sys', 'features': ort_sys}]},
    }


class NoDownloadTests(unittest.TestCase):
    def test_qualified_dynamic_loading_is_allowed(self):
        result = gate.check(metadata(['std', 'load-dynamic'], ['std', 'disable-linking']))
        self.assertEqual(result['ort']['version'], '2.0.0-rc.12')

    def test_old_pumas_download_graph_is_refused(self):
        with self.assertRaisesRegex(ValueError, 'forbidden'):
            gate.check(metadata(['std', 'download-binaries'], ['download-binaries']))

    def test_additive_download_feature_is_refused_even_with_disabled_linking(self):
        with self.assertRaisesRegex(ValueError, 'ort-sys enables forbidden'):
            gate.check(metadata(['load-dynamic'], ['disable-linking', 'download-binaries']))

    def test_missing_dynamic_loading_cannot_qualify(self):
        with self.assertRaisesRegex(ValueError, 'must enable load-dynamic'):
            gate.check(metadata(['std'], ['std']))


if __name__ == '__main__':
    unittest.main()
