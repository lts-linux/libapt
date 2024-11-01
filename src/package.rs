//! Implementation of the package types and parsing.
//!
#[cfg(not(test))]
use log::error;

#[cfg(test)]
use std::println as error;

use std::cmp::Ordering;

use crate::util::{parse_package_relation, parse_stanza};
use crate::{Architecture, Distro, Error, ErrorType, PackageVersion, Priority, Result, Version};

/// The Package struct groups all data about a package.
///
/// When the package index file is parsed, all specified values from
/// [Debian Wiki Package Indices specification](https://wiki.debian.org/DebianRepository/Format#A.22Packages.22_Indices)
/// are considered.
/// For parsing the single entries the
/// [Debian Wiki Binary Package specification](https://www.debian.org/doc/debian-policy/ch-controlfields.html#debian-binary-package-control-files-debian-control)
/// is used as a base.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Package {
    // fields from apt package index
    pub package: String,
    pub source: Option<String>,
    // list of sections is unstable, not using type.
    pub section: Option<String>,
    pub priority: Option<Priority>,
    pub architecture: Option<Architecture>,
    pub essential: Option<bool>,
    // see https://www.debian.org/doc/debian-policy/ch-relationships.html
    pub depends: Vec<PackageVersion>,
    pub pre_depends: Vec<PackageVersion>,
    pub recommends: Vec<PackageVersion>,
    pub suggests: Vec<PackageVersion>,
    pub breaks: Vec<PackageVersion>,
    pub conflicts: Vec<PackageVersion>,
    pub provides: Vec<PackageVersion>,
    pub replaces: Vec<PackageVersion>,
    pub enhances: Vec<PackageVersion>,
    pub version: Version,
    pub size: u32,
    pub installed_size: Option<u32>,
    pub filename: String,
    pub md5sum: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub sha512: Option<String>,
    pub maintainer: String,
    pub description: String,
    pub description_md5: Option<String>,
    pub homepage: Option<String>,
    pub built_using: Option<Vec<PackageVersion>>,
}

impl Package {
    /// New struct with default values.
    pub fn new(
        package: &str,
        version: Version,
        size: u32,
        filename: &str,
        maintainer: &str,
        description: &str,
    ) -> Package {
        Package {
            package: package.to_string(),
            source: None,
            section: None,
            priority: None,
            architecture: None,
            essential: None,
            depends: Vec::new(),
            pre_depends: Vec::new(),
            recommends: Vec::new(),
            suggests: Vec::new(),
            breaks: Vec::new(),
            conflicts: Vec::new(),
            provides: Vec::new(),
            replaces: Vec::new(),
            enhances: Vec::new(),
            version: version,
            size: size,
            installed_size: None,
            filename: filename.to_string(),
            md5sum: None,
            sha1: None,
            sha256: None,
            sha512: None,
            maintainer: maintainer.to_string(),
            description: description.to_string(),
            description_md5: None,
            homepage: None,
            built_using: None,
        }
    }

