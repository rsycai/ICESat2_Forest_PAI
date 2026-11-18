"""ICESat2_Forest_PAI download and batch workflow. Python 3.9+, standard library only."""
import argparse
import datetime as dt
import pathlib
import subprocess
import sys


def pairs(root):
    groups = {}
    for path in sorted(root.rglob('ATL*.h5')):
        parts = path.stem.split('_')
        if len(parts) != 5 or parts[0] not in ('ATL03', 'ATL08'):
            continue
        key = tuple(parts[1:4])
        groups.setdefault(key, {}).setdefault(parts[0], []).append(path)
    for key, products in sorted(groups.items()):
        if 'ATL03' not in products or 'ATL08' not in products:
            print('Unpaired granule:', '_'.join(key), file=sys.stderr)
            continue
        # Latest revision per product; release/time/orbit must match.
        yield key, max(products['ATL03'], key=lambda p: p.stem), max(products['ATL08'], key=lambda p: p.stem)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', nargs='?', default='batch', choices=['download', 'batch', 'run'])
    parser.add_argument('--config', type=pathlib.Path, default=pathlib.Path(__file__).resolve().parent / 'config' / 'config.yaml')
    parser.add_argument('--directory', type=pathlib.Path, default=None)
    parser.add_argument('--start', default='2018-10-13T00:00:00Z')
    parser.add_argument('--end', default=dt.datetime.now(dt.timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ'))
    parser.add_argument('--bbox', help='west,south,east,north; omitted means global')
    parser.add_argument('--all', action='store_true', help='entire supported time range and globe')
    parser.add_argument('--release', default='007')
    parser.add_argument('--landcover', type=pathlib.Path, default=pathlib.Path('tests/data/landcover.tif'))
    parser.add_argument('--output-directory', type=pathlib.Path)
    parser.add_argument('--bin-directory', type=pathlib.Path, default=pathlib.Path(__file__).resolve().parent / 'target' / 'release')
    args = parser.parse_args()
    using_test_inputs = args.action == 'batch' and args.directory is None
    if args.directory is None:
        args.directory = pathlib.Path('tests/data') if using_test_inputs else pathlib.Path.cwd()
    if args.all and (args.bbox or '--start' in sys.argv or '--end' in sys.argv):
        parser.error('--all cannot be combined with time or spatial filters')
    if args.action != 'download' and (not args.landcover or not args.landcover.is_file()):
        parser.error('batch/run requires --landcover pointing to a GeoTIFF')
    suffix = '.exe' if sys.platform == 'win32' else ''
    def binary(name):
        path = args.bin_directory / (name + suffix)
        if not path.is_file():
            parser.error('Build first: cargo build --release --locked')
        return str(path.resolve())
    calculator = binary('ICESat2_Forest_PAI') if args.action != 'download' else None
    if args.action in ('download', 'run'):
        downloader = binary('ICESat2_Forest_PAI-download')
        for product in ('ATL03', 'ATL08'):
            command = [downloader, 'fetch', 'icesat2', product, args.release, args.start, args.end, '--output', str(args.directory.resolve())]
            command += ['--config', str(args.config.resolve())]
            command += ['--bbox', args.bbox] if args.bbox else ['--global']
            subprocess.run(command, check=True)
    if args.action != 'download':
        output = args.output_directory or (pathlib.Path('tests/output') if using_test_inputs else args.directory / 'products')
        failures = 0
        count = 0
        for key, atl03, atl08 in pairs(args.directory):
            count += 1
            dest = output / ('PAI_' + '_'.join(key) + '.h5')
            if dest.exists():
                print('Skip existing:', dest)
                continue
            result = subprocess.run([calculator, '--config', str(args.config.resolve()), '--atl03', str(atl03), '--atl08', str(atl08), '--landcover', str(args.landcover.resolve()), '--output', str(dest)])
            failures += result.returncode != 0
        print(f'Pairs: {count}; failures: {failures}')
        if failures or count == 0:
            return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())


