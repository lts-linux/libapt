//! Implementation of the source types and parsing.
//!
#[cfg(not(test))]
use log::error;

#[cfg(test)]
use std::println as error;

use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap};

use crate::util::{parse_package_relation, parse_stanza};
use crate::{
    Architecture, Distro, Error, ErrorType, Link, LinkHash, PackageVersion, Priority, Result,
    Version,
};

/// A PackageReference is a Debian source package package-list entry.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct PackageReference {
    pub name: String,
    pub package_type: String,
    pub section: String,
    pub priority: Priority,
    pub architecture: Vec<Architecture>,
}

/// The Source struct groups all data about a source package.
///
/// When the source package index file is parsed, all specified values from
/// [Debian Wiki Package Indices specification](https://wiki.debian.org/DebianRepository/Format#A.22Sources.22_Indices)
/// are considered.
/// For parsing the single entries the
/// [Debian Policy Source Package specification](https://www.debian.org/doc/debian-policy/ch-controlfields.html#debian-source-package-control-files-dsc)
/// is used as a base.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Source {
    // fields from apt source package index
    pub format: String,
    pub package: String,
    pub binary: Vec<String>,
    pub architecture: Vec<Architecture>,
    pub version: Version,
    pub maintainer: String,
    pub uploaders: Vec<String>,
    pub homepage: Option<String>,
    pub vcs_arch: Option<String>,
    pub vcs_bzr: Option<String>,
    pub vcs_cvs: Option<String>,
    pub vcs_darcs: Option<String>,
    pub vcs_git: Option<String>,
    pub vcs_hg: Option<String>,
    pub vcs_mtn: Option<String>,
    pub vcs_svn: Option<String>,
    pub vcs_browser: Option<String>,
    pub testsuite: Vec<String>,
    pub dgit: Option<String>,
    pub standards_version: Option<String>,
    pub build_depends: Vec<PackageVersion>,
    pub build_depends_indep: Vec<PackageVersion>,
    pub build_depends_arch: Vec<PackageVersion>,
    pub build_conflicts: Vec<PackageVersion>,
    pub build_conflicts_indep: Vec<PackageVersion>,
    pub build_conflicts_arch: Vec<PackageVersion>,
    pub package_list: Vec<PackageReference>,
    // The links group the checksums with the size and the hash,
    // for all checksums and files.
    pub links: HashMap<String, Link>,
    pub directory: String,
    pub priority: Option<Priority>,
    // list of sections is unstable, not using type.
    pub section: Option<String>,
}

impl Source {
    /// New struct with default values.
    pub fn new(
        format: &str,
        package: &str,
        version: Version,
        maintainer: &str,
        directory: &str,
    ) -> Source {
        Source {
            // fields from apt source package index
            format: format.to_string(),
            package: package.to_string(),
            binary: Vec::new(),
            architecture: Vec::new(),
            version: version,
            maintainer: maintainer.to_string(),
            uploaders: Vec::new(),
            homepage: None,
            vcs_arch: None,
            vcs_bzr: None,
            vcs_cvs: None,
            vcs_darcs: None,
            vcs_git: None,
            vcs_hg: None,
            vcs_mtn: None,
            vcs_svn: None,
            vcs_browser: None,
            testsuite: Vec::new(),
            dgit: None,
            standards_version: None,
            build_depends: Vec::new(),
            build_depends_indep: Vec::new(),
            build_depends_arch: Vec::new(),
            build_conflicts: Vec::new(),
            build_conflicts_indep: Vec::new(),
            build_conflicts_arch: Vec::new(),
            package_list: Vec::new(),
            // The links group the checksums with the size and the hash,
            // for all checksums and files.
            links: HashMap::new(),
            directory: directory.to_string(),
            priority: None,
            // list of sections is unstable, not using type.
            section: None,
        }
    }

