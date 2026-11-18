"""Run the bundled fixtures without downloading data or overwriting existing products."""
import argparse
from pathlib import Path
import subprocess
import sys
import tempfile


def main():
    root = Path(__file__).resolve().parents[1]
    suffix = '.exe' if sys.platform == 'win32' else ''
    default = root / 'target' / 'release' / ('ICESat2_Forest_PAI' + suffix)
    if not default.is_file():
        default = root / ('ICESat2_Forest_PAI' + suffix)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=default)
    args = parser.parse_args()
    if not args.binary.is_file():
        parser.error('Build first with cargo build --release --locked')
    with tempfile.TemporaryDirectory() as directory:
        output = Path(directory) / 'PAI.h5'
        subprocess.run([str(args.binary.resolve()), '--output', str(output)], cwd=root, check=True, timeout=60)
        with output.open('rb') as stream:
            if stream.read(8) != b'\x89HDF\r\n\x1a\n':
                raise RuntimeError('Output is not an HDF5 file')
        print('Smoke test passed: bundled inputs produced a final HDF5 product.')


if __name__ == '__main__':
    main()
