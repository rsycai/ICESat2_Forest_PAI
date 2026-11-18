# Small real-data fixtures

Run from the project root on Linux or Windows:

```console
cargo run --release --locked --bin ICESat2_Forest_PAI
python workflow.py
python tests/run_smoke.py
```

The computation command defaults to the two HDF5 files and `landcover.tif` under `tests/data`, and writes `tests/output/PAI.h5`. The workflow defaults to batch processing of `tests/data` and writes to `tests/output`. Download/run actions still download into the current working directory unless `--directory` is provided. Inputs and output paths can be overridden with the existing command-line options.

Existing products are not overwritten. `run_smoke.py` uses a temporary output directory so it can be repeated safely. It requires only the Python standard library and the built executable. It checks successful execution and the HDF5 signature; it does not independently verify the scientific algorithm.

The fixtures are cropped from NASA granules `ATL03_20200101235632_00960601_007_01.h5` and `ATL08_20200101235632_00960601_007_01.h5`, beam `gt1l`, retaining three land segments and their associated photons. They are deliberately incomplete granules, intended only for an offline smoke test. Segment IDs, coordinates, heights, and classifications are retained; ATL03 photon offsets are rebased for the crop. Nonessential metadata and reference attributes are omitted.

The landcover fixture is one 256-by-256 tile from the corresponding 2020 MODIS IGBP raster, preserving its EPSG:4326 georeferencing. It is not a global landcover dataset.

The default fixture produces one retained forest segment: ID 58359, PAI 3.528987407684326 and PAIe 1.7149004936218262. All output columns were checked against the full-granule result. `fixtures.json` records provenance, SHA-256 hashes, and expected values.

To regenerate with original source data, install h5py, numpy, and tifffile in a development environment, then run:

```console
python tests/build_fixtures.py --atl03 data/ATL03_20200101235632_00960601_007_01.h5 --atl08 data/ATL08_20200101235632_00960601_007_01.h5 --landcover data/landcover_2020.tif
```

No Earthdata token, network access, or scientific Python package is needed to run the bundled fixtures.