    /// Parse a Package from its stanza.
    pub fn from_stanza(stanza: &str, distro: &Distro) -> Result<Source> {
        let kv = parse_stanza(stanza);

        let format = match kv.get("format") {
            Some(name) => name,
            None => {
                let message = format!("Invalid source stanza, format missing!\n{stanza}");
                error!("{}", &message);
                return Err(Error::new(&message, ErrorType::InvalidPackageMeta));
            }
        };

        let package = match kv.get("package") {
            Some(package) => package,
            None => {
                let message = format!("Invalid source stanza, package missing!\n{stanza}");
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

        let maintainer = match kv.get("maintainer") {
            Some(maintainer) => maintainer,
            None => {
                let message = format!("Invalid source stanza, maintainer missing!\n{stanza}");
                error!("{}", &message);
                return Err(Error::new(&message, ErrorType::InvalidPackageMeta));
            }
        };

        let directory = match kv.get("directory") {
            Some(directory) => directory,
            None => {
                let message = format!("Invalid source stanza, directory missing!\n{stanza}");
                error!("{}", &message);
                return Err(Error::new(&message, ErrorType::InvalidPackageMeta));
            }
        };

        let mut source = Source::new(format, package, version, maintainer, directory);

        match kv.get("binary") {
            Some(binary) => {
                source.binary = binary
                    .split(",")
                    .map(|b| b.trim())
                    .map(|b| b.to_string())
                    .collect();
            }
            None => {}
        }

        match kv.get("section") {
            Some(section) => {
                source.section = Some(section.clone());
            }
            None => {}
        }

        match kv.get("architecture") {
            Some(architecture) => {
                let architectures: Result<Vec<Architecture>> = architecture
                    .trim()
                    .split(" ")
                    .map(|a| Architecture::from_str(a.trim()))
                    .collect();
                source.architecture = architectures?;
            }
            None => {}
        }

        match kv.get("priority") {
            Some(priority) => {
                let priority = Priority::from_str(priority)?;
                source.priority = Some(priority);
            }
            None => {}
        }

        match kv.get("homepage") {
            Some(homepage) => {
                source.homepage = Some(homepage.clone());
            }
            None => {}
        }

        match kv.get("build-depends") {
            Some(build_depends) => {
                source.build_depends = parse_package_relation(build_depends)?;
            }
            None => {}
        };

        match kv.get("build-depends-indep") {
            Some(build_depends_indep) => {
                source.build_depends_indep = parse_package_relation(build_depends_indep)?;
            }
            None => {}
        };

        match kv.get("build-depends-arch") {
            Some(build_depends_arch) => {
                source.build_depends_arch = parse_package_relation(build_depends_arch)?;
            }
            None => {}
        };

        match kv.get("build-conflicts") {
            Some(build_conflicts) => {
                source.build_conflicts = parse_package_relation(build_conflicts)?;
            }
            None => {}
        };

        match kv.get("build-conflicts-indep") {
            Some(build_conflicts_indep) => {
                source.build_conflicts_indep = parse_package_relation(build_conflicts_indep)?;
            }
            None => {}
        };

        match kv.get("build-conflicts-arch") {
            Some(build_conflicts_arch) => {
                source.build_conflicts_arch = parse_package_relation(build_conflicts_arch)?;
            }
            None => {}
        };

        match kv.get("uploaders") {
            Some(uploaders) => {
                source.uploaders = uploaders
                    .split(",")
                    .map(|b| b.trim())
                    .map(|b| b.to_string())
                    .collect();
            }
            None => {}
        }

        match kv.get("vcs-arch") {
            Some(vcs_arch) => {
                source.vcs_arch = Some(vcs_arch.clone());
            }
            None => {}
        }

        match kv.get("vcs-bzr") {
            Some(vcs_bzr) => {
                source.vcs_bzr = Some(vcs_bzr.clone());
            }
            None => {}
        }

        match kv.get("vcs-cvs") {
            Some(vcs_cvs) => {
                source.vcs_cvs = Some(vcs_cvs.clone());
            }
            None => {}
        }

        match kv.get("vcs-darcs") {
            Some(vcs_darcs) => {
                source.vcs_darcs = Some(vcs_darcs.clone());
            }
            None => {}
        }

        match kv.get("vcs-git") {
            Some(vcs_git) => {
                source.vcs_git = Some(vcs_git.clone());
            }
            None => {}
        }

        match kv.get("vcs-hg") {
            Some(vcs_hg) => {
                source.vcs_hg = Some(vcs_hg.clone());
            }
            None => {}
        }

        match kv.get("vcs-mtn") {
            Some(vcs_mtn) => {
                source.vcs_mtn = Some(vcs_mtn.clone());
            }
            None => {}
        }

        match kv.get("vcs-svn") {
            Some(vcs_svn) => {
                source.vcs_svn = Some(vcs_svn.clone());
            }
            None => {}
        }

        match kv.get("vcs-browser") {
            Some(vcs_browser) => {
                source.vcs_browser = Some(vcs_browser.clone());
            }
            None => {}
        }

        match kv.get("testsuite") {
            Some(testsuite) => {
                source.testsuite = testsuite
                    .split(",")
                    .map(|b| b.trim())
                    .map(|b| b.to_string())
                    .collect();
            }
            None => {}
        }

        match kv.get("dgit") {
            Some(dgit) => {
                source.dgit = Some(dgit.clone());
            }
            None => {}
        }

        match kv.get("standards-version") {
            Some(standards_version) => {
                source.standards_version = Some(standards_version.clone());
            }
            None => {}
        }

        match kv.get("package-list") {
            Some(package_list) => {
                let list: Vec<&str> = package_list
                    .split("\n")
                    .filter(|l| !l.trim().is_empty())
                    .collect();

                for line in list {
                    let parts: Vec<&str> = line
                        .trim()
                        .split(" ")
                        .map(|p| p.trim())
                        .filter(|l| !l.is_empty())
                        .collect();

                    // Additional values are ignored.
                    if parts.len() < 4 {
                        return Err(Error::new(
                            &format!("Invalid Package-List line: {line}"),
                            ErrorType::InvalidPackageMeta,
                        ));
                    }

                    let name = parts[0].to_string();
                    let package_type = parts[1].to_string();
                    let section = parts[2].to_string();
                    let priority = Priority::from_str(parts[3])?;

                    let architecture: Vec<Architecture> = if parts.len() > 4 {
                        let architecture: Result<Vec<Architecture>> = parts[4]
                            .split(",")
                            .map(|a| a.trim())
                            .map(|a| {
                                if a.starts_with("arch=") {
                                    &a[5..].trim()
                                } else {
                                    a
                                }
                            })
                            .map(|a| Architecture::from_str(a.trim()))
                            .collect();
                        architecture?
                    } else {
                        Vec::new()
                    };

                    let pr = PackageReference {
                        name: name,
                        package_type: package_type,
                        section: section,
                        priority: priority,
                        architecture: architecture,
                    };

                    source.package_list.push(pr);
                }
            }
            None => {}
        }

        source.parse_files(kv.get("files"), distro, LinkHash::Md5, true, stanza)?;
        source.parse_files(
            kv.get("checksums-sha256"),
            distro,
            LinkHash::Sha256,
            true,
            stanza,
        )?;
        source.parse_files(
            kv.get("checksums-sha512"),
            distro,
            LinkHash::Sha512,
            false,
            stanza,
        )?;
        source.parse_files(
            kv.get("checksums-sha1"),
            distro,
            LinkHash::Sha1,
            false,
            stanza,
        )?;

        Ok(source)
    }

    fn parse_files(
        &mut self,
        files: Option<&String>,
        distro: &Distro,
        hash_type: LinkHash,
        required: bool,
        stanza: &str,
    ) -> Result<()> {
        match files {
            Some(files) => {
                let files: Vec<&str> = files
                    .split("\n")
                    .filter(|l| !l.trim().is_empty())
                    .map(|p| p.trim())
                    .collect();

                for file in files {
                    let mut link = Link::form_source(file, distro, self)?;

                    match self.links.get_mut(&link.url) {
                        Some(link) => {
                            link.add_hash(file, hash_type.clone())?;
                        }
                        None => {
                            link.add_hash(file, hash_type.clone())?;
                            self.links.insert(link.url.clone(), link);
                        }
                    }
                }
            }
            None => {
                if required {
                    let message = format!("Invalid stanza, files missing!\n{stanza}");
                    error!("{}", &message);
                    return Err(Error::new(&message, ErrorType::InvalidPackageMeta));
                }
            }
        };

        Ok(())
    }
}

impl PartialOrd for Source {
    fn partial_cmp(&self, other: &Source) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Source {
    fn cmp(&self, other: &Source) -> Ordering {
        self.version.cmp(&other.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Key, VersionRelation};

    #[test]
    fn parse_source() {
        let distro = Distro::repo(
            "http://archive.ubuntu.com/ubuntu",
            "jammy",
            Key::NoSignatureCheck,
        );

        let stanza = r#"
Package: constantly
Format: 3.0 (quilt)
Binary: python3-constantly
Architecture: all
Version: 15.1.0-2
Priority: optional
Section: misc
Maintainer: Debian Python Modules Team <python-modules-team@lists.alioth.debian.org>
Uploaders: Free Ekanayaka <freee@debian.org>
Standards-Version: 3.9.8
Build-Depends: debhelper-compat (= 9), dh-python, python3-all, python3-setuptools (>= 0.6b3)
Homepage: https://github.com/twisted/constantly
Vcs-Browser: https://salsa.debian.org/python-team/modules/constantly
Vcs-Git: https://salsa.debian.org/python-team/modules/constantly.git
Directory: pool/main/c/constantly
Package-List:
 python3-constantly deb python optional arch=all
Files:
 807a24c0019e9b1c8e3b6a0654a3b040 2032 constantly_15.1.0-2.dsc
 f0762f083d83039758e53f8cf0086eef 21465 constantly_15.1.0.orig.tar.gz
 4c52076736ca1c069f436be9308b42aa 2612 constantly_15.1.0-2.debian.tar.xz
Checksums-Sha1:
 30834594e62c0cbd8a8fa05168b877f77164f9e3 2032 constantly_15.1.0-2.dsc
 02e60c17889d029e48a52a74259462e087a3dcdd 21465 constantly_15.1.0.orig.tar.gz
 b905b08c9be3c6e1a308c0b62e1a56305fc291f8 2612 constantly_15.1.0-2.debian.tar.xz
Checksums-Sha256:
 af28fa59bb101ff6469a7d3e709e75658163e523df52a4f00b596ed2cfa5c45b 2032 constantly_15.1.0-2.dsc
 586372eb92059873e29eba4f9dec8381541b4d3834660707faf8ba59146dfc35 21465 constantly_15.1.0.orig.tar.gz
 40e5a20cd6a157de997b71cc1a95393cacd23d9a6ff9bc2bd021cb983f785835 2612 constantly_15.1.0-2.debian.tar.xz
Checksums-Sha512:
 043542750e6d37dd994c468775dc442581d6c7dec42446ed4ef46a75e1e2ad3b4ee7ea48bc3a5dff67576d382d76d12e95289025db952de52c95da232c7fcbf7 2032 constantly_15.1.0-2.dsc
 ccc6f41b0bd552d2bb5346cc9d64cd7b91a59dd30e0cf66b01e82f7e0e079c01c34bc6c66b69c5fee9d2eed35ae5455258d309e66278d708d5f576ddf2e00ac3 21465 constantly_15.1.0.orig.tar.gz
 4795112fc25d74214a89df6ecdb935fd107f3b8cce79c49cd0c1b57354f914e10b90857eec3c78dd10c8234ff69d4825c8ab7c06cf317a6d11a8f40a98e62aeb 2612 constantly_15.1.0-2.debian.tar.xz
"#;

        let source = Source::from_stanza(stanza, &distro).unwrap();
        assert_eq!(source.package, "constantly");
        assert_eq!(source.format, "3.0 (quilt)".to_string());
        assert_eq!(source.binary, vec!["python3-constantly"]);
        assert_eq!(source.architecture, vec![Architecture::All]);
        assert_eq!(source.version.epoch, None);
        assert_eq!(source.version.version, "15.1.0");
        assert_eq!(source.version.revision, Some("2".to_string()));
        assert_eq!(source.priority, Some(Priority::Optional));
        assert_eq!(source.section, Some("misc".to_string()));
        assert_eq!(
            source.maintainer,
            "Debian Python Modules Team <python-modules-team@lists.alioth.debian.org>"
        );
        assert_eq!(source.uploaders, vec!["Free Ekanayaka <freee@debian.org>"]);
        assert_eq!(source.standards_version, Some("3.9.8".to_string()));

        assert_eq!(source.build_depends.len(), 4);

        assert_eq!(source.build_depends[0].name, "debhelper-compat");
        assert_eq!(
            source.build_depends[0].version,
            Some(Version::from_str("9").unwrap())
        );
        assert_eq!(
            source.build_depends[0].relation,
            Some(VersionRelation::from_str("=").unwrap())
        );

        assert_eq!(source.build_depends[1].name, "dh-python");
        assert_eq!(source.build_depends[1].version, None);
        assert_eq!(source.build_depends[1].relation, None);

        assert_eq!(source.build_depends[2].name, "python3-all");
        assert_eq!(source.build_depends[2].version, None);
        assert_eq!(source.build_depends[2].relation, None);

        assert_eq!(source.build_depends[3].name, "python3-setuptools");
        assert_eq!(
            source.build_depends[3].version,
            Some(Version::from_str("0.6b3").unwrap())
        );
        assert_eq!(
            source.build_depends[3].relation,
            Some(VersionRelation::from_str(">=").unwrap())
        );

        assert_eq!(
            source.homepage,
            Some("https://github.com/twisted/constantly".to_string())
        );
        assert_eq!(
            source.vcs_browser,
            Some("https://salsa.debian.org/python-team/modules/constantly".to_string())
        );
        assert_eq!(
            source.vcs_git,
            Some("https://salsa.debian.org/python-team/modules/constantly.git".to_string())
        );
        assert_eq!(source.directory, "pool/main/c/constantly".to_string());

        assert_eq!(source.package_list.len(), 1);

        println!("{:?}", source.package_list);

        assert_eq!(source.package_list[0].name, "python3-constantly");
        assert_eq!(source.package_list[0].package_type, "deb");
        assert_eq!(source.package_list[0].section, "python");
        assert_eq!(source.package_list[0].priority, Priority::Optional);
        assert_eq!(source.package_list[0].architecture, vec![Architecture::All]);

        assert_eq!(source.links.len(), 3);

        let url = "http://archive.ubuntu.com/ubuntu/pool/main/c/constantly/constantly_15.1.0-2.dsc";
        let link = source.links.get(url).unwrap();
        assert_eq!(link.url, url);
        assert_eq!(link.size, 2032);
        assert_eq!(
            link.hashes.get(&LinkHash::Md5).unwrap(),
            "807a24c0019e9b1c8e3b6a0654a3b040"
        );
        assert_eq!(
            link.hashes.get(&LinkHash::Sha1).unwrap(),
            "30834594e62c0cbd8a8fa05168b877f77164f9e3"
        );
        assert_eq!(
            link.hashes.get(&LinkHash::Sha256).unwrap(),
            "af28fa59bb101ff6469a7d3e709e75658163e523df52a4f00b596ed2cfa5c45b"
        );
        assert_eq!(link.hashes.get(&LinkHash::Sha512).unwrap(), "043542750e6d37dd994c468775dc442581d6c7dec42446ed4ef46a75e1e2ad3b4ee7ea48bc3a5dff67576d382d76d12e95289025db952de52c95da232c7fcbf7");

        let url =
            "http://archive.ubuntu.com/ubuntu/pool/main/c/constantly/constantly_15.1.0.orig.tar.gz";
        let link = source.links.get(url).unwrap();
        assert_eq!(link.url, url);
        assert_eq!(link.size, 21465);
        assert_eq!(
            link.hashes.get(&LinkHash::Md5).unwrap(),
            "f0762f083d83039758e53f8cf0086eef"
        );
        assert_eq!(
            link.hashes.get(&LinkHash::Sha1).unwrap(),
            "02e60c17889d029e48a52a74259462e087a3dcdd"
        );
        assert_eq!(
            link.hashes.get(&LinkHash::Sha256).unwrap(),
            "586372eb92059873e29eba4f9dec8381541b4d3834660707faf8ba59146dfc35"
        );
        assert_eq!(link.hashes.get(&LinkHash::Sha512).unwrap(), "ccc6f41b0bd552d2bb5346cc9d64cd7b91a59dd30e0cf66b01e82f7e0e079c01c34bc6c66b69c5fee9d2eed35ae5455258d309e66278d708d5f576ddf2e00ac3");

        let url = "http://archive.ubuntu.com/ubuntu/pool/main/c/constantly/constantly_15.1.0-2.debian.tar.xz";
        let link = source.links.get(url).unwrap();
        assert_eq!(link.url, url);
        assert_eq!(link.size, 2612);
        assert_eq!(
            link.hashes.get(&LinkHash::Md5).unwrap(),
            "4c52076736ca1c069f436be9308b42aa"
        );
        assert_eq!(
            link.hashes.get(&LinkHash::Sha1).unwrap(),
            "b905b08c9be3c6e1a308c0b62e1a56305fc291f8"
        );
        assert_eq!(
            link.hashes.get(&LinkHash::Sha256).unwrap(),
            "40e5a20cd6a157de997b71cc1a95393cacd23d9a6ff9bc2bd021cb983f785835"
        );
        assert_eq!(link.hashes.get(&LinkHash::Sha512).unwrap(), "4795112fc25d74214a89df6ecdb935fd107f3b8cce79c49cd0c1b57354f914e10b90857eec3c78dd10c8234ff69d4825c8ab7c06cf317a6d11a8f40a98e62aeb");
    }
}
