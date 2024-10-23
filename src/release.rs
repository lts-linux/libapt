use chrono::DateTime;
use chrono::FixedOffset;
use log::warn;
use std::collections::HashMap;

use crate::Architecture;
use crate::Distro;
use crate::{Error, ErrorType, Result, Package};
use crate::util::join_url;


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileHash {
    Md5Sum(String),
    Sha1(String),
    Sha256(String),
    Sha512(String),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Link {
    pub url: String,
    pub size: usize,
    pub hashes: Vec<FileHash>,
}

#[derive(Debug)]
pub struct Release {
    // fields from apt release file
    // see https://wiki.debian.org/DebianRepository/Format#A.22Release.22_files
    hash: Option<String>,
    pub origin: Option<String>,
    pub label: Option<String>,
    pub suite: Option<String>,
    pub version: Option<String>,
    pub codename: Option<String>,
    pub date: Option<DateTime<FixedOffset>>,
    pub valid_until: Option<DateTime<FixedOffset>>,
    pub architectures: Vec<Architecture>,
    pub components: Vec<String>,
    pub description: Option<String>,
    pub links: HashMap<String, Link>,
    pub acquire_by_hash: bool,
    pub signed_by: Vec<String>,
    pub changelogs: Option<String>,
    pub snapshots: Option<String>,
    // internal data
    distro: Distro,
    // Component, Name, [Package, ...]
    packages: HashMap<Architecture, HashMap<String, Vec<Package>>>,
}

impl Release {
    pub fn new(distro: &Distro) -> Release {
        Release {
            hash: None,
            origin: None,
            label: None,
            suite: None,
            version: None,
            codename: None,
            date: None,
            valid_until: None,
            architectures: Vec::new(),
            components: Vec::new(),
            description: None,
            links: HashMap::new(),
            acquire_by_hash: false, // default is false
            signed_by: Vec::new(),
            changelogs: None,
            snapshots: None,
            // internal data
            distro: distro.clone(),
            packages: HashMap::new(),
        }
    }

    pub fn parse(content: &str, distro: &Distro) -> Result<Release> {
        let mut section = ReleaseSection::Keywords;
        let mut release = Release::new(distro);

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if line.starts_with("---") {
                if line.starts_with("-----BEGIN PGP SIGNATURE") {
                    // Signature check not implemented. Stop parsing.
                    break;
                } else {
                    // Skip PGP headers.
                    continue;
                }
            } else if !line.starts_with(" ") {
                section = ReleaseSection::Keywords;
            }

            match &section {
                ReleaseSection::Keywords => {
                    if !line.contains(":") {
                        return Err(Error::new(
                            &format!("Invalid line! {line}"),
                            ErrorType::InvalidReleaseFormat,
                        ));
                    }

                    let mut parts = line.splitn(2, ":");
                    let keyword = parts.next().unwrap();
                    let value = parts.next().unwrap();

                    let keyword = keyword.to_lowercase();
                    let plain_value = value.trim();
                    let value = Some(plain_value.to_string());

                    if keyword == "hash" {
                        release.hash = value;
                    } else if keyword == "origin" {
                        release.origin = value;
                    } else if keyword == "label" {
                        release.label = value;
                    } else if keyword == "suite" {
                        release.suite = value;
                    } else if keyword == "version" {
                        release.version = value;
                    } else if keyword == "codename" {
                        release.codename = value;
                    } else if keyword == "description" {
                        release.description = value;
                    } else if keyword == "changelogs" {
                        release.changelogs = value;
                    } else if keyword == "snapshots" {
                        release.snapshots = value;
                    } else if keyword == "date" {
                        let plain_value = plain_value.replace("UTC", "+0000");
                        release.date = match DateTime::parse_from_rfc2822(&plain_value) {
                            Ok(date) => Some(date),
                            Err(e) => {
                                warn!("Parsing Release date \"{plain_value}\" failed! {e}");
                                None
                            }
                        }
                    } else if keyword == "valid-until" {
                        let plain_value = plain_value.replace("UTC", "+0000");
                        release.valid_until = match DateTime::parse_from_rfc2822(&plain_value) {
                            Ok(date) => Some(date),
                            Err(e) => {
                                warn!("Parsing Release valid until failed! {e}");
                                None
                            }
                        }
                    } else if keyword == "architectures" {
                        release.architectures = plain_value
                            .split(" ")
                            .filter_map(|e| match Architecture::from_str(e) {
                                Ok(arch) => Some(arch),
                                Err(e) => {
                                    warn!("Parsing architecture {e} failed!");
                                    None
                                }
                            })
                            .collect();
                    } else if keyword == "components" {
                        release.components = plain_value
                            .split(" ")
                            .filter(|e| !e.trim().is_empty())
                            .map(|e| e.to_string())
                            .collect();
                    } else if keyword == "acquire-by-hash" {
                        release.acquire_by_hash = match plain_value.to_lowercase().as_str() {
                            "yes" => true,
                            _ => false,
                        }
                    } else if keyword == "signed-by" {
                        release.signed_by = plain_value
                            .split(",")
                            .map(|e| e.trim().to_string())
                            .collect();
                    } else if keyword == "md5sum" {
                        section = ReleaseSection::Files(FileHash::Md5Sum("".to_string()));
                    } else if keyword == "sha1" {
                        section = ReleaseSection::Files(FileHash::Sha1("".to_string()));
                    } else if keyword == "sha256" {
                        section = ReleaseSection::Files(FileHash::Sha256("".to_string()));
                    } else if keyword == "sha512" {
                        section = ReleaseSection::Files(FileHash::Sha512("".to_string()));
                    } else {
                        warn!("Unknown keyword: {keyword} of line {line}!");
                    }
                }
                ReleaseSection::Files(hash) => {
                    let parts: Vec<String> = line
                        .split(" ")
                        .filter(|e| !e.trim().is_empty())
                        .map(|e| e.trim().to_string())
                        .collect();

                    if parts.len() != 3 {
                        warn!("Invalid file line! {line}");
                        continue;
                    }

                    let size = match parts[1].parse::<usize>() {
                        Ok(size) => size,
                        Err(e) => {
                            warn!("Invalid file size of line {line}! {e}");
                            continue;
                        }
                    };
                    let url = distro.url(&parts[2]);
                    let hash = match hash {
                        FileHash::Md5Sum(_) => FileHash::Md5Sum(parts[0].clone()),
                        FileHash::Sha1(_) => FileHash::Sha1(parts[0].clone()),
                        FileHash::Sha256(_) => FileHash::Sha256(parts[0].clone()),
                        FileHash::Sha512(_) => FileHash::Sha512(parts[0].clone()),
                    };

                    if release.links.contains_key(&url) {
                        let link = release.links.get_mut(&url).unwrap();
                        if link.size != size {
                            let link_size = link.size;
                            warn!("Size mismatch for {url}! {link_size} != {size}")
                        }
                        link.hashes.push(hash);
                    } else {
                        release.links.insert(
                            url.clone(),
                            Link {
                                url: url,
                                size: size,
                                hashes: vec![hash],
                            },
                        );
                    }
                }
            }
        }

        // init package map
        for arch in &release.architectures { 
            release.packages.insert(arch.clone(), HashMap::new());
        }

        Ok(release)
    }

    pub fn parse_components(&self) -> Result<()> {
        for component in &self.components {
            self.parse_component(component)?;
        }
        Ok(())
    }

    pub fn parse_component(&self, component: &str) -> Result<()> {
        // source packages
        self.parse_component_arch(component, &Architecture::Source)?;

        // binary packages
        for arch in &self.architectures {
            self.parse_component_arch(component, arch)?;
        }

        Ok(())
    }

    fn parse_component_arch(&self, component: &str, arch: &Architecture) -> Result<()> {
        let url = if arch == &Architecture::Source {
            let path = format!("{component}/source/Sources");
            self.distro.url(&path)
        } else {
            let arch_str = arch.to_string();
            let path = format!("{component}/binary-{arch_str}/Packages");
            self.distro.url(&path)
        };
        let url = self.distro.url(&url);

        let extensions = vec!["xz", "bz2", "gz", ""];
        for ext in extensions {
            let package_index = join_url(&url, &ext);
            if self.links.contains_key(&package_index) {
                // TODO: download and parse package index.
            } else {
                log::info!("Index {package_index} not found.");
            }
        }

        Ok(())
    }
}

enum ReleaseSection {
    Keywords,
    Files(FileHash),
}
