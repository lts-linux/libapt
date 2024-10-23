mod distro;
mod error;
mod package_version;
mod package;
mod release;
mod signature;
mod types;
mod util;
mod version;

pub use distro::{Distro, Key};
pub use error::{Error, ErrorType, Result};
pub use release::{FileHash, Release};
pub use types::architecture::Architecture;
pub use types::priority::Priority;
pub use version::Version;
pub use package_version::{PackageVersion, VersionRelation};
pub use package::Package;

use crate::util::download;

pub fn parse_distro(distro: &Distro) -> Result<Release> {
    let url = distro.in_release_url()?;
    let content = download(&url)?;
    let content = signature::verify_in_release(content, &distro)?;
    let release = Release::parse(&content, &distro)?;

    
    Ok(release)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ubuntu_jammy_release_file() {
        let distro = Distro::repo(
            "http://archive.ubuntu.com/ubuntu",
            "jammy",
            Key::NoSignatureCheck,
        );
        let release = parse_distro(&distro);

        let release = release.unwrap();

        assert_eq!(release.origin, Some("Ubuntu".to_string()), "Origin");
        assert_eq!(release.label, Some("Ubuntu".to_string()), "Label");
        assert_eq!(release.suite, Some("jammy".to_string()), "Suite");
        assert_eq!(release.codename, Some("jammy".to_string()), "Codename");
        assert_eq!(release.version, Some("22.04".to_string()), "Version");
        assert_eq!(release.acquire_by_hash, true, "Acquire-By-Hash");
    }
}
