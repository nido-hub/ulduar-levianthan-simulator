"""Build-time, read-only WoW 3.3.5a importer. Never used by the game itself."""
import argparse
import ctypes as ct
import hashlib
import io
import json
import math
from pathlib import Path
import re
import struct

from PIL import Image, ImageDraw


def records(data, offset, count, fmt):
    size = struct.calcsize(fmt)
    if offset < 0 or count < 0 or offset > len(data) or count > (len(data) - offset) // size:
        raise ValueError(f'Binary range outside file: {offset} + {count} * {size} / {len(data)}')
    return list(struct.iter_unpack(fmt, data[offset:offset + count * size]))


def array(data, header_offset, fmt):
    count, offset = records(data, header_offset, 1, '<II')[0]
    return records(data, offset, count, fmt)


def coordinates(v):
    # Proper rotation (determinant +1): WoW X-forward/Z-up -> glTF -Z-forward/Y-up.
    return (-v[1], v[2], -v[0])


def skin_indices(data, vertex_count):
    if data[:4] != b'SKIN':
        raise ValueError('Expected WotLK SKIN')
    vertices = [r[0] for r in array(data, 4, '<H')]
    indices = [r[0] for r in array(data, 12, '<H')]
    if len(indices) % 3 or any(v >= vertex_count for v in vertices) or any(i >= len(vertices) for i in indices):
        raise ValueError('Invalid SKIN vertex/triangle index')
    return [vertices[i] for i in indices]


def archive_priority(path):
    name = path.stem.lower()
    if name.startswith('patch'):
        suffix = name.removeprefix('patch').lstrip('-')
        return (2, int(suffix) if suffix.isdigit() else (100 + ord(suffix[0]) if suffix else 0), name)
    return (1, {'common': 0, 'common-2': 1, 'expansion': 2, 'lichking': 3}.get(name, 4), name)


class Mpq:
    def __init__(self, client, dll):
        self.lib = ct.WinDLL(str(dll), use_last_error=True)
        signatures = {
            'SFileOpenArchive': [ct.c_wchar_p, ct.c_uint32, ct.c_uint32, ct.POINTER(ct.c_void_p)],
            'SFileOpenFileEx': [ct.c_void_p, ct.c_char_p, ct.c_uint32, ct.POINTER(ct.c_void_p)],
            'SFileGetFileSize': [ct.c_void_p, ct.POINTER(ct.c_uint32)],
            'SFileReadFile': [ct.c_void_p, ct.c_void_p, ct.c_uint32, ct.POINTER(ct.c_uint32), ct.c_void_p],
            'SFileCloseFile': [ct.c_void_p],
            'SFileCloseArchive': [ct.c_void_p],
        }
        for name, args in signatures.items():
            getattr(self.lib, name).argtypes = args
            getattr(self.lib, name).restype = ct.c_bool
        self.lib.SFileGetFileSize.restype = ct.c_uint32
        self.handles = []
        self.sources = {}
        paths = sorted((client / 'Data').glob('*.MPQ'), key=archive_priority, reverse=True)
        paths += sorted((client / 'Data' / 'enUS').glob('*.MPQ'), key=archive_priority, reverse=True)
        if not paths:
            raise FileNotFoundError(f'No MPQ archives under {client / "Data"}')
        try:
            for path in paths:
                handle = ct.c_void_p()
                if not self.lib.SFileOpenArchive(str(path.resolve()), 0, 0x100, ct.byref(handle)):
                    raise OSError(ct.get_last_error(), f'Cannot open {path} read-only')
                self.handles.append((path, handle))
        except Exception:
            self.close()
            raise

    def read(self, name):
        for path, archive in self.handles:
            handle = ct.c_void_p()
            if not self.lib.SFileOpenFileEx(archive, name.encode('ascii'), 0, ct.byref(handle)):
                continue
            try:
                high = ct.c_uint32()
                size = self.lib.SFileGetFileSize(handle, ct.byref(high))
                if high.value or size > 64 * 1024 * 1024:
                    raise ValueError(f'Unexpectedly large asset: {name}')
                buffer = ct.create_string_buffer(size)
                count = ct.c_uint32()
                if not self.lib.SFileReadFile(handle, buffer, size, ct.byref(count), None) or count.value != size:
                    raise OSError(ct.get_last_error(), f'Cannot read {name} from {path.name}')
                data = buffer.raw
                self.sources[name] = {'archive': path.name, 'sha256': hashlib.sha256(data).hexdigest()}
                return data
            finally:
                self.lib.SFileCloseFile(handle)
        raise FileNotFoundError(name)

    def close(self):
        for _, handle in self.handles:
            self.lib.SFileCloseArchive(handle)
        self.handles.clear()


