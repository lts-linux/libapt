#![doc = include_str!("../README.md")]

mod distro;
mod error;
mod link;
mod package;
mod package_index;
mod package_version;
mod release;
mod signature;
mod source;
mod source_index;
mod types;
mod util;
mod version;

pub use distro::Distro;
pub use distro::Key;
pub use error::{Error, ErrorType, Result};
pub use link::Link;
pub use link::LinkHash;
pub use package::Package;
pub use package_index::PackageIndex;
pub use package_version::{PackageVersion, VersionRelation};
pub use release::Release;
pub use source::Source;
pub use source_index::SourceIndex;
pub use types::architecture::Architecture;
pub use types::priority::Priority;
pub use util::get_etag;
pub use version::Version;
