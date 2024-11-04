//! Implementation of the package index parsing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use crate::Result;
use crate::{util::download_compressed, Error};
use crate::{Architecture, Link, Package, PackageVersion, Release};

/// A PackageIndex is a set of packages for a specific architecture and component.
#[derive(Debug, Deserialize, Serialize)]
pub struct PackageIndex {
    /// Architecture of the packages.
    pub architecture: Architecture,
    /// Map of packages, key is the package name.
    /// Vec is used to handle the case of different package versions.
    pub package_map: HashMap<String, Vec<Package>>,
    /// Package parsing issues.
    pub issues: Vec<Error>,
}

impl PackageIndex {
    /// Parse a package index.
    pub fn new(
        release: &Release,
        component: &str,
        architecture: &Architecture,
    ) -> Result<PackageIndex> {
        if architecture == &Architecture::Source {
            return Err(Error::new(
                "Source architecture is not supported by this method!",
                crate::ErrorType::ApiUsage,
            ));
        }

        let mut package_index = PackageIndex {
            architecture: architecture.clone(),
            package_map: HashMap::new(),
            issues: Vec::new(),
        };

        let link = release.get_package_index_link(component, architecture)?;

        package_index.issues = package_index.parse_index(&link, release)?;

        Ok(package_index)
    }

    /// Download the package index, verify the hash, and parse the content.
    fn parse_index(&mut self, link: &Link, release: &Release) -> Result<Vec<Error>> {
        let content = download_compressed(&link)?;
        let mut issues = Vec::new();

        for stanza in content.split("\n\n") {
            let stanza = stanza.trim();

            if stanza.is_empty() {
                continue;
            }

            match Package::from_stanza(stanza, &release.distro) {
                Ok(package) => self.add(package),
                Err(e) => issues.push(e),
            }
        }

        Ok(issues)
    }

    // Add package to index.
    fn add(&mut self, package: Package) {
        match self.package_map.get_mut(&package.package) {
            Some(list) => {
                list.push(package);
            }
            None => {
                self.package_map
                    .insert(package.package.clone(), vec![package]);
            }
        }
    }

    // Get package with the given name and fitting version.
    pub fn get(&self, name: &str, version: Option<PackageVersion>) -> Option<Package> {
        match self.package_map.get(name) {
            Some(packages) => {
                let packages = match &version {
                    Some(rel) => {
                        let mut packages: Vec<Package> = packages
                            .iter()
                            .filter(|p| name == p.package)
                            .filter(|p| rel.matches(&p.version))
                            .map(|p| p.clone())
                            .collect();
                        packages.sort();
                        packages
                    }
                    None => {
                        let mut packages = packages.clone();
                        packages.sort();
                        packages
                    }
                };

                if !packages.is_empty() {
                    Some(packages[packages.len() - 1].clone())
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Get all available versions of the given binary package.
    pub fn get_all(&self, name: &str) -> Vec<Package> {
        match self.package_map.get(name) {
            Some(sources) => sources.clone(),
            None => Vec::new(),
        }
    }

    /// Get the number of packages in this index.
    pub fn package_count(&self) -> usize {
        self.package_map.len()
    }

    /// Get all package names.
    pub fn packages(&self) -> Vec<&String> {
        self.package_map.keys().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Architecture, Distro, Key, Release};

    use super::PackageIndex;

    #[test]
    fn parse_ubuntu_jammy_main_amd64() {
        // Ubuntu Jammy signing key.
        let key = Key::key("/etc/apt/trusted.gpg.d/ubuntu-keyring-2018-archive.gpg");

        // Ubuntu Jammy distribution.
        let distro = Distro::repo("http://archive.ubuntu.com/ubuntu", "jammy", key);

        let release = Release::from_distro(&distro).unwrap();

        let package_index = PackageIndex::new(&release, "main", &Architecture::Amd64).unwrap();

        assert_eq!(package_index.architecture, Architecture::Amd64);

        println!("Package count: {}", package_index.package_count());
        assert!(package_index.package_count() > 5000);

        let busybox = package_index.get("busybox-static", None).unwrap();
        assert_eq!(busybox.package, "busybox-static".to_string());
        assert_eq!(busybox.architecture, Some(Architecture::Amd64));
    }

    #[test]
    fn parse_source_index() {
        // Ubuntu Jammy signing key.
        let key = Key::key("/etc/apt/trusted.gpg.d/ubuntu-keyring-2018-archive.gpg");

        // Ubuntu Jammy distribution.
        let distro = Distro::repo("http://archive.ubuntu.com/ubuntu", "jammy", key);

        let release = Release::from_distro(&distro).unwrap();

        match PackageIndex::new(&release, "main", &Architecture::Source) {
            Ok(_) => assert!(false), // Binary and source indices use different formats.
            Err(_) => {}             // Expected.
        }
    }
}
