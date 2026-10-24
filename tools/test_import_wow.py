"""Small fixtures catch corrupt ranges, wrong handedness and external dependencies."""
import io
import json
import struct
import unittest
import os
from pathlib import Path

import import_wow as wow
from PIL import Image


def triangle_files():
    data = bytearray(512)
    struct.pack_into('<4sI', data, 0, b'MD20', 264)
    struct.pack_into('<II', data, 60, 3, 304)
    for i, position in enumerate([(0,0,0), (1,0,0), (0,1,0)]):
        struct.pack_into('<3f4B4B3f4f', data, 304 + i * 48, *position,
                         255,0,0,0, 0,0,0,0, 0,0,1, 0,0,0,0)
    struct.pack_into('<II', data, 80, 1, 448)
    struct.pack_into('<4I', data, 448, 11, 0, 0, 0)
    struct.pack_into('<II', data, 112, 1, 464)
    struct.pack_into('<II', data, 128, 1, 468)
    skin = bytearray(136)
    struct.pack_into('<4s10I', skin, 0, b'SKIN', 3,48, 3,54, 0,0, 1,60, 1,108)
    struct.pack_into('<6H', skin, 48, 0,1,2, 0,1,2)
    struct.pack_into('<10H7f', skin, 60, 0,0,0,3,0,3,0,0,0,0,*([0.]*7))
    struct.pack_into('<Bb11H', skin, 108, 16,0,0,0,0,65535,0,0,1,0,0,0,0)
    image = io.BytesIO()
    Image.new('RGBA', (2,2), (255,80,20,255)).save(image, format='PNG')
    return {'model.M2': data, 'model00.skin': bytes(skin), 'neutral.blp': image.getvalue()}


def skeletal_files():
    files = triangle_files()
    data = files['model.M2']
    data.extend(b'\x00' * 512)
    struct.pack_into('<II', data, 44, 1, 512)
    struct.pack_into('<iIhHHH', data, 512, -1, 0, -1, 0, 0, 0)
    struct.pack_into('<3f', data, 588, 1.,2.,3.)
    struct.pack_into('<II', data, 28, 2, 608)
    for i, animation in enumerate([0,4]):
        struct.pack_into('<HHIfIhhIII6ffhh', data, 608+i*64,
                         animation,0,1000,0.,32,0,0,0,0,0,*([0.]*7),-1,0)
    struct.pack_into('<Hh4I', data, 528, 1,-1,2,800,2,816)
    struct.pack_into('<8I', data, 800, 1,832,1,836,1,840,1,852)
    struct.pack_into('<2I', data, 832, 0,0)
    struct.pack_into('<6f', data, 840, 1.,0.,0.,2.,0.,0.)
    return files


