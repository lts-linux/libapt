//! Helper functions.

#[cfg(not(test))]
use log::{error, info};

#[cfg(test)]
use std::{println as info, println as error};

use std::{collections::HashMap, io::Read};

use flate2::bufread::GzDecoder;
use lzma;
use md5;
use reqwest::blocking::Client;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

use crate::{Error, Link, LinkHash, PackageVersion, Result};

/// Get the timestamp when the URL was last modified.
pub fn get_etag(url: &str) -> Result<String> {
    let client = Client::new();
    let response = client
        .head(url)
        .send()
        .map_err(|e| Error::from_reqwest(e, url))?;

    if !response.status().is_success() {
        return Err(Error::new(
            &format!("Url {url} download failed!"),
            crate::ErrorType::Download,
        ));
    }

    let etag = match response.headers().get("etag") {
        Some(etag) => etag,
        None => {
            return Err(Error::new(
                &format!("No etag found in header of {url}!"),
                crate::ErrorType::Download,
            ));
        }
    };

    let etag = etag
        .to_str()
        .map_err(|e| Error::from_to_str_error(e, url))?;

    Ok(etag.to_string())
}

/// Download the content of the given URL as a String.
pub fn download(url: &str) -> Result<String> {
    let client = Client::new();
    Ok(client
        .get(url)
        .send()
        .map_err(|e| Error::from_reqwest(e, url))?
        .text()
        .map_err(|e| Error::from_reqwest(e, url))?)
}

/// Verify the hash of the downloaded content for the given Link.
fn verify_hash(content: &Vec<u8>, link: &Link) -> Result<()> {
    let (name, hash, data_hash) = if let Some(hash) = link.hashes.get(&LinkHash::Sha512) {
        let mut sha512 = Sha512::new();
        sha512.update(content);
        let data_hash = sha512.finalize();
        let data_hash = format!("{:x}", data_hash);

        ("SHA512", hash.clone(), data_hash)
    } else if let Some(hash) = link.hashes.get(&LinkHash::Sha256) {
        let mut sha256 = Sha256::new();
        sha256.update(content);
        let data_hash = sha256.finalize();
        let data_hash = format!("{:x}", data_hash);

        ("SHA256", hash.clone(), data_hash)
    } else if let Some(hash) = link.hashes.get(&LinkHash::Sha1) {
        let mut sha1 = Sha1::new();
        sha1.update(content);
        let data_hash = sha1.finalize();
        let data_hash = format!("{:x}", data_hash);

        ("SHA1", hash.clone(), data_hash)
    } else if let Some(hash) = link.hashes.get(&LinkHash::Md5) {
        let digest = md5::compute(content);
        let data_hash = format!("{:x}", digest);

        ("MD5", hash.clone(), data_hash)
    } else {
        return Err(Error::new(
            &format!("No hash for URL {} provided!", &link.url),
            crate::ErrorType::Download,
        ));
    };

    let hash = hash.to_lowercase();

    if hash != data_hash {
        return Err(Error::new(
            &format!("{} hash verification of URL {} failed!", name, &link.url),
            crate::ErrorType::Download,
        ));
    } else {
        info!("Verified {} hash for URL {} successfully.", name, &link.url);
        Ok(())
    }
}

/// Download and decompress the content of the given URL as a String.
///
/// The compression type is guessed using the extension.
/// Known extensions are "xz" and "gz".
/// In case of an unknown extension, no compression is guessed.
pub fn download_compressed(link: &Link) -> Result<String> {
    let url = &link.url;

    let client = Client::new();

    let result = client
        .get(url)
        .send()
        .map_err(|e| Error::from_reqwest(e, url))?;

    let text = if url.ends_with(".xz") {
        let data = result.bytes().map_err(|e| Error::from_reqwest(e, url))?;

        verify_hash(&data.to_vec(), link)?;

        let content = lzma::decompress(&data).map_err(|e| Error::from_lzma(e, url))?;
        String::from_utf8_lossy(&content).to_string()
    } else if url.ends_with(".gz") {
        let data = result.bytes().map_err(|e| Error::from_reqwest(e, url))?;

        verify_hash(&data.to_vec(), link)?;

        let mut gz = GzDecoder::new(&data[..]);
        let mut text = String::new();
        gz.read_to_string(&mut text)
            .map_err(|e| Error::from_io_error(e, url))?;
        text
    } else {
        info!("No known extension, assuming plain text.");

        let data = result.bytes().map_err(|e| Error::from_reqwest(e, url))?;
        let data = data.to_vec();

        verify_hash(&data, link)?;

        String::from_utf8(data).map_err(|e| Error::from_utf8_error(e, url))?
    };

    Ok(text)
}

/// Join a base URL with a path string.
///
/// The string "./" is ignored.
pub fn join_url(base: &str, path: &str) -> String {
    let url: String = if base.ends_with("/") {
        base.to_string()
    } else {
        base.to_string() + "/"
    };

    let path: &str = if path == "./" {
        ""
    } else if path.starts_with("/") {
        &path[1..]
    } else {
        path
    };

    url + path
}

/// Transform stanza into a key value hashmap.
pub fn parse_stanza(stanza: &str) -> HashMap<String, String> {
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
                    value = line[(pos + 1)..].trim().to_string();
                }
                None => {
                    error!("Invalid line: {line}")
                }
            }
        }
    }

    if !key.is_empty() {
        kv.insert(key.to_lowercase(), value.clone());
    }

    kv
}

/// Parse a package dependency and relation field.
pub fn parse_package_relation(depends: &str) -> Result<Vec<PackageVersion>> {
    let pvs: Result<Vec<Vec<PackageVersion>>> = depends
        .split(",")
        .map(|p| p.trim())
        .map(|p| PackageVersion::from_str(p))
        .collect();
    let pvs = pvs?;
    let pvs: Vec<PackageVersion> = pvs.iter().flatten().map(|pv| pv.clone()).collect();

    Ok(pvs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_ubuntu_jammy_release_metadata() {
        let url = "http://archive.ubuntu.com/ubuntu/dists/jammy/Release";
        let text = download(url).unwrap();
        assert!(!text.is_empty(), "Content is not empty");
    }

    #[test]
    fn test_join_url() {
        let base = "http://archive.ubuntu.com/";
        let path = "ubuntu";
        let url = join_url(base, path);
        assert!(
            url == "http://archive.ubuntu.com/ubuntu",
            "base ends with slash"
        );

        let base = "http://archive.ubuntu.com";
        let path = "ubuntu";
        let url = join_url(base, path);
        assert!(
            url == "http://archive.ubuntu.com/ubuntu",
            "base doesn't end with slash"
        );
    }

    #[test]
    fn test_get_etag() {
        let etag = get_etag("http://archive.ubuntu.com/ubuntu/dists/noble/InRelease").unwrap();
        println!("etag: {etag}");

        match get_etag("http://archive.ubuntu.com/ubuntu/dists/norbert/InRelease") {
            Ok(_) => assert!(false), // No etag for invalid url.
            Err(_) => {}
        };
    }
}
