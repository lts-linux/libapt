//! Implementation of the package types and parsing.
//!
#[cfg(not(test))]
use log::error;

use std::collections::HashMap;
#[cfg(test)]
use std::println as error;

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::util::{parse_package_relation, parse_stanza};
use crate::{
    Architecture, Distro, Error, ErrorType, Link, PackageVersion, Priority, Result, Version,
};

/// The Package struct groups all data about a package.
///
/// When the package index file is parsed, all specified values from
/// [Debian Wiki Package Indices specification](https://wiki.debian.org/DebianRepository/Format#A.22Packages.22_Indices)
/// are considered.
/// For parsing the single entries the
/// [Debian Wiki Binary Package specification](https://www.debian.org/doc/debian-policy/ch-controlfields.html#debian-binary-package-control-files-debian-control)
/// is used as a base.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
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
    pub installed_size: Option<u32>,
    pub link: Link,
    pub maintainer: String,
    pub description: String,
    pub description_md5: Option<String>,
    pub homepage: Option<String>,
    pub built_using: Vec<PackageVersion>,
    pub issues: Vec<Error>,
}

impl Package {
    /// New struct with default values.
    pub fn new(
        package: &str,
        version: Version,
        size: usize,
        filename: &str,
        maintainer: &str,
        description: &str,
    ) -> Package {
        let link = Link {
            url: filename.to_string(),
            size: size,
            hashes: HashMap::new(),
        };

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
            installed_size: None,
            link: link,
            maintainer: maintainer.to_string(),
            description: description.to_string(),
            description_md5: None,
            homepage: None,
            built_using: Vec::new(),
            issues: Vec::new(),
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
                return Err(Error::new(&message, ErrorType::PackageFormat));
            }
        };

        let version = match kv.get("version") {
            Some(version) => Version::from_str(version)?,
            None => {
                let message = format!("Invalid stanza, version missing!\n{stanza}");
                error!("{}", &message);
                return Err(Error::new(&message, ErrorType::PackageFormat));
            }
        };

        let size = match kv.get("size") {
            Some(size) => size.parse::<usize>().map_err(|e| {
                Error::new(
                    &format!("Parsing of size failed! {e}"),
                    ErrorType::PackageFormat,
                )
            })?,
            None => {
                let message = format!("Invalid stanza, version missing!\n{stanza}");
                error!("{}", &message);
                return Err(Error::new(&message, ErrorType::PackageFormat));
            }
        };

        let filename = match kv.get("filename") {
            Some(filename) => distro.url(&filename, true),
            None => {
                let message = format!("Invalid stanza, filename missing!\n{stanza}");
                error!("{}", &message);
                return Err(Error::new(&message, ErrorType::PackageFormat));
            }
        };

        let maintainer = match kv.get("maintainer") {
            Some(maintainer) => maintainer,
            None => {
                let message = format!("Invalid stanza, maintainer missing!\n{stanza}");
                error!("{}", &message);
                return Err(Error::new(&message, ErrorType::PackageFormat));
            }
        };

        let description = match kv.get("description") {
            Some(description) => description,
            None => {
                let message = format!("Invalid stanza, description missing!\n{stanza}");
                error!("{}", &message);
                return Err(Error::new(&message, ErrorType::PackageFormat));
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
                package
                    .link
                    .hashes
                    .insert(crate::LinkHash::Md5, md5sum.to_string());
            }
            None => {}
        }

        match kv.get("sha1") {
            Some(sha1) => {
                package
                    .link
                    .hashes
                    .insert(crate::LinkHash::Sha1, sha1.to_string());
            }
            None => {}
        }

        match kv.get("sha256") {
            Some(sha256) => {
                package
                    .link
                    .hashes
                    .insert(crate::LinkHash::Sha256, sha256.to_string());
            }
            None => {}
        }

        match kv.get("sha512") {
            Some(sha512) => {
                package
                    .link
                    .hashes
                    .insert(crate::LinkHash::Sha512, sha512.to_string());
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
            Some(priority) => match Priority::from_str(priority) {
                Ok(priority) => {
                    package.priority = Some(priority);
                }
                Err(e) => package.issues.push(e),
            },
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
                        ErrorType::PackageFormat,
                    )
                });
                match is {
                    Ok(is) => {
                        package.installed_size = Some(is);
                    }
                    Err(e) => {
                        package.issues.push(e);
                    }
                }
            }
            None => {}
        };

        match kv.get("depends") {
            Some(depends) => match parse_package_relation(depends) {
                Ok(depends) => {
                    package.depends = depends;
                }
                Err(e) => {
                    package.issues.push(e);
                }
            },
            None => {}
        };

        match kv.get("pre-depends") {
            Some(pre_depends) => match parse_package_relation(pre_depends) {
                Ok(pre_depends) => {
                    package.pre_depends = pre_depends;
                }
                Err(e) => {
                    package.issues.push(e);
                }
            },
            None => {}
        };

        match kv.get("recommends") {
            Some(recommends) => match parse_package_relation(recommends) {
                Ok(recommends) => {
                    package.recommends = recommends;
                }
                Err(e) => {
                    package.issues.push(e);
                }
            },
            None => {}
        };

        match kv.get("suggests") {
            Some(suggests) => match parse_package_relation(suggests) {
                Ok(suggests) => {
                    package.suggests = suggests;
                }
                Err(e) => {
                    package.issues.push(e);
                }
            },
            None => {}
        };

        match kv.get("breaks") {
            Some(breaks) => match parse_package_relation(breaks) {
                Ok(breaks) => {
                    package.breaks = breaks;
                }
                Err(e) => {
                    package.issues.push(e);
                }
            },
            None => {}
        };

        match kv.get("conflicts") {
            Some(conflicts) => match parse_package_relation(conflicts) {
                Ok(conflicts) => {
                    package.conflicts = conflicts;
                }
                Err(e) => {
                    package.issues.push(e);
                }
            },
            None => {}
        };

        match kv.get("provides") {
            Some(provides) => match parse_package_relation(provides) {
                Ok(provides) => {
                    package.provides = provides;
                }
                Err(e) => {
                    package.issues.push(e);
                }
            },
            None => {}
        };

        match kv.get("replaces") {
            Some(replaces) => match parse_package_relation(replaces) {
                Ok(replaces) => {
                    package.replaces = replaces;
                }
                Err(e) => {
                    package.issues.push(e);
                }
            },
            None => {}
        };

        match kv.get("enhances") {
            Some(enhances) => match parse_package_relation(enhances) {
                Ok(enhances) => {
                    package.enhances = enhances;
                }
                Err(e) => {
                    package.issues.push(e);
                }
            },
            None => {}
        };

        match kv.get("built-using") {
            Some(built_using) => match parse_package_relation(built_using) {
                Ok(built_using) => {
                    package.built_using = built_using;
                }
                Err(e) => {
                    package.issues.push(e);
                }
            },
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
    use crate::{Key, LinkHash};

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

        let link = package.link;
        assert_eq!(
            link.url,
            "http://archive.ubuntu.com/ubuntu/pool/main/l/linux-s32/linux-headers-5.15.0-1034-s32_5.15.0-1034.43_arm64.deb"
        );
        assert_eq!(link.size, 2794378);
        assert_eq!(
            link.hashes.get(&LinkHash::Md5).unwrap(),
            "69c3ccf8a2a6a7f52cf2d795520fa036"
        );
        assert_eq!(
            link.hashes.get(&LinkHash::Sha1).unwrap(),
            "7fe7be41e74389346df466e000bbeae8e36040ef"
        );
        assert_eq!(
            link.hashes.get(&LinkHash::Sha256).unwrap(),
            "70372f37d5206a2d52eef900bbf7fbf09e285aba38dcb66ef5d3ce1385f11a1f"
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
