# Optional xt-laipath Python bindings

ICESat2_Forest_PAI runs its scientific calculations through the Rust library. The download and batch workflow uses Python 3.9+ with the standard library only; installing these optional bindings is not required.

The inherited xt-laipath crate provides a `python` feature for PyO3 bindings using the CPython stable ABI (abi3-py39). Native extensions must be built separately for each operating system and CPU architecture. A Windows extension cannot run on Linux.

The public command-line workflow and portable build instructions are documented in the project README. Historical wheel-building scripts and their platform-specific examples are not part of this standalone project.

The optional bindings share the GPL-3.0-or-later numerical implementation. See COPYING, NOTICE, and GSL-COPYRIGHTS.txt in this crate.