    /// Parse a Package from its stanza.
    pub fn from_stanza(stanza: &str, distro: &Distro) -> Result<Package> {
        let kv = parse_stanza(stanza);

        let name = match kv.get("package") {
            Some(name) => name,
            None => {
                let message = format!("Invalid stanza, package missing!\n{stanza}");
                error!("{}", &message);
                return Err(Error::new(&message, ErrorType::InvalidPackageMeta));
            }
        };

        let version = match kv.get("version") {
            Some(version) => Version::from_str(version)?,
            None => {
                let message = format!("Invalid stanza, version missing!\n{stanza}");
                error!("{}", &message);
                return Err(Error::new(&message, ErrorType::InvalidPackageMeta));
            }
        };

        let size = match kv.get("size") {
            Some(size) => size.parse::<u32>().map_err(|e| {
                Error::new(
                    &format!("Parsing of size failed! {e}"),
                    ErrorType::InvalidPackageMeta,
                )
            })?,
            None => {
                let message = format!("Invalid stanza, version missing!\n{stanza}");
                error!("{}", &message);
                return Err(Error::new(&message, ErrorType::InvalidPackageMeta));
            }
        };

        let filename = match kv.get("filename") {
            Some(filename) => distro.url(&filename, true),
            None => {
                let message = format!("Invalid stanza, filename missing!\n{stanza}");
                error!("{}", &message);
                return Err(Error::new(&message, ErrorType::InvalidPackageMeta));
            }
        };

        let maintainer = match kv.get("maintainer") {
            Some(maintainer) => maintainer,
            None => {
                let message = format!("Invalid stanza, maintainer missing!\n{stanza}");
                error!("{}", &message);
                return Err(Error::new(&message, ErrorType::InvalidPackageMeta));
            }
        };

        let description = match kv.get("description") {
            Some(description) => description,
            None => {
                let message = format!("Invalid stanza, description missing!\n{stanza}");
                error!("{}", &message);
                return Err(Error::new(&message, ErrorType::InvalidPackageMeta));
            }
        };

        let mut package = Package::new(name, version, size, &filename, maintainer, description);

        match kv.get("source") {
            Some(source) => {
                package.source = Some(source.clone());
            }
            None => {}
        }

        match kv.get("section") {
            Some(section) => {
                package.section = Some(section.clone());
            }
            None => {}
        }

        match kv.get("architecture") {
            Some(architecture) => {
                package.architecture = Some(Architecture::from_str(&architecture)?);
            }
            None => {}
        }

        match kv.get("md5sum") {
            Some(md5sum) => {
                package.md5sum = Some(md5sum.clone());
            }
            None => {}
        }

        match kv.get("sha1") {
            Some(sha1) => {
                package.sha1 = Some(sha1.clone());
            }
            None => {}
        }

        match kv.get("sha256") {
            Some(sha256) => {
                package.sha256 = Some(sha256.clone());
            }
            None => {}
        }

        match kv.get("sha512") {
            Some(sha512) => {
                package.sha512 = Some(sha512.clone());
            }
            None => {}
        }

        match kv.get("description-md5") {
            Some(description_md5) => {
                package.description_md5 = Some(description_md5.clone());
            }
            None => {}
        }

        match kv.get("homepage") {
            Some(homepage) => {
                package.homepage = Some(homepage.clone());
            }
            None => {}
        }

        match kv.get("priority") {
            Some(priority) => {
                let priority = Priority::from_str(priority)?;
                package.priority = Some(priority);
            }
            None => {}
        }

        match kv.get("essential") {
            Some(essential) => {
                if essential.to_lowercase() == "true" {
                    package.essential = Some(true);
                } else {
                    package.essential = Some(false);
                }
            }
            None => {}
        }

        match kv.get("installed-size") {
            Some(installed_size) => {
                let is = installed_size.parse::<u32>().map_err(|e| {
                    Error::new(
                        &format!("Parsing of installed_size failed! {e}"),
                        ErrorType::InvalidPackageMeta,
                    )
                })?;
                package.installed_size = Some(is);
            }
            None => {}
        };

        match kv.get("depends") {
            Some(depends) => {
                package.depends = parse_package_relation(depends)?;
            }
            None => {}
        };

        match kv.get("pre-depends") {
            Some(pre_depends) => {
                package.pre_depends = parse_package_relation(pre_depends)?;
            }
            None => {}
        };

        match kv.get("recommends") {
            Some(recommends) => {
                package.recommends = parse_package_relation(recommends)?;
            }
            None => {}
        };

        match kv.get("suggests") {
            Some(suggests) => {
                package.suggests = parse_package_relation(suggests)?;
            }
            None => {}
        };

        match kv.get("breaks") {
            Some(breaks) => {
                package.breaks = parse_package_relation(breaks)?;
            }
            None => {}
        };

        match kv.get("conflicts") {
            Some(conflicts) => {
                package.conflicts = parse_package_relation(conflicts)?;
            }
            None => {}
        };

        match kv.get("provides") {
            Some(provides) => {
                package.provides = parse_package_relation(provides)?;
            }
            None => {}
        };

        match kv.get("replaces") {
            Some(replaces) => {
                package.replaces = parse_package_relation(replaces)?;
            }
            None => {}
        };

        match kv.get("enhances") {
            Some(enhances) => {
                package.enhances = parse_package_relation(enhances)?;
            }
            None => {}
        };

        match kv.get("built-using") {
            Some(built_using) => {
                package.built_using = Some(parse_package_relation(built_using)?);
            }
            None => {}
        };

        Ok(package)
    }
}

