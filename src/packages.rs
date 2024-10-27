#[cfg(not(test))] 
use log::error;

#[cfg(test)]
use std::println as error;

use std::collections::HashMap;

use crate::{PackageVersion, Priority, Version, Result, Error, ErrorType};

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
    installed_size: Option<u32>,
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

impl Package {
    pub fn new(
        name: &str,
        version: Version,
        size: u32,
        filename: &str,
        maintainer: &str,
        description: &str,
) -> Package {
        Package {
            package: name.to_string(),
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

    pub fn parse(stanza: &str) -> Result<Package> {
        let kv = Package::parse_stanza(stanza);

        let name = match kv.get("name") {
            Some(name) => name,
            None => {
                let message = format!("Invalid stanza, name missing!\n{stanza}");
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
            Some(size) => size.parse::<u32>().map_err(|e|
                Error::new(&format!("Parsing of size failed! {e}"), ErrorType::InvalidPackageMeta)
            )?,
            None => {
                let message = format!("Invalid stanza, version missing!\n{stanza}");
                error!("{}", &message);
                return Err(Error::new(&message, ErrorType::InvalidPackageMeta));
            }
        };
        
        let filename = match kv.get("filename") {
            Some(filename) => filename,
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

        let mut package = Package::new(name, version, size, filename, maintainer, description);

        match kv.get("source") {
            Some(source) => {
                package.source = Some(source.clone());
            }
            None => {},
        }

        match kv.get("section") {
            Some(section) => {
                package.section = Some(section.clone());
            }
            None => {},
        }

        match kv.get("architecture") {
            Some(architecture) => {
                package.architecture = Some(architecture.clone());
            }
            None => {},
        }

        match kv.get("md5sum") {
            Some(md5sum) => {
                package.md5sum = Some(md5sum.clone());
            }
            None => {},
        }

        match kv.get("sha1") {
            Some(sha1) => {
                package.sha1 = Some(sha1.clone());
            }
            None => {},
        }

        match kv.get("sha256") {
            Some(sha256) => {
                package.sha256 = Some(sha256.clone());
            }
            None => {},
        }

        match kv.get("sha512") {
            Some(sha512) => {
                package.sha512 = Some(sha512.clone());
            }
            None => {},
        }

        match kv.get("description_md5") {
            Some(description_md5) => {
                package.description_md5 = Some(description_md5.clone());
            }
            None => {},
        }

        match kv.get("homepage") {
            Some(homepage) => {
                package.homepage = Some(homepage.clone());
            }
            None => {},
        }

        match kv.get("priority") {
            Some(priority) => {
                let priority = Priority::from_str(priority)?;
                package.priority = Some(priority);
            }
            None => {},
        }

        match kv.get("essential") {
            Some(essential) => {
                if essential.to_lowercase() == "true" {
                    package.essential = Some(true);
                } else {
                    package.essential = Some(false);
                }
            }
            None => {},
        }
    
        match kv.get("installed_size") {
            Some(installed_size) => {
                let is = installed_size.parse::<u32>()
                    .map_err(|e| Error::new(
                        &format!("Parsing of installed_size failed! {e}"),
                        ErrorType::InvalidPackageMeta))?;
                package.installed_size = Some(is);
            },
            None => {}
        };

        match kv.get("depends") {
            Some(depends) => {
                package.depends = Package::parse_package_relation(depends)?;
            },
            None => {}
        };

        match kv.get("pre_depends") {
            Some(pre_depends) => {
                package.pre_depends = Package::parse_package_relation(pre_depends)?;
            },
            None => {}
        };

        match kv.get("recommends") {
            Some(recommends) => {
                package.recommends = Package::parse_package_relation(recommends)?;
            },
            None => {}
        };

        match kv.get("suggests") {
            Some(suggests) => {
                package.suggests = Package::parse_package_relation(suggests)?;
            },
            None => {}
        };

        match kv.get("breaks") {
            Some(breaks) => {
                package.breaks = Package::parse_package_relation(breaks)?;
            },
            None => {}
        };

        match kv.get("conflicts") {
            Some(conflicts) => {
                package.conflicts = Package::parse_package_relation(conflicts)?;
            },
            None => {}
        };

        match kv.get("provides") {
            Some(provides) => {
                package.provides = Package::parse_package_relation(provides)?;
            },
            None => {}
        };

        match kv.get("replaces") {
            Some(replaces) => {
                package.replaces = Package::parse_package_relation(replaces)?;
            },
            None => {}
        };

        match kv.get("enhances") {
            Some(enhances) => {
                package.enhances = Package::parse_package_relation(enhances)?;
            },
            None => {}
        };

        match kv.get("built_using") {
            Some(built_using) => {
                package.built_using = Some(Package::parse_package_relation(built_using)?);
            },
            None => {}
        };

        Ok(package)
    }

    fn parse_package_relation(depends: &str) -> Result<Vec<PackageVersion>> {
        let pvs: Vec<Result<PackageVersion>> = depends
            .split(",")
            .map(|p| p.trim())
            .map(|p| PackageVersion::from_str(p))
            .collect();

        let mut result: Vec<PackageVersion> = Vec::new();

        for pv in pvs {
            let pv = pv?;
            result.push(pv);
        }

        Ok(result)
    }

    fn parse_stanza(stanza: &str) -> HashMap<String, String> {
        let mut kv = HashMap::new();

        let mut key = "";
        let mut value = String::new();

        for line in stanza.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if line.starts_with(' ') {
                if key.is_empty() {
                    error!("Continuation line found without keyword! {line}")
                } else {
                    value += "\n";
                    value += line.trim();
                }
            } else {
                if !key.is_empty() {
                    kv.insert(key.to_lowercase(), value.clone());
                }

                match line.find(':') {
                    Some(pos) => {
                        key = line[..pos].trim();
                        value = line[(pos+1)..].trim().to_string();
                    },
                    None => {
                        error!("Invalid line: {line}")
                    }
                }
            }
        }

        kv
    }
}

