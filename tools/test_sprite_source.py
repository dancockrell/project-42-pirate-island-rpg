import copy
import hashlib
from pathlib import Path
import tempfile
import unittest
from PIL import Image
from inspect_sprite_source import inspect_source


class SpriteSourceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name) / "fixture.png"
        image = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
        image.putpixel((8, 8), (255, 0, 0, 255))
        image.save(self.path)
        self.manifest = {"schemaVersion": 1, "sourceSha256": hashlib.sha256(self.path.read_bytes()).hexdigest(),
            "frames": [{"id": "idle.south.0", "action": "idle", "direction": "south", "rect": [0, 0, 16, 16],
                        "footPivot": [8, 16], "durationSeconds": .1, "loop": True}]}

    def test_valid_structure_is_not_art_approval(self):
        result = inspect_source(self.path, self.manifest)
        self.assertTrue(result["structuralAdmission"])
        self.assertEqual(result["artAdmission"], "not-assessed")

    def test_no_manifest_rejected(self):
        self.assertFalse(inspect_source(self.path)["structuralAdmission"])

    def test_stale_source_rejected(self):
        self.manifest["sourceSha256"] = "0" * 64
        self.assertFalse(inspect_source(self.path, self.manifest)["structuralAdmission"])

    def test_opaque_checkerboard_rejected(self):
        image = Image.new("RGB", (16, 16), "white")
        image.putpixel((8, 8), (200, 200, 200))
        image.save(self.path)
        self.manifest["sourceSha256"] = hashlib.sha256(self.path.read_bytes()).hexdigest()
        self.assertFalse(inspect_source(self.path, self.manifest)["structuralAdmission"])

    def test_bad_frame_fields_rejected(self):
        for field, value in [("rect", [-1,0,16,16]), ("rect", [0,0,17,16]), ("rect", [True,0,16,16]),
                             ("footPivot", [17,8]), ("durationSeconds", float("nan")), ("durationSeconds", True),
                             ("loop", 1), ("action", ""), ("direction", None)]:
            with self.subTest(field=field, value=value):
                manifest = copy.deepcopy(self.manifest)
                manifest["frames"][0][field] = value
                self.assertFalse(inspect_source(self.path, manifest)["structuralAdmission"])

    def test_duplicate_ids_rejected(self):
        self.manifest["frames"] *= 2
        self.assertFalse(inspect_source(self.path, self.manifest)["structuralAdmission"])

    def test_empty_frame_rejected(self):
        self.manifest["frames"][0]["rect"] = [0,0,4,4]
        self.manifest["frames"][0]["footPivot"] = [2,4]
        self.assertFalse(inspect_source(self.path, self.manifest)["structuralAdmission"])

    def test_read_only(self):
        before = self.path.read_bytes()
        inspect_source(self.path, self.manifest)
        self.assertEqual(self.path.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
