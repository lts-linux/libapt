use reqwest::blocking::Client;

use crate::{Error, Result};

pub fn download(url: &str) -> Result<String> {
    let client = Client::new();
    Ok(client
        .get(url)
        .send()
        .map_err(Error::from_reqwest)?
        .text()
        .map_err(Error::from_reqwest)?)
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
