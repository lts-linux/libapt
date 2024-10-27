#[cfg(not(test))] 
use log::info;

#[cfg(test)]
use std::println as info;

use std::collections::HashMap;

use crate::{release::Link, Architecture, Distro, Package, Release, VersionRelation};
pub use crate::{Error, ErrorType, Result};
use crate::util::download_compressed;

pub struct PackageIndex {
    pub architecture: Architecture,
    package_map: HashMap<String, Vec<Package>>,
}

impl PackageIndex {
    pub fn new(distro: &Distro, release: &Release, component: &str, architecture: &Architecture) -> Result<PackageIndex> {
        if architecture == &Architecture::Source {
            return Err(Error::new("Source packages are not supported by PackageIndex type!", ErrorType::InvalidArchitecture));
        }

        let mut package_index = PackageIndex {
            architecture: architecture.clone(),
            package_map: HashMap::new(),
        };
        
        let index_url = if architecture == &Architecture::Source {
            format!("{component}/source/Sources")
        } else {
            let arch_str = architecture.to_string();
            format!("{component}/binary-{arch_str}/Packages")
        };
        let index_url = distro.url(&index_url, false);

        package_index.parse(&index_url, distro, release)?;

        Ok(package_index)
    }

    fn parse(&mut self, url: &str, distro: &Distro, release: &Release) -> Result<()> {
        // Supported compression extensions, try form best to no compression
        let extensions = vec![".xz", ".gz", ""];

        for ext in extensions {
            // Build URL for compressed index.
            let package_index = url.to_string() + ext;

            // Find link in release.
            // The link is mandatory to get the hash sums for verification.
            match release.links.get(&package_index) {
                Some(link) => {
                    self.parse_index(link, distro)?;
                    return Ok(());
                },
                None => {
                    info!("Index {package_index} not found.");
                }
            }
        }

        // No link found.
        Err(Error::new(
            &format!("No matching package index found! url: {}", url),
            ErrorType::DownloadFailure,
        ))
    }

    fn parse_index(&mut self, link: &Link, distro: &Distro) -> Result<()> {
        let content = download_compressed(&link.url)?;

        // TODO: verify checksum!

        for stanza in content.split("\n\n") {
            let stanza = stanza.trim();
            
            if stanza.is_empty() {
                continue;
            }

            let package = Package::from_stanza(stanza, distro)?;
            self.add(package);
        }

        Ok(())
    }

    fn add(&mut self, package: Package) {
        match self.package_map.get_mut(&package.package) {
            Some(list) => {
                list.push(package);
            },
            None => {
                self.package_map.insert(package.package.clone(), vec![package]);
            }
        }
    }

    pub fn get(&self, name: &str, version: Option<VersionRelation>) -> Option<&Package> {
        match self.package_map.get(name) {
            Some(packages) => {
                match version {
                    Some(_version) => {
                        // TODO: impl package version filter
                        packages.first()
                    },
                    None => {
                        // TODO: return most recent package
                        packages.first()
                    }
                }
                
            },
            None => None
        }
    }

    pub fn package_count(&self) -> usize {
        self.package_map.len()
    }

    pub fn packages(&self) -> Vec<&String> {
        self.package_map.keys().collect()
    }
}

// TODO: test