class Glb:
    def __init__(self):
        self.binary = bytearray()
        self.document = {
            'asset': {'version': '2.0', 'generator': 'LeviathanRust local WotLK converter'},
            'scene': 0, 'scenes': [{'nodes': [0]}], 'nodes': [{'name': 'WoW vehicle', 'children': []}],
            'meshes': [], 'materials': [], 'textures': [], 'images': [],
            'samplers': [{'magFilter': 9729, 'minFilter': 9729, 'wrapS': 10497, 'wrapT': 10497}],
            'buffers': [], 'bufferViews': [], 'accessors': [],
        }

    def view(self, data, target=None):
        self.binary.extend(b'\x00' * (-len(self.binary) % 4))
        value = {'buffer': 0, 'byteOffset': len(self.binary), 'byteLength': len(data)}
        if target:
            value['target'] = target
        index = len(self.document['bufferViews'])
        self.document['bufferViews'].append(value)
        self.binary.extend(data)
        return index

    def accessor(self, values, fmt, component, kind, bounds=False):
        if not all(math.isfinite(value) for row in values for value in row):
            raise ValueError('Nonfinite GLB vertex, skeleton or animation data')
        view = self.view(b''.join(struct.pack(fmt, *v) for v in values), 34963 if kind == 'SCALAR' and component != 5126 else 34962)
        value = {'bufferView': view, 'componentType': component, 'count': len(values), 'type': kind}
        if bounds:
            value['min'] = [min(v[i] for v in values) for i in range(len(values[0]))]
            value['max'] = [max(v[i] for v in values) for i in range(len(values[0]))]
        index = len(self.document['accessors'])
        self.document['accessors'].append(value)
        return index

    def texture(self, png):
        index = len(self.document['images'])
        self.document['images'].append({'bufferView': self.view(png), 'mimeType': 'image/png'})
        self.document['textures'].append({'source': index, 'sampler': 0})
        return index

    def finish(self):
        self.document['buffers'] = [{'byteLength': len(self.binary)}]
        text = json.dumps(self.document, separators=(',', ':'), allow_nan=False).encode()
        text += b' ' * (-len(text) % 4)
        self.binary.extend(b'\x00' * (-len(self.binary) % 4))
        length = 28 + len(text) + len(self.binary)
        return (struct.pack('<4sII I4s', b'glTF', 2, length, len(text), b'JSON') + text
                + struct.pack('<I4s', len(self.binary), b'BIN\x00') + self.binary)


def png_bytes(data):
    with Image.open(io.BytesIO(data)) as image:
        output = io.BytesIO()
        image.convert('RGBA').save(output, format='PNG')
        return output.getvalue()


def quaternion(shorts):
    q = [(v - 32767) / 32767 if v >= 0 else (v + 32768) / 32767 for v in shorts]
    length = math.sqrt(sum(v * v for v in q))
    if length < 1e-6:
        raise ValueError('Invalid M2 quaternion')
    xyz = coordinates(q[:3])
    return tuple(v / length for v in (*xyz, q[3]))


