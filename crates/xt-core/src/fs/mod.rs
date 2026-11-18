
mod paths;
mod removal;

pub use paths::bounded_path;
pub use removal::{remove_empty_parents, remove_file_and_empty_parents};
