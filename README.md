# libapt

Libapt is a pure Rust apt library.

You can find the sources at [Codeberg](https://codeberg.org/tomirgang/libapt) and [Github](https://github.com/lts-linux/libapt).

## Architecture

This crate provides a pure Rust interface for Debian repositories.

### Debian apt repositories

The format of Debian, i.e. apt, repositories is described in the [Debian wiki](https://wiki.debian.org/DebianRepository/Format#Debian_Repository_Format).

In general, apt repositories consist of distributions and packages.
The path to this file on the server depends on the repository type.
By default, a apt repository is structured in the following way:

```plain
Root --- /dists ------ /dist_a
      \          \---- /dist_b
       \          \--- ...
        \
         \--/pool ----- /a
                   \--- /b                  
                    \-- ...
```

The dists folder contains one or many distributions.

Each distribution is defined by a _InRelease_, or _Release_, index file.
This index file is signed using GPG key of the distribution provider.
The matching public key is typically installed on the local machine,
and the client tool verifies the signature to ensure the repository was not manipulated.

The distribution index file contains relative paths and checksums to package index files.
These package indices are grouped by _components_.
The client tool downloads the wanted component indices,
verifies the integrity using the checksum,
and parses the packages _[stanzas](https://wiki.debian.org/DebianRepository/Format#A.22Packages.22_Indices)_.

The package indices provide, as part of the stanzas, paths to the packages,
relative to the root of the apt repository, and checksums for each package.
These checksums allow verification of the packages after download.

The packages are typically placed in a _pool_ folder,
and this folder typically uses a sub-folder structure consisting of
first letter or lib plus first letter and name of the source package.
This structure is not relevant for the apt repository client tooling,
because it's part of the paths provided in the component package indices.

Some tools generate _[flat repositories](https://wiki.debian.org/DebianRepository/Format#Flat_Repository_Format)_ instead of default repositories. 
These repositories work in the same way, but use a _path_ instead of a distribution name.
The distribution _InRelease_ file for these repositories is located at _root_ / _path_ / _InRelease_.
These repositories typically also doesn't follow the pool structure for organizing the packages.

### libapt

The goal of _libapt_ is to provide a pure Rust library for interfacing with Debian apt repositories.
It shall be possible to use _libapt_ to parse an apt repository,
get the packages, package metadata, checksums and locations,
and verify the validity of the metadata with respect to
signatures, compliance with the Debian repository format specification,
and existence of packages indices and packages.

#### Struct Distro

The struct [Distro] is the starting point to parse an apt repository.
It groups all information which need to be provided by a user to locate
and verify the _InRelease_ distribution index.

For verifying the integrity of the distribution index, a public key
needs to be provided. This pubic key can be armored public GPG key,
e.g. form a URL or a local file, or a non-armored binary GPG key,
e.g. form the local `/etc/apt/trusted.gpg.d/`.
For receiving both types of keys, a download is tried if the provided string starts with 'http',
else the string is interpreted as a local path.

If no verification is wanted, the value _Key::NoSignatureCheck_ can be provided.

```rust
use libapt::{Distro, Key};

let key = Key::key("/etc/apt/trusted.gpg.d/ubuntu-keyring-2018-archive.gpg");

// Default repository format using a public key form an URL.
let distro = Distro::repo(
    "http://archive.ubuntu.com/ubuntu",
    "jammy",
    key.clone(),
);

// Flat repository format using a public key form an URL.
let distro = Distro::flat_repo(
    "http://archive.ubuntu.com/ubuntu",
    "dists/jammy",
    key.clone(),
);

// Flat repo skipping verification.
let distro = Distro::flat_repo(
    "http://archive.ubuntu.com/ubuntu",
    "dists/jammy",
    Key::NoSignatureCheck,
);
```

#### Struct Release

The struct [Release] groups all information contained in the InRelease file.
When a new [Release] is created by running [Release::from_distro],
the content of the InRelease file is downloaded,
the inline signature is verified using the [Distro] key,
and the content is parsed.

```rust
use libapt::{Distro, Key, Release};

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

// Check for compliance with https://wiki.debian.org/DebianRepository/Format#A.22Release.22_files.
release.check_compliance();
```

The struct [Release] provides also some convenience methods to access the package indices.

The method [Release::get_package_links] provides all available package indices.
This means, for each combination of architecture and component,
the function [get_etag] is used to check if the URL really exists,
and if the URL exists, the [Link] is provided.
The return value of this method is a _Vec<(String, Architecture, Link)>_,
and each tuple consists of _component name_, _architecture_, and _URL to the index as [Link]_.

The method [Release::get_package_index_link] provides the [Link] to one specific package index.
The parameters are the _component name_ and the _architecture_, and the result is a _[Link]_.

#### Struct Link

The [Link] struct groups references to files, i.e. URLs,
with the hash sums to verify the file, and the size of the file.
The supported hash types are MD5, SHA1, SHA256 and SHA512.
When the file is downloaded, the hash is verified to ensure the integrity.

#### Struct PackageIndex

The struct [PackageIndex] groups all packages of one component and architecture.
This structure also provides support for parsing the apt repository package index files.


## Limitations

- Apt repositories providing only the old _Release_ with detached _Release.gpg_ signature are not supported.
- Host dependencies on Ubuntu Linux:
    - rust-lzma requires _liblzma-dev_ and _pkg-config_
    - reqwest requires _libssl-dev_

## Usage

TODO: describe

## Examples

Libapt provides some example applications.
You can run the examples using the command `cargo run --example NAME`,
where name is the name of the example.

### Test the examples

Some of the example tests are black box tests and depend on the binaries to exist.
Run `cargo build --examples` before `cargo test --examples`, else these tests will fail.
