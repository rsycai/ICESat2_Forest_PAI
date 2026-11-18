# ICESat2_Forest_PAI

Standalone computation and download workflow. Inputs are paired ATL03/ATL08 HDF5 granules and a MODIS IGBP landcover GeoTIFF. G(0) is fixed at 0.5. The final HDF5 product retains forest classes 1-5 and QA 0-5 by default.

## Build

Requires Rust 1.89+ and access to Cargo dependencies on the first build. Python 3.9+ is needed for the workflow script; no Python packages are required. Native GDAL and HDF5 installations are not required.

```console
cargo build --release --locked
```

The commands below work on Linux and Windows and use paths relative to the current working directory. Run them from the project root. Use `python3` instead of `python` if that is the Python 3 command on your system. Native executables must be built for the target operating system; the Windows ZIP cannot run on Linux.

## Offline quick start

Small real-data ATL03/ATL08 and landcover fixtures are bundled under `tests/data` (about 157 KB total). Default input paths point to these files. Run from the project root:

```console
cargo run --release --locked --bin ICESat2_Forest_PAI
python workflow.py
python tests/run_smoke.py
```

The first command writes `tests/output/PAI.h5`; the second runs batch processing on the fixtures. The smoke test uses temporary output and can be repeated without overwriting products. No download or token is needed. See `tests/README.md` for provenance and expected values. These are small cropped fixtures, not complete NASA granules or a global landcover raster.

## Compute one pair

Use original product filenames for granules from the same observation and release. Replace the placeholder input filenames below with your actual filenames. `cargo run` selects the platform-specific executable automatically.

```console
cargo run --release --locked --bin ICESat2_Forest_PAI -- --atl03 data/ATL03_original.h5 --atl08 data/ATL08_original.h5 --landcover data/landcover.tif --output output/PAI.h5
```

The output path names the single final product. Existing outputs are rejected. Temporary matched data and intermediate products are automatically removed.

## Download and batch processing

Set the `EARTHDATA_TOKEN` environment variable to your own Earthdata token using your operating system or terminal settings before downloading. No credentials are bundled.

```console
python workflow.py download --start 2020-01-01T00:00:00Z --end 2020-01-02T00:00:00Z --bbox "100,20,101,21"
python workflow.py run --start 2020-01-01T00:00:00Z --end 2020-01-02T00:00:00Z --bbox "100,20,101,21" --landcover data/landcover.tif
python workflow.py batch --directory downloads --landcover data/landcover.tif
python workflow.py download --all
```

Downloads default to the current working directory. Omitted spatial bounds mean global coverage. Omitted time bounds cover mission start through command execution time. Time-only and spatial-only filters are supported. `--all` covers all times and locations for the selected release, default 007; use `--release` to override it. Global downloads can be very large.

Spatial search selects intersecting granules without clipping photons. Use landcover with the appropriate coverage and acquisition year.

Batch processing recursively pairs matching acquisition times, orbit identifiers, and releases, selecting the latest local revision per product. Missing pairs are reported. Existing products are skipped. Failures or no available pairs produce a nonzero exit status. Final products are saved under `products/` unless `--output-directory` is specified.

## Configuration

`config/config.yaml` contains logging, download concurrency/retries/resume, matching read windows, and landcover/QA filters. `resource/download/sources.xml` defines data sources, releases, and the credential environment variable; the token is empty.

The workflow loads the bundled YAML regardless of the working directory. Use `--config` for another file. The single-pair executable also accepts `--config`; otherwise it searches `config/config.yaml` in the current working directory. The default landcover path is `tests/data/landcover.tif`. The command-line landcover path overrides configuration. G(0) remains fixed at 0.5. Download location is controlled by `--directory`.

## Validation and publication

Release packaging, NASA download, automatic pairing, and final-product generation have been tested on one real ATL03/ATL08 pair (2020-01-01). The product contains 269 rows across six beams and matches the local-input run dataset for dataset. Broader scientific validation remains pending. The project includes GPL-3.0-or-later code from xt-laipath; finalize distribution licensing, attribution, and the paper citation before publication. Do not commit credentials or downloaded datasets.
