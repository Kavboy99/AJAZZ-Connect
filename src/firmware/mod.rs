//! Static analysis of vendor firmware packages. Files are parsed as bytes: nothing is executed and
//! nothing is written to a device. A parsed image never proves compatibility with a board.

mod error;
mod image;
mod package;
mod pe;
#[cfg(test)]
mod tests;

pub use error::InspectError;
pub use image::ImageInfo;
pub use package::{PackageInfo, inspect_bytes, inspect_file};
