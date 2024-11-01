use libapt::{Distro, Key, Release, PackageIndex, Architecture, SourceIndex};

fn main() {
    let key = Key::key("/etc/apt/trusted.gpg.d/ubuntu-keyring-2018-archive.gpg");

    // Default repository format using a public key form an URL.
    let distro = Distro::repo(
        "http://archive.ubuntu.com/ubuntu",
        "jammy",
        key.clone(),
    );

    // Parse the InRelease file.
    let release = Release::from_distro(&distro).unwrap();

    // Check for compliance with https://wiki.debian.org/DebianRepository/Format#A.22Release.22_files.
    match release.check_compliance() {
        Ok(_) => println!("The Ubuntu Jammy release is compliant with the standard."),
        Err(e) => println!("The Ubuntu Jammy release violates the standard: {e}"),
    };

    // Parse the package index of the main component for the amd64 architecture.
    let main_amd64 = PackageIndex::new(&release, "main", &Architecture::Amd64).unwrap();

    println!("Ubuntu Jammy main provides {} packages for amd64.", main_amd64.package_count());

    // Get a Package from the package index.
    let busybox = main_amd64.get("busybox-static", None).unwrap();

    println!("Ubuntu Jammy main provides busybox-static version {:?}.", busybox.version);

    // Ubuntu Jammy signing key.
    let key = Key::key("/etc/apt/trusted.gpg.d/ubuntu-keyring-2018-archive.gpg");

    // Ubuntu Jammy distribution.
    let distro = Distro::repo(
        "http://archive.ubuntu.com/ubuntu",
        "jammy",
        key,
    );

    // Parse the InRelease file.
    let release = Release::from_distro(&distro).unwrap();

    // Parse the package index of the main component for the amd64 architecture.
    let main_sources = SourceIndex::new(&release, "main").unwrap();

    println!("Ubuntu Jammy main provides {} source packages.", main_sources.package_count());

    // Get a Package from the package index.
    let busybox = main_sources.get("busybox", None).unwrap();

    println!("Ubuntu Jammy main provides busybox version {:?}.", busybox.version);
}

#[cfg(test)]
mod tests {
    use assert_cmd::prelude::*;
    use std::process::Command;

    #[test]
    fn hello() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("hello")?;

        let output = cmd.output()?;
        let stdout = output.stdout;
        let stdout = String::from_utf8_lossy(&stdout).to_string();
        assert!(stdout.contains("The Ubuntu Jammy release is compliant with the standard."));

        Ok(())
    }
}
