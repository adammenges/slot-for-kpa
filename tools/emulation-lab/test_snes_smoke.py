import io
import unittest
import zipfile
from snes_smoke import fixtures
from make_snes_probe import rom

class SnesSmokeFixtureTests(unittest.TestCase):
    def test_every_launch_format_contains_the_expected_original_program(self):
        files=fixtures()
        self.assertEqual(len(files),5)
        for name, content in files.items():
            if name.endswith('.zip'):
                with zipfile.ZipFile(io.BytesIO(content)) as archive:
                    self.assertEqual(archive.read('Probe.sfc'),rom())
            elif name.endswith('.smc'):
                self.assertEqual(content[:512],bytes(512))
                self.assertEqual(content[512:],rom())
            else:
                self.assertEqual(len(content),32768)
                self.assertEqual(content[0x7fc0:0x7fd5],b'SLOT SNES INPUT PROBE')
