use crate::{PackageVersion, Priority, Version};

pub struct Package {
    // fields from apt package index
    // see https://wiki.debian.org/DebianRepository/Format#A.22Packages.22_Indices
    // and see https://www.debian.org/doc/debian-policy/ch-controlfields.html#debian-binary-package-control-files-debian-control
    package: String,
    source: Option<String>,
    // list of sections is unstable, not using type.
    section: Option<String>,
    priority: Option<Priority>,
    architecture: Option<String>,
    essential: Option<bool>,
    // see https://www.debian.org/doc/debian-policy/ch-relationships.html
    depends: Vec<PackageVersion>,
    pre_depends: Vec<PackageVersion>,
    recommends: Vec<PackageVersion>,
    suggests: Vec<PackageVersion>,
    breaks: Vec<PackageVersion>,
    conflicts: Vec<PackageVersion>,
    provides: Vec<PackageVersion>,
    replaces: Vec<PackageVersion>,
    enhances: Vec<PackageVersion>,
    version: Version,
    size: u32,
    installed_size: u32,
    filename: String,
    md5sum: Option<String>,
    sha1: Option<String>,
    sha256: Option<String>,
    sha512: Option<String>,
    maintainer: String,
    description: String,
    description_md5: Option<String>,
    homepage: Option<String>,
    built_using: Option<Vec<PackageVersion>>,
}