impl PartialOrd for Package {
    fn partial_cmp(&self, other: &Package) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Package {
    fn cmp(&self, other: &Package) -> Ordering {
        self.version.cmp(&other.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Key;

    #[test]
    fn parse_package() {
        let distro = Distro::repo(
            "http://archive.ubuntu.com/ubuntu",
            "jammy",
            Key::NoSignatureCheck,
        );

        let stanza = r#"
Package: linux-headers-5.15.0-1034-s32
Source: linux-s32
Priority: optional
Section: devel
Installed-Size: 18568
Maintainer: Ubuntu Kernel Team <kernel-team@lists.ubuntu.com>
Architecture: arm64
Version: 5.15.0-1034.43
Provides: linux-headers, linux-headers-3.0
Depends: linux-s32-headers-5.15.0-1034, libc6 (>= 2.34), libelf1 (>= 0.142), libssl3 (>= 3.0.0~~alpha1), zlib1g (>= 1:1.2.3.3)
Filename: pool/main/l/linux-s32/linux-headers-5.15.0-1034-s32_5.15.0-1034.43_arm64.deb
Size: 2794378
MD5sum: 69c3ccf8a2a6a7f52cf2d795520fa036
SHA1: 7fe7be41e74389346df466e000bbeae8e36040ef
SHA256: 70372f37d5206a2d52eef900bbf7fbf09e285aba38dcb66ef5d3ce1385f11a1f
Description: Linux kernel headers for version 5.15.0 on ARMv8 SMP
Description-md5: 2ab472dd12387a67ae9ecbe0508146a7
"#;

        let package = Package::from_stanza(stanza, &distro).unwrap();
        assert_eq!(package.package, "linux-headers-5.15.0-1034-s32");
        assert_eq!(package.source, Some("linux-s32".to_string()));
        assert_eq!(package.priority, Some(Priority::Optional));
        assert_eq!(package.section, Some("devel".to_string()));
        assert_eq!(package.installed_size, Some(18568));
        assert_eq!(
            package.maintainer,
            "Ubuntu Kernel Team <kernel-team@lists.ubuntu.com>"
        );
        assert_eq!(package.architecture, Some(Architecture::Arm64));
        assert_eq!(package.version.epoch, None);
        assert_eq!(package.version.version, "5.15.0");
        assert_eq!(package.version.revision, Some("1034.43".to_string()));
        assert_eq!(package.provides.len(), 2);
        assert_eq!(package.provides[0].name, "linux-headers");
        assert_eq!(package.provides[1].name, "linux-headers-3.0");
        assert_eq!(package.depends.len(), 5);
        assert_eq!(package.depends[0].name, "linux-s32-headers-5.15.0-1034");
        assert_eq!(package.depends[1].name, "libc6");
        assert_eq!(
            package.depends[1].version,
            Some(Version::from_str("2.34").unwrap())
        );
        assert_eq!(package.depends[2].name, "libelf1");
        assert_eq!(
            package.depends[2].version,
            Some(Version::from_str("0.142").unwrap())
        );
        assert_eq!(package.depends[3].name, "libssl3");
        assert_eq!(
            package.depends[3].version,
            Some(Version::from_str("3.0.0~~alpha1").unwrap())
        );
        assert_eq!(package.depends[4].name, "zlib1g");
        assert_eq!(
            package.depends[4].version,
            Some(Version::from_str("1:1.2.3.3").unwrap())
        );
        assert_eq!(
            package.filename,
            "http://archive.ubuntu.com/ubuntu/pool/main/l/linux-s32/linux-headers-5.15.0-1034-s32_5.15.0-1034.43_arm64.deb"
        );
        assert_eq!(package.size, 2794378);
        assert_eq!(
            package.md5sum,
            Some("69c3ccf8a2a6a7f52cf2d795520fa036".to_string())
        );
        assert_eq!(
            package.sha1,
            Some("7fe7be41e74389346df466e000bbeae8e36040ef".to_string())
        );
        assert_eq!(
            package.sha256,
            Some("70372f37d5206a2d52eef900bbf7fbf09e285aba38dcb66ef5d3ce1385f11a1f".to_string())
        );
        assert_eq!(
            package.description,
            "Linux kernel headers for version 5.15.0 on ARMv8 SMP"
        );
        assert_eq!(
            package.description_md5,
            Some("2ab472dd12387a67ae9ecbe0508146a7".to_string())
        );
    }
}
