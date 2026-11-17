"""Exercise the shipped WASM itself. Run: python -m unittest discover -s tests -v"""
import struct
import unittest
from zipfile import ZipFile
from pathlib import Path
import wasmtime
ROOT = Path(__file__).resolve().parents[1]

class PortTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.engine = wasmtime.Engine()
        cls.module = wasmtime.Module.from_file(cls.engine, str(ROOT / 'dist/port/ppk.wasm'))

    def setUp(self):
        self.store = wasmtime.Store(self.engine)
        self.e = wasmtime.Instance(self.store, self.module, []).exports(self.store)
        self.call('init')
        self.memory = self.e['memory']
        self.base = self.call('data_pointer')
        self.data = self.base + 0xf0a0

    def call(self, name, *args):
        return self.e[name](self.store, *args)

    def state(self, offset):
        return self.call('debug_state', offset)

    def put(self, offset, value):
        self.memory.write(self.store, struct.pack('<H', value & 65535), self.data + offset)

    def ticks(self, count):
        for _ in range(count):
            self.call('tick')
            self.assertEqual(self.call('get_error'), 0)

    def test_standalone_module_and_initial_stage(self):
        self.assertEqual(len(self.module.imports), 0)
        self.assertEqual((self.state(0x561), self.state(0x563)), (248, 168))
        self.assertEqual(self.state(0x13fa), 10)
        self.assertEqual(self.state(0x1398), 5)

    def test_intro_waits_for_space_and_restores_stage_palette(self):
        self.call('show_intro')
        self.assertEqual(self.call('get_phase'), 3)
        fb = self.call('framebuffer')
        title = self.memory.read(self.store, fb, fb + 256000)
        self.ticks(12)
        self.assertEqual(self.call('get_phase'), 3)
        self.assertEqual(self.state(0x13fa), 10)
        self.call('key', 57, 1)
        self.ticks(1)
        self.assertEqual(self.call('get_phase'), 0)
        self.assertEqual((self.state(0x561), self.state(0x563)), (248, 168))
        self.assertNotEqual(title, self.memory.read(self.store, fb, fb + 256000))

    def test_left_movement_and_wall(self):
        self.call('key', 75, 1)
        self.ticks(10)
        self.assertEqual(self.state(0x561), 221)
        self.call('init')
        self.call('key', 77, 1)
        self.ticks(10)
        self.assertEqual(self.state(0x561), 248)

    def test_original_jump_curve(self):
        self.call('key', 57, 1)
        heights = []
        for _ in range(10):
            self.ticks(1)
            heights.append(self.state(0x563))
        self.assertEqual(heights, [168,166,164,163,162,161,161,163,165,168])

    def test_ladder(self):
        self.put(0x561, 128)
        self.call('key', 72, 1)
        self.ticks(16)
        self.assertEqual(self.state(0x563), 136)
        self.call('key', 72, 0)
        self.call('key', 80, 1)
        self.ticks(16)
        self.assertEqual(self.state(0x563), 168)

    def test_fruit_and_score(self):
        self.put(0x561, 80)
        self.put(0x58b, 0)
        self.ticks(1)
        self.assertEqual(self.state(0x13fa), 9)
        self.assertEqual(self.state(0xcb6), 1)
        self.ticks(1)
        self.assertEqual(self.state(0x13fa), 9)

    def test_death_and_respawn(self):
        self.put(0x583, 1)
        self.ticks(1)
        self.assertEqual(self.call('get_phase'), 1)
        self.ticks(13)
        self.assertEqual(self.call('get_phase'), 0)
        self.assertEqual(self.state(0x1398), 4)
        self.assertEqual((self.state(0x561), self.state(0x563)), (248,168))
        self.assertEqual(self.state(0x13fa), 10)

    def test_timeout(self):
        self.put(0x1396, 134)
        self.put(0x139c, 29)
        self.ticks(1)
        self.assertEqual(self.state(0x583), 1)

    def test_clear_transition(self):
        # Fixture: last fruit remains, at the player's position.
        self.put(0x13fa, 1)
        self.put(0x561, 80)
        self.put(0x58b, 0)
        self.ticks(1)
        self.assertEqual(self.state(0x13fa), 0)
        self.assertEqual(self.call('get_phase'), 4)
        self.ticks(1)  # release gate prevents a held jump from skipping the clear screen
        self.call('key', 57, 1)
        self.ticks(1)
        self.assertEqual(self.call('get_phase'), 0)
        self.assertEqual(self.call('get_stage'), 2)
        self.assertEqual(self.state(0x13fa), 8)

    def test_all_fifty_stage_maps_and_rendering(self):
        self.assertEqual(self.call('stage_count'), 50)
        with ZipFile(ROOT / 'original/ppk.zip') as archive:
            maps = archive.read('ppk/STAGE.PPK')
        for stage in range(1, 51):
            with self.subTest(stage=stage):
                self.call('start_stage', stage)
                self.assertEqual(self.call('get_stage'), stage)
                expected = maps[(stage-1)*330:stage*330]
                actual = self.memory.read(self.store, self.data+0x1244, self.data+0x1244+330)
                # Original constructor consumes enemy spawn markers into actor arrays.
                self.assertEqual(actual, expected.translate(bytes.maketrans(b'stuv', b'    ')))
                self.assertEqual(self.state(0x13fa), expected.count(b'o'))
                scenery = self.memory.read(self.store, self.base+0x9220+0x700, self.base+0x9220+0x780)
                original = self.memory.read(self.store, self.base+0xa220+(stage%27)*128, self.base+0xa220+(stage%27+1)*128)
                self.assertEqual(scenery, original)
                for tick in range(100):
                    self.call('key', 75 if tick < 50 else 77, 1)
                    self.ticks(1)
                    self.call('render_interpolated', 500)
                    self.assertEqual(self.call('get_error'), 0)

    def test_stage_progression_preserves_score_lives_and_wraps(self):
        for stage in range(1, 51):
            self.assertEqual(self.call('get_stage'), stage)
            self.put(0xcb6, 1234)
            self.put(0x1398, 3)
            self.put(0x13fa, 0)  # clear fixture; gameplay collection is tested separately
            self.ticks(1)
            self.assertEqual(self.call('get_phase'), 4)
            self.ticks(1)
            self.call('key', 57, 1)
            self.ticks(1)
            self.assertEqual(self.call('get_phase'), 0)
            self.assertEqual(self.call('get_stage'), stage % 50 + 1)
            self.assertEqual(self.state(0xcb6), 1234)
            self.assertEqual(self.state(0x1398), 3)

    def test_selected_stage_survives_intro_and_respawn(self):
        self.call('start_stage', 27)
        self.call('show_intro')
        self.call('key', 57, 1)
        self.ticks(1)
        self.assertEqual(self.call('get_stage'), 27)
        self.put(0x583, 1)
        self.ticks(14)
        self.assertEqual(self.call('get_stage'), 27)
        self.assertEqual(self.state(0x1398), 4)

    def test_continue_preserves_stage_and_score(self):
        self.call('start_stage', 50)
        self.put(0xcb6, 1234)
        self.put(0x1398, 0)
        self.put(0x583, 1)
        self.ticks(14)
        self.assertEqual(self.call('get_phase'), 2)
        self.ticks(1)
        self.call('key', 28, 1)
        self.ticks(1)
        self.assertEqual(self.call('get_phase'), 0)
        self.assertEqual(self.call('get_stage'), 50)
        self.assertEqual(self.state(0xcb6), 1234)
        self.assertEqual(self.state(0x1398), 5)
        self.assertEqual(self.state(0x159a), 4)

    def test_finish_signal_only_latches_on_last_candy_of_stage_fifty(self):
        self.put(0x13fa, 0)
        self.ticks(1)
        self.assertEqual(self.call('run_finished'), 0)  # stage 1 clear is not a finish
        self.call('start_stage', 50)
        self.assertEqual(self.call('run_finished'), 0)
        # Put the player at an actual candy, with only that final candy remaining.
        grid = self.memory.read(self.store, self.data+0xcec, self.data+0xcec+960)
        index = next(i for i, tile in enumerate(grid) if tile == 5)
        self.put(0x561, (index % 40 + 1) * 8)
        self.put(0x563, (index // 40) * 8 + 16)
        self.put(0x58b, 0)
        for count in [0x13f0, 0x1592, 0x13fc]:
            self.put(count, 0)
        self.put(0x13fa, 1)
        self.ticks(1)
        self.assertEqual(self.state(0x13fa), 0)
        self.assertEqual(self.call('run_finished'), 1)
        self.ticks(1)
        self.call('key', 57, 1)
        self.ticks(1)
        self.assertEqual(self.call('get_stage'), 1)
        self.assertEqual(self.call('run_finished'), 1)  # result stays final after wrap
        self.call('start_stage', 1)
        self.assertEqual(self.call('run_finished'), 0)

    def test_invalid_stage_does_not_reset_game(self):
        self.call('start_stage', 7)
        self.call('start_stage', 0)
        self.call('start_stage', 51)
        self.assertEqual(self.call('get_stage'), 7)

    def test_interpolation_is_read_only_and_endpoint_matches(self):
        self.call('key', 75, 1)
        for tick in range(120):
            if tick == 40:
                self.call('key', 75, 0)
                self.call('key', 57, 1)
            self.ticks(1)
            fb = self.call('framebuffer')
            endpoint = self.memory.read(self.store, fb, fb + 256000)
            before = self.memory.read(self.store, self.base, self.base + 0x50000)
            self.call('render_interpolated', 500)
            after = self.memory.read(self.store, self.base, self.base + 0x50000)
            self.assertEqual(before, after)
            self.call('render_interpolated', 1000)
            self.assertEqual(endpoint, self.memory.read(self.store, fb, fb + 256000))

if __name__ == '__main__':
    unittest.main()
