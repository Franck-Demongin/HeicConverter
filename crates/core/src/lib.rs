pub mod conversion;
pub mod error;

pub use conversion::{ConversionOptions, ConversionResult, OutputFormat, convert_file};
pub use error::Error;

/// Version visible sans interface pour verif de compilation.
pub fn library_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
