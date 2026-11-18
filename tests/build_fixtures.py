"""Extract small real-data fixtures. Development dependencies: h5py, numpy, tifffile."""
import argparse
import hashlib
import json
from pathlib import Path
import h5py
import numpy as np
import tifffile


def copy_numeric(source, target, indices, length):
    for name, item in source.items():
        if isinstance(item, h5py.Group):
            copy_numeric(item, target.create_group(name), indices, length)
        elif item.dtype.kind in 'iuf':
            values = item[indices] if item.ndim and item.shape[0] == length else item[()]
            ds = target.create_dataset(name, data=values)
            if '_FillValue' in item.attrs:
                ds.attrs['_FillValue'] = item.attrs['_FillValue']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--atl03', type=Path, required=True)
    parser.add_argument('--atl08', type=Path, required=True)
    parser.add_argument('--landcover', type=Path, required=True)
    parser.add_argument('--segment', type=int, default=58359)
    args = parser.parse_args()
    out = Path(__file__).resolve().parent / 'data'
    out.mkdir(exist_ok=True)
    beam = 'gt1l'
    with h5py.File(args.atl03) as a, h5py.File(args.atl08) as b:
        land = b[beam + '/land_segments']
        first = int(np.flatnonzero(land['segment_id_beg'][:] == args.segment)[0])
        chosen = slice(first, first + 3)
        low = int(land['segment_id_beg'][first])
        high = int(land['segment_id_end'][first + 2])
        geo = a[beam + '/geolocation']
        geo_indices = np.flatnonzero((geo['segment_id'][:] >= low) & (geo['segment_id'][:] <= high))
        starts = geo['ph_index_beg'][geo_indices]
        counts = geo['segment_ph_cnt'][geo_indices]
        photon_start = int(np.min(starts[counts > 0])) - 1
        photon_end = int(np.max(starts[counts > 0] + counts[counts > 0] - 1))
        photon_slice = slice(photon_start, photon_end)
        signal = b[beam + '/signal_photons']
        signal_ids = signal['ph_segment_id'][:]
        signal_indices = np.flatnonzero((signal_ids >= low) & (signal_ids <= high))
        for source, filename, is_atl03 in [(a, args.atl03.name, True), (b, args.atl08.name, False)]:
            with h5py.File(out / filename, 'w', libver='earliest') as target:
                target.attrs['fixture_note'] = 'Cropped real granule; not a complete NASA product.'
                orbit = target.create_group('orbit_info')
                for name in ['rgt', 'cycle_number', 'orbit_number']:
                    orbit.create_dataset(name, data=source['orbit_info/' + name][:])
                dst = target.create_group(beam)
                for name in ['atlas_beam_type', 'atlas_spot_number']:
                    if name in source[beam].attrs:
                        dst.attrs[name] = source[beam].attrs[name]
                if is_atl03:
                    g = dst.create_group('geolocation')
                    for name in ['segment_id', 'ph_index_beg', 'segment_ph_cnt', 'segment_dist_x']:
                        values = geo[name][geo_indices]
                        if name == 'ph_index_beg':
                            values = np.where(values > 0, values - photon_start, values)
                        g.create_dataset(name, data=values)
                    heights = a[beam + '/heights']
                    copy_numeric(heights, dst.create_group('heights'), photon_slice, len(heights['lat_ph']))
                else:
                    copy_numeric(land, dst.create_group('land_segments'), chosen, len(land['segment_id_beg']))
                    copy_numeric(signal, dst.create_group('signal_photons'), signal_indices, len(signal_ids))
        lon, lat = float(land['longitude'][first]), float(land['latitude'][first])
    with tifffile.TiffFile(args.landcover) as raster:
        page = raster.pages[0]
        sx, sy, _ = page.tags['ModelPixelScaleTag'].value
        _, _, _, x0, y0, _ = page.tags['ModelTiepointTag'].value
        col, row = int((lon - x0) / sx), int((y0 - lat) / sy)
        tile_col, tile_row = col // page.tilewidth, row // page.tilelength
        index = tile_row * ((page.imagewidth + page.tilewidth - 1) // page.tilewidth) + tile_col
        raster.filehandle.seek(page.dataoffsets[index])
        tile = page.decode(raster.filehandle.read(page.databytecounts[index]), index)[0].squeeze()
        keys = (1, 1, 0, 3, 1024, 0, 1, 2, 1025, 0, 1, 1, 2048, 0, 1, 4326)
        tifffile.imwrite(out / 'landcover.tif', tile.astype('uint8'), photometric='minisblack', metadata=None,
            extratags=[(33550, 'd', 3, (sx, sy, 0.), False),
                       (33922, 'd', 6, (0., 0., 0., x0 + tile_col * page.tilewidth * sx, y0 - tile_row * page.tilelength * sy, 0.), False),
                       (34735, 'H', len(keys), keys, False)])
    report = {'source_granules': [args.atl03.name, args.atl08.name], 'beam': beam,
              'land_segments': 3, 'segment_id_range': [low, high],
              'atl03_photons': photon_end - photon_start, 'atl08_classified_photons': len(signal_indices),
              'landcover_source': args.landcover.name,
              'files': {p.name: {'bytes': p.stat().st_size, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()} for p in out.iterdir() if p.is_file()}}
    (out.parent / 'fixtures.json').write_text(json.dumps(report, indent=2))
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
