use std::io::Read;

use flate2::bufread::GzDecoder;
use lzma;
use reqwest::blocking::Client;

use crate::{Error, Result};

pub fn download(url: &str) -> Result<String> {
    let client = Client::new();
    Ok(client
        .get(url)
        .send()
        .map_err(|e| Error::from_reqwest(e, url))?
        .text()
        .map_err(|e| Error::from_reqwest(e, url))?)
}

pub fn download_compressed(url: &str) -> Result<String> {
    let client = Client::new();
    let result = client.get(url).send().map_err(|e| Error::from_reqwest(e, url))?;

    let text = if url.ends_with(".xz") {
        let data = result.bytes().map_err(|e| Error::from_reqwest(e, url))?;
        let content = lzma::decompress(&data).map_err(|e| Error::from_lzma(e, url))?;
        String::from_utf8_lossy(&content).to_string()
    } else if url.ends_with(".gz") {
        let data = result.bytes().map_err(|e| Error::from_reqwest(e, url))?;
        let mut gz = GzDecoder::new(&data[..]);
        let mut text = String::new();
        gz.read_to_string(&mut text).map_err(|e| Error::from_io_error(e, url))?;
        text
    } else {
        log::info!("No known extension, assuming plain text.");
        result.text().map_err(|e| Error::from_reqwest(e, url))?
    };

    Ok(text)
}

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
}