def track(data, offset, sequence, fmt):
    interpolation, global_sequence = records(data, offset, 1, '<Hh')[0]
    timestamps = array(data, offset + 4, '<II')
    values = array(data, offset + 12, '<II')
    sequence = 0 if global_sequence >= 0 else sequence
    if sequence >= len(timestamps) or sequence >= len(values):
        return [], []
    nt, ot = timestamps[sequence]
    nv, ov = values[sequence]
    if nt != nv or interpolation > 1:
        raise ValueError('Unsupported or inconsistent M2 animation track')
    times = [t[0] / 1000. for t in records(data, ot, nt, '<I')]
    keys = records(data, ov, nv, fmt)
    if any(a >= b for a, b in zip(times, times[1:])):
        raise ValueError('Non-increasing M2 animation times')
    return times, keys


def resolve_sequence(sequences, index):
    visited = set()
    while True:
        if index < 0 or index >= len(sequences) or index in visited:
            raise ValueError('Invalid/cyclic M2 animation alias')
        visited.add(index)
        if not sequences[index][4] & 0x40:
            return index
        index = sequences[index][-1]


def convert_model(read, name, replacements, scale, overrides=None):
    data = read(name)
    if data[:4] != b'MD20' or records(data, 4, 1, '<I')[0][0] != 264:
        raise ValueError(f'Expected WotLK M2 version 264: {name}')
    vertices = array(data, 60, '<3f4B4B3f4f')
    if not vertices or not all(math.isfinite(x) for v in vertices for x in (*v[:3], *v[11:])):
        raise ValueError('Empty or nonfinite M2 vertices')
    skin = read(name[:-3] + '00.skin')
    indices = skin_indices(skin, len(vertices))
    sections = array(skin, 28, '<10H7f')
    batches = array(skin, 36, '<Bb11H')
    textures = array(data, 80, '<4I')
    lookup = [v[0] for v in array(data, 128, '<H')]
    materials = array(data, 112, '<HH')
    bones = array(data, 44, '<iIhHHH20s20s20s3f')
    sequences = array(data, 28, '<HHIfIhhIII6ffhh')
    glb = Glb()
    doc = glb.document
    doc['nodes'][0].update(name=Path(name.replace('\\', '/')).stem, scale=[scale]*3)
    texture_cache = {}

    def texture_path(index):
        if index >= len(textures):
            raise ValueError('Texture index outside M2')
        kind, flags, length, offset = textures[index]
        if kind:
            if kind not in replacements:
                raise ValueError(f'Missing replacement texture type {kind} for {name}')
            return replacements[kind]
        value = bytes(v[0] for v in records(data, offset, length, '<B')).rstrip(b'\x00').decode('ascii')
        return (overrides or {}).get(value.upper(), value)

    def texture(path):
        key = path.lower()
        if key not in texture_cache:
            texture_cache[key] = glb.texture(png_bytes(read(path)))
        return texture_cache[key]

    attributes = {
        'POSITION': glb.accessor([coordinates(v[:3]) for v in vertices], '<3f', 5126, 'VEC3', True),
        'NORMAL': glb.accessor([coordinates(v[11:14]) for v in vertices], '<3f', 5126, 'VEC3'),
        'TEXCOORD_0': glb.accessor([v[14:16] for v in vertices], '<2f', 5126, 'VEC2'),
    }
    if bones:
        if any(v[3+i] and v[7+i] >= len(bones) for v in vertices for i in range(4)):
            raise ValueError('Vertex references nonexistent bone')
        attributes['JOINTS_0'] = glb.accessor([v[7:11] for v in vertices], '<4H', 5123, 'VEC4')
        weights = []
        for v in vertices:
            total = sum(v[3:7])
            weights.append(tuple(w / total for w in v[3:7]) if total else (1., 0., 0., 0.))
        attributes['WEIGHTS_0'] = glb.accessor(weights, '<4f', 5126, 'VEC4')
        pivots = [coordinates(b[-3:]) for b in bones]
        inverse_bind = []
        local_pivots = []
        for i, bone in enumerate(bones):
            parent = bone[2]
            if parent >= i or parent < -1:
                raise ValueError('Invalid/cyclic M2 bone hierarchy')
            pivot = pivots[i]
            local = tuple(pivot[k] - (pivots[parent][k] if parent >= 0 else 0) for k in range(3))
            local_pivots.append(local)
            node = {'name': f'Bone_{i}', 'translation': local, 'children': []}
            doc['nodes'].append(node)
            doc['nodes'][parent+1 if parent >= 0 else 0]['children'].append(i+1)
            inverse_bind.append((1.,0.,0.,0., 0.,1.,0.,0., 0.,0.,1.,0., -pivot[0],-pivot[1],-pivot[2],1.))
        doc['skins'] = [{'joints': list(range(1, len(bones)+1)),
                         'inverseBindMatrices': glb.accessor(inverse_bind, '<16f', 5126, 'MAT4')}]

        # Only the two locomotion clips used by the trainer, not combat choreography.
        doc['animations'] = []
        for animation_id, label in [(0, 'Idle'), (4, 'Drive')]:
            found = next((i for i, a in enumerate(sequences) if a[0] == animation_id and a[1] == 0), None)
            if found is None:
                continue
            found = resolve_sequence(sequences, found)
            if sequences[found][2] == 0:
                raise ValueError(f'Zero-duration M2 animation: {name} / {label}')
            if not sequences[found][4] & 0x20:
                raise ValueError(f'External animation not supported: {name} / {label}')
            clip = {'name': label, 'channels': [], 'samplers': []}
            for i, bone in enumerate(bones):
                pivot = local_pivots[i]
                for field, fmt, path in [(16, '<3f', 'translation'), (36, '<4h', 'rotation'), (56, '<3f', 'scale')]:
                    times, keys = track(data, records(data,44,1,'<II')[0][1] + i*88 + field, found, fmt)
                    if not keys:
                        continue
                    if path == 'translation':
                        values = [tuple(pivot[k] + coordinates(v)[k] for k in range(3)) for v in keys]
                    elif path == 'rotation':
                        values = [quaternion(v) for v in keys]
                        for k in range(1, len(values)):
                            if sum(a*b for a,b in zip(values[k-1], values[k])) < 0:
                                values[k] = tuple(-v for v in values[k])
                    else:
                        values = [(v[1], v[2], v[0]) for v in keys]
                    if label == 'Idle':
                        doc['nodes'][i+1][path] = values[0]
                    # glTF requires strictly increasing timestamps, even for constant tracks.
                    if len(times) == 1:
                        times = [0., sequences[found][2] / 1000.]
                        values = [values[0], values[0]]
                    sampler = len(clip['samplers'])
                    clip['samplers'].append({'input': glb.accessor([(t,) for t in times], '<f', 5126, 'SCALAR', True),
                                             'output': glb.accessor(values, '<4f' if path == 'rotation' else '<3f',
                                                                    5126, 'VEC4' if path == 'rotation' else 'VEC3'),
                                             'interpolation': 'LINEAR'})
                    clip['channels'].append({'sampler': sampler, 'target': {'node': i+1, 'path': path}})
            if clip['channels']:
                doc['animations'].append(clip)
    primitives = []
    for section_index, section in enumerate(sections):
        primary = next((b for b in batches if b[3] == section_index and b[7] == 0 and not b[0] & 8), None)
        if primary is None:
            continue  # Animated smoke cards use WoW-specific projected texture shaders.
        _, _, _, _, _, _, material_index, _, count, combo, _, _, _ = primary
        if material_index >= len(materials) or combo >= len(lookup):
            raise ValueError('Invalid M2 material lookup')
        flags, blend = materials[material_index]
        base = texture_path(lookup[combo])
        material = {'name': base, 'pbrMetallicRoughness': {
            'baseColorTexture': {'index': texture(base)}, 'metallicFactor': 0., 'roughnessFactor': 0.88},
            'doubleSided': bool(flags & 4)}
        if blend == 1:
            material.update(alphaMode='MASK', alphaCutoff=0.5)
        elif blend > 1:
            material['alphaMode'] = 'BLEND'
        for b in batches:
            if b[3] != section_index or b[0] & 8:
                continue
            for texture_combo in range(b[9], b[9] + b[8]):
                if texture_combo >= len(lookup):
                    raise ValueError('Invalid texture combination')
                path = texture_path(lookup[texture_combo])
                if 'glow' in path.lower():
                    material.update(emissiveTexture={'index': texture(path)}, emissiveFactor=[0.65]*3)
        mat = len(doc['materials'])
        doc['materials'].append(material)
        start = section[4] + (section[1] << 16)
        count = section[5]
        if start + count > len(indices) or count % 3:
            raise ValueError('Invalid submesh triangle range')
        primitives.append({'attributes': attributes, 'material': mat,
                           'indices': glb.accessor([(i,) for i in indices[start:start+count]], '<I', 5125, 'SCALAR')})
    if not primitives:
        raise ValueError('No renderable mesh sections')
    if not bones or [a['name'] for a in doc.get('animations', [])] != ['Idle', 'Drive']:
        raise ValueError(f'Model must contain a skeleton and usable Idle/Drive animations: {name}')
    doc['meshes'].append({'name': name, 'primitives': primitives})
    node = {'name': 'VehicleMesh', 'mesh': 0}
    if bones:
        node['skin'] = 0
    doc['nodes'][0]['children'].append(len(doc['nodes']))
    doc['nodes'].append(node)
    return glb.finish()


