"""Feature-union regressions for the no-download build admission gate."""
import importlib.util
from contextlib import redirect_stdout
import io
import json
from pathlib import Path
import runpy
import sys
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('gate', Path(__file__).with_name('check-onnx-no-download.py'))
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


def metadata(ort, ort_sys):
    return {
        'packages': [{'id': name, 'name': name, 'version': '2.0.0-rc.12'} for name in ('ort', 'ort-sys')],
        'resolve': {'nodes': [{'id': 'ort', 'features': ort}, {'id': 'ort-sys', 'features': ort_sys}]},
    }


class NoDownloadTests(unittest.TestCase):
    def test_cli_reads_non_ascii_cargo_json_as_utf8_under_windows_default_encoding(self):
        data = metadata(['load-dynamic'], ['disable-linking'])
        data['workspace_root'] = 'C:/Users/あ/Eidetic'
        original_read_text = Path.read_text
        encodings = []

        def windows_read_text(path, encoding=None, errors=None):
            encodings.append(encoding)
            return original_read_text(path, encoding=encoding or 'cp1252', errors=errors)

        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'cargo-metadata.json'
            path.write_bytes(json.dumps(data, ensure_ascii=False).encode('utf-8'))
            output = io.StringIO()
            # Run the actual CLI, emulating the failing Windows locale default
            # without depending on the test host's locale or Python UTF-8 mode.
            with patch.object(Path, 'read_text', windows_read_text), patch.object(
                sys, 'argv', [gate.__file__, str(path)]
            ), redirect_stdout(output):
                runpy.run_path(gate.__file__, run_name='__main__')
            self.assertEqual(encodings, ['utf-8'])
            self.assertEqual(json.loads(output.getvalue()), gate.check(data))

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
