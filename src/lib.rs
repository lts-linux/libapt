#![doc = include_str!("../README.md")]


mod distro;
mod error;
mod package;
mod package_index;
mod package_version;
mod release;
mod signature;
mod types;
mod util;
mod version;

/// Debian distribution specification
pub use distro::Distro;

pub use distro::Key;
pub use error::{Error, ErrorType, Result};
pub use package::Package;
pub use package_index::PackageIndex;
pub use package_version::{PackageVersion, VersionRelation};
pub use release::{FileHash, Release};
pub use types::architecture::Architecture;
pub use types::priority::Priority;
pub use version::Version;
