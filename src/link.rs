//! Implementation of Link type, which represents apt file reverences.

#[cfg(not(test))]
use log::warn;

#[cfg(test)]
use std::println as warn;

use std::collections::HashMap;

use crate::{source::Source, util::join_url, Distro, Error, ErrorType, Result};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum LinkHash {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

/// Link represents a file referenced from InRelease.
///
/// This type is used to group all hashes for a referenced path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub url: String,
    pub size: usize,
    pub hashes: HashMap<LinkHash, String>,
}

impl Link {
    pub fn form_release(line: &str, distro: &Distro) -> Result<Link> {
        let link = Link {
            url: Link::url_of_line(line, distro, false, None)?,
            size: Link::size_of_line(line)?,
            hashes: HashMap::new(),
        };

        Ok(link)
    }

    pub fn form_source(line: &str, distro: &Distro, source: &Source) -> Result<Link> {
        let link = Link {
            url: Link::url_of_line(line, distro, true, Some(&source.directory))?,
            size: Link::size_of_line(line)?,
            hashes: HashMap::new(),
        };

        Ok(link)
    }

    fn split_line(line: &str) -> Result<Vec<&str>> {
        let parts: Vec<&str> = line
            .trim()
            .split(" ")
            .filter(|p| !p.trim().is_empty())
            .collect();

        if parts.len() != 3 {
            let message = format!("Invalid URL reference: {line}");
            return Err(Error::new(&message, ErrorType::InvalidPackageMeta));
        }

        Ok(parts)
    }

    fn hash_of_line(line: &str) -> Result<String> {
        let parts = Link::split_line(line)?;
        Ok(parts[0].to_string())
    }

    fn size_of_line(line: &str) -> Result<usize> {
        let parts = Link::split_line(line)?;

        let size = match parts[1].parse::<usize>() {
            Ok(size) => size,
            Err(e) => {
                let message = format!("Invalid URL reference, invalid size: {e}\n{line}");
                return Err(Error::new(&message, ErrorType::InvalidPackageMeta));
            }
        };

        Ok(size)
    }

    fn url_of_line(
        line: &str,
        distro: &Distro,
        package: bool,
        path: Option<&str>,
    ) -> Result<String> {
        let parts = Link::split_line(line)?;

        let path = match path {
            Some(prefix) => join_url(prefix, &parts[2]),
            None => parts[2].to_string(),
        };

        let url = distro.url(&path, package);
        Ok(url)
    }

    pub fn add_hash(&mut self, line: &str, hash_type: LinkHash) -> Result<()> {
        let hash = Link::hash_of_line(line)?;

        let size = Link::size_of_line(line)?;
        if self.size != size {
            warn!("Size mismatch of line {line}. {size} != {}", self.size);
        }

        self.hashes.insert(hash_type, hash);
        Ok(())
    }
}