class ConversionTests(unittest.TestCase):
    @unittest.skipUnless(os.environ.get('WOW_STORMLIB'), 'Set WOW_STORMLIB for local MPQ integration')
    def test_archive_reader_skips_missing_files_and_reads_model(self):
        mpq = wow.Mpq(Path(__file__).resolve().parents[2], Path(os.environ['WOW_STORMLIB']))
        try:
            with self.assertRaises(FileNotFoundError):
                mpq.read('DefinitelyNotARealAsset-leviathan.m2')
            self.assertEqual(mpq.read(r'Creature\DemolisherVehicle\DemolisherVehicle.M2')[:4], b'MD20')
        finally:
            mpq.close()

    def test_array_rejects_truncated_and_hostile_offsets(self):
        self.assertEqual(wow.records(b'\x03\x00\x05\x00', 0, 2, '<H'), [(3,), (5,)])
        for offset, count in [(3, 1), (0, 999999999), (-1, 1)]:
            with self.assertRaises(ValueError):
                wow.records(b'\x00' * 4, offset, count, '<H')

    def test_wow_forward_becomes_negative_z_without_mirroring(self):
        self.assertEqual(wow.coordinates((1., 2., 3.)), (-2., 3., -1.))

    def test_skin_indices_resolve_local_vertex_lookup(self):
        data = bytearray(68)
        struct.pack_into('<4s10I', data, 0, b'SKIN', 3, 48, 3, 54, 0, 0, 0, 0, 0, 0)
        struct.pack_into('<6H', data, 48, 2, 0, 1, 0, 1, 2)
        self.assertEqual(wow.skin_indices(bytes(data), 3), [2, 0, 1])
        struct.pack_into('<H', data, 54, 8)
        with self.assertRaises(ValueError):
            wow.skin_indices(bytes(data), 3)

    def test_glb_embeds_image_and_binary_with_aligned_views(self):
        image = io.BytesIO()
        Image.new('RGBA', (2, 2), (80, 100, 120, 255)).save(image, format='PNG')
        builder = wow.Glb()
        tex = builder.texture(image.getvalue())
        builder.document['materials'].append({'pbrMetallicRoughness': {'baseColorTexture': {'index': tex}}})
        data = builder.finish()
        self.assertEqual(struct.unpack_from('<4sII', data), (b'glTF', 2, len(data)))
        n = struct.unpack_from('<I', data, 12)[0]
        doc = json.loads(data[20:20+n])
        self.assertNotIn('uri', doc['buffers'][0])
        self.assertNotIn('uri', doc['images'][0])
        self.assertEqual(doc['images'][0]['mimeType'], 'image/png')
        self.assertTrue(all(v['byteOffset'] % 4 == 0 for v in doc['bufferViews']))

    def test_archive_priority_places_patches_before_base_files(self):
        paths = [Path(n) for n in ['common.MPQ', 'patch-2.MPQ', 'patch.MPQ', 'lichking.MPQ', 'patch-3.MPQ']]
        self.assertEqual([p.name for p in sorted(paths, key=wow.archive_priority, reverse=True)],
                         ['patch-3.MPQ', 'patch-2.MPQ', 'patch.MPQ', 'lichking.MPQ', 'common.MPQ'])

    def test_compressed_quaternion_identity_and_normalization(self):
        self.assertEqual(wow.quaternion((32767, 32767, 32767, -1)), (0., 0., 0., 1.))
        q = wow.quaternion((32767, 32767, -1, 32767))
        self.assertEqual(q, (0., 1., 0., 0.))

    def test_model_converts_triangle_and_uses_replacement_texture(self):
        files = skeletal_files()
        glb = wow.convert_model(files.__getitem__, 'model.M2', {11: 'neutral.blp'}, 1.)
        doc = json.loads(glb[20:20+struct.unpack_from('<I', glb, 12)[0]])
        primitive = doc['meshes'][0]['primitives'][0]
        self.assertEqual(doc['accessors'][primitive['indices']]['count'], 3)
        positions = doc['accessors'][primitive['attributes']['POSITION']]
        self.assertEqual(positions['min'], [-1., 0., -1.])
        self.assertEqual(len(doc['images']), 1)
        with self.assertRaises(ValueError):
            wow.convert_model(files.__getitem__, 'model.M2', {}, 1.)

    def test_idle_and_drive_translations_use_bind_pivot_not_previous_clip_pose(self):
        files = skeletal_files()
        glb = wow.convert_model(files.__getitem__, 'model.M2', {11: 'neutral.blp'}, 1.)
        length = struct.unpack_from('<I', glb, 12)[0]
        doc = json.loads(glb[20:20+length])
        binary = glb[28+length:]
        self.assertEqual(doc['nodes'][1]['translation'], [-2.,3.,-2.])
        for animation, expected in zip(doc['animations'], [(-2.,3.,-2.),(-2.,3.,-3.)]):
            output = doc['accessors'][animation['samplers'][0]['output']]
            offset = doc['bufferViews'][output['bufferView']]['byteOffset']
            self.assertEqual(struct.unpack_from('<3f', binary, offset), expected)

    def test_animation_alias_cycle_is_rejected_instead_of_hanging(self):
        files = skeletal_files()
        data = files['model.M2']
        struct.pack_into('<I', data, 608+12, 0x60)
        struct.pack_into('<h', data, 608+62, 1)
        struct.pack_into('<I', data, 672+12, 0x60)
        struct.pack_into('<h', data, 672+62, 0)
        with self.assertRaisesRegex(ValueError, 'alias'):
            wow.resolve_sequence(wow.array(data,28,'<HHIfIhhIII6ffhh'),0)

    def test_nonfinite_skeleton_and_animation_keys_are_rejected(self):
        for offset in [588, 840, 852]:
            for value in [float('nan'), float('inf')]:
                with self.subTest(offset=offset, value=value):
                    files = skeletal_files()
                    struct.pack_into('<f', files['model.M2'], offset, value)
                    with self.assertRaises(ValueError):
                        wow.convert_model(files.__getitem__, 'model.M2', {11:'neutral.blp'}, 1.)

    def test_missing_or_zero_duration_drive_clip_is_rejected(self):
        for offset, fmt, value in [(672,'<H',16), (676,'<I',0)]:
            with self.subTest(offset=offset):
                files = skeletal_files()
                struct.pack_into(fmt,files['model.M2'],offset,value)
                with self.assertRaises(ValueError):
                    wow.convert_model(files.__getitem__, 'model.M2', {11:'neutral.blp'}, 1.)

    def test_glb_cannot_serialize_nonfinite_json(self):
        builder = wow.Glb()
        builder.document['nodes'][0]['translation'] = [0.,float('nan'),0.]
        with self.assertRaises(ValueError):
            builder.finish()


if __name__ == '__main__':
    unittest.main()
