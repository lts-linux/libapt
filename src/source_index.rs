//! Implementation of the source index parsing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::util::download_compressed;
pub use crate::Result;
use crate::{Architecture, Error, Link, PackageVersion, Release, Source};

#[derive(Debug, Deserialize, Serialize)]
/// A SourceIndex is a set of packages for a specific architecture and component.
pub struct SourceIndex {
    /// Map of source packages, key is the source package name.
    /// Vec is used to handle the case of different package versions.
    pub package_map: HashMap<String, Vec<Source>>,
    pub issues: Vec<Error>,
}

impl SourceIndex {
    /// Parse a package index.
    pub fn new(release: &Release, component: &str) -> Result<SourceIndex> {
        let mut source_index = SourceIndex {
            package_map: HashMap::new(),
            issues: Vec::new(),
        };

        let link = release.get_package_index_link(component, &Architecture::Source)?;

        source_index.parse_index(&link, release)?;

        Ok(source_index)
    }

    /// Download the source package index, verify the hash, and parse the content.
    fn parse_index(&mut self, link: &Link, release: &Release) -> Result<()> {
        let content = download_compressed(&link)?;

        for stanza in content.split("\n\n") {
            let stanza = stanza.trim();

            if stanza.is_empty() {
                continue;
            }

            match Source::from_stanza(stanza, &release.distro) {
                Ok(source) => self.add(source),
                Err(e) => self.issues.push(e),
            }
        }

        Ok(())
    }

    // Add package to index.
    fn add(&mut self, source: Source) {
        match self.package_map.get_mut(&source.package) {
            Some(list) => {
                list.push(source);
            }
            None => {
                self.package_map
                    .insert(source.package.clone(), vec![source]);
            }
        }
    }

    // Get package with the given name and fitting version.
    pub fn get(&self, name: &str, version: Option<PackageVersion>) -> Option<Source> {
        match self.package_map.get(name) {
            Some(packages) => {
                let packages = match &version {
                    Some(rel) => {
                        let mut packages: Vec<Source> = packages
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

    /// Get all available versions of the given source package.
    pub fn get_all(&self, name: &str) -> Vec<Source> {
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

    use super::SourceIndex;

    #[test]
    fn parse_ubuntu_jammy_main_sources() {
        // Ubuntu Jammy signing key.
        let key = Key::key("/etc/apt/trusted.gpg.d/ubuntu-keyring-2018-archive.gpg");

        // Ubuntu Jammy distribution.
        let distro = Distro::repo("http://archive.ubuntu.com/ubuntu", "jammy", key);

        let release = Release::from_distro(&distro).unwrap();

        let package_index = SourceIndex::new(&release, "main").unwrap();

        println!("Package count: {}", package_index.package_count());
        assert!(package_index.package_count() > 2000);

        let busybox = package_index.get("busybox", None).unwrap();
        assert_eq!(busybox.package, "busybox".to_string());
        assert_eq!(
            busybox.architecture,
            vec![Architecture::Any, Architecture::All]
        );
    }
}
