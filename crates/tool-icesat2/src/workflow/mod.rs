mod cleanup;
pub mod manifest;
mod pair;
mod processor;

pub use pair::{process_pair, process_pair_with_context};
pub use processor::Icesat2Processor;