def dbc_rows(data):
    if data[:4] != b'WDBC':
        raise ValueError('Expected WDBC')
    count, fields, size, string_size = records(data, 4, 1, '<4I')[0]
    if size != fields * 4 or 20 + count * size + string_size != len(data):
        raise ValueError('Invalid DBC size')
    return records(data, 20, count, '<' + 'I' * fields), data[20 + count * size:]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--client', required=True, type=Path)
    parser.add_argument('--stormlib', required=True, type=Path, help='64-bit Unicode StormLib DLL (build tool only)')
    args = parser.parse_args()
    output = Path(__file__).resolve().parents[1] / 'assets' / 'wow'
    ui = output / 'ui'
    ui.mkdir(parents=True, exist_ok=True)
    mpq = Mpq(args.client, args.stormlib)
    outputs = {}

    def save(name, data):
        target = output / name
        # Build in memory first; atomic replacement prevents half-written individual assets.
        temp = target.with_suffix(target.suffix + '.tmp')
        temp.write_bytes(data)
        temp.replace(target)
        outputs[name] = hashlib.sha256(data).hexdigest()

    try:
        models = [
            ('demolisher', r'DemolisherVehicle\DemolisherVehicle',
             {11: r'Creature\DemolisherVehicle\DemolisherVehicle01_Neutral.blp',
              12: r'Creature\DemolisherVehicle\DemolisherVehicle02_Neutral.blp'}, 0.5, {}),
            ('siege', r'SiegeVehicle\SiegeVehicleHasGun', {}, 0.5,
             {r'CREATURE\SIEGEVEHICLE\SIEGEVEHICLEALLIANCE_01.BLP': r'Creature\SiegeVehicle\SiegeVehicle_01_Neutral.blp',
              r'CREATURE\SIEGEVEHICLE\SIEGEVEHICLEALLIANCE_02.BLP': r'Creature\SiegeVehicle\SiegeVehicle_02_Neutral.blp'}),
            ('moto', r'MotorcycleVehicle\MotorcycleVehicle',
             {11: r'Creature\MotorcycleVehicle\MotorcycleVehicle_Alliance01.blp',
              12: r'Creature\MotorcycleVehicle\MotorcycleVehicle_Alliance02.blp',
              13: r'Creature\MotorcycleVehicle\MotorcycleVehicle_Alliance02_glow.blp'}, 1., {}),
            ('leviathan', r'FlameLeviathan\FlameLeviathan', {}, 1., {}),
        ]
        for key, stem, replacements, scale, overrides in models:
            glb = convert_model(mpq.read, 'Creature\\' + stem + '.M2', replacements, scale, overrides)
            save(key + '.glb', glb)
            print(f'{key}.glb: {len(glb):,} bytes, embedded textures and Idle/Drive clips')
        images = {
            'vehicle-frame': r'Interface\Vehicles\UI-Vehicle-Frame',
            'vehicle-border': r'Interface\Vehicles\UI-Vehicle-Frame-Border',
            'vehicle-trim': r'Interface\Vehicles\UI-Vehicles-Trim-Alliance',
            'vehicle-endcap': r'Interface\Vehicles\UI-Vehicles-Endcap',
            'quickslot': r'Interface\Buttons\UI-Quickslot2',
            'dialog': r'Interface\DialogFrame\UI-DialogBox-Background',
            'statusbar': r'Interface\TargetingFrame\UI-StatusBar',
            'target-frame': r'Interface\TargetingFrame\UI-TargetingFrame-Elite',
            'seat-demo': r'Interface\Vehicles\SeatIndicator\Vehicle-Demolisher',
            'seat-siege': r'Interface\Vehicles\SeatIndicator\Vehicle-SiegeEngine',
            'seat-moto': r'Interface\Vehicles\SeatIndicator\vehicle-motorcycle',
            'boss': r'Interface\Icons\Achievement_Boss_TheFlameLeviathan_01',
        }
        for key, path in images.items():
            save(f'ui/{key}.png', png_bytes(mpq.read(path + '.blp')))
        for source, key, box in [('quickslot', 'slot-frame', (12,12,51,51)),
                                 ('vehicle-endcap', 'dock', (0,204,256,254)),
                                 ('vehicle-trim', 'trim', (0,24,128,40))]:
            with Image.open(ui / f'{source}.png') as image:
                buffer = io.BytesIO()
                image.crop(box).save(buffer, format='PNG')
                save(f'ui/{key}.png', buffer.getvalue())
        with Image.open(ui / 'boss.png') as image:
            image = image.convert('RGBA')
            mask = Image.new('L', image.size)
            ImageDraw.Draw(mask).ellipse((0,0,image.width-1,image.height-1), fill=255)
            image.putalpha(mask)
            buffer = io.BytesIO()
            image.save(buffer, format='PNG')
            save('ui/boss.png', buffer.getvalue())
        save('ui/frizqt.ttf', mpq.read(r'Fonts\FRIZQT__.TTF'))
        icons, strings = dbc_rows(mpq.read(r'DBFilesClient\SpellIcon.dbc'))
        wanted = {3760, 3779, 3762, 3768, 3761, 2589, 2735, 3766, 3772,
                  3763, 2102, 3774, 3845, 3769, 3770, 3773, 104, 3771}
        for row in icons:
            if row[0] in wanted:
                start = row[1]
                end = strings.find(b'\x00', start)
                if start >= len(strings) or end < 0:
                    raise ValueError('Invalid SpellIcon string offset')
                path = strings[start:end].decode('ascii') + '.blp'
                save(f'ui/icon-{row[0]}.png', png_bytes(mpq.read(path)))
                wanted.remove(row[0])
        if wanted:
            raise ValueError(f'Missing icon IDs: {wanted}')
        manifest = {
            'origin': 'Local World of Warcraft 3.3.5a client. Blizzard assets; not freely redistributable.',
            'runtime': 'Standalone GLB/PNG/TTF copies; no client, archive, network or StormLib access at runtime.',
            'limits': 'Diffuse/emissive materials and Idle/Drive skeletal animation; no WoW reflection shaders, particle emitters or combat attachment choreography.',
            'sources': mpq.sources, 'outputs': outputs,
        }
        save('manifest.json', json.dumps(manifest, indent=2, ensure_ascii=False).encode('utf8'))
        print(f'Copied {len(outputs)} standalone resources into {output}')
    finally:
        mpq.close()


if __name__ == '__main__':
    main()
