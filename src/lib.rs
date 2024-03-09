//! [![github]](https://github.com/collinoc/cargo-lookup)
//!
//! [github]: https://img.shields.io/badge/github-blue?style=for-the-badge&logo=github&link=https%3A%2F%2Fgithub.com%2Fcollinoc%2Fcargo-lookup
//!
//! A library for querying Rust crate registries
//!
//! ## Examples
//!
//! Get all info for a package:
//! ```no_run
//! use cargo_lookup::{Query, Result};
//!
//! fn main() -> Result<()> {
//!     let query: Query = "cargo".parse()?;
//!     let all_package_info = query.package()?;
//!
//!     println!("{all_package_info:?}");
//!
//!     Ok(())
//! }
//! ```
//!
//! Get a specific release of a package:
//! ```no_run
//! use cargo_lookup::{Query, Result};
//!
//! fn main() -> Result<()> {
//!     let query: Query = "cargo@=0.2.153".parse()?;
//!     let specific_release = query.submit()?;
//!
//!     println!("{specific_release:?}");
//!
//!     Ok(())
//! }
//! ```

#![deny(clippy::all)]

pub mod error;
#[cfg(test)]
mod tests;

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, str::FromStr};

use error::Error;

/// The default crates.io index URL
pub const CRATES_IO_INDEX_URL: &str = "https://index.crates.io";

pub type Result<T> = std::result::Result<T, Error>;

/// A query for a specific rust package based on the packages name, an option version requirement,
/// in an optional custom index. By default, [`CRATES_IO_INDEX_URL`] will be used as the index
#[derive(Debug, Clone)]
pub struct Query {
    name: String,
    version_req: Option<VersionReq>,
    custom_index: Option<String>,
}

impl FromStr for Query {
    type Err = Error;

    fn from_str(name: &str) -> std::result::Result<Self, Self::Err> {
        let (name, version_req) = match name.split_once('@') {
            Some((name, version)) if !version.is_empty() => (name, Some(version)),
            _ => (name, None),
        };

        let version_req = version_req
            .map(|req| VersionReq::parse(req).map_err(Error::InvalidVersion))
            .transpose()?;

        Ok(Self {
            name: name.to_owned(),
            version_req,
            custom_index: None,
        })
    }
}

impl Query {
    /// USe a custom crate index for this query
    pub fn with_index<T>(mut self, custom_index: T) -> Self
    where
        String: From<T>,
    {
        self.custom_index = Some(String::from(custom_index));
        self
    }

    /// Return the raw contents of the index file found by this query
    pub fn raw_index(&self) -> Result<String> {
        let index_url = self.custom_index.as_deref().unwrap_or(CRATES_IO_INDEX_URL);
        let index_path = get_index_path(&self.name);
        let response = ureq::get(&format!("{index_url}/{index_path}"))
            .call()
            .map_err(|err| Error::Request(Box::new(err)))?
            .into_string()
            .map_err(Error::Io)?;

        Ok(response)
    }

    /// Return all of the info for the package found by this query
    pub fn package(&self) -> Result<Package> {
        Package::from_index(self.raw_index()?)
    }

    /// Return a specific release of a package found by this query
    ///
    /// If no version requirement ws specified, the latest version of the found package
    /// will be returned
    pub fn submit(&self) -> Result<Option<Release>> {
        let package = self.package()?;

        match self.version_req {
            Some(ref version_req) => Ok(package.into_version(version_req)),
            None => Ok(package.into_latest()),
        }
    }
}

/// All info on a package from it's index file, including all of it's releases
#[derive(Debug, Clone)]
pub struct Package {
    name: String,
    index_path: String,
    releases: Vec<Release>,
}

impl Package {
    /// Return this package's name
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Return the index path for this package
    pub fn index_path(&self) -> &str {
        self.index_path.as_str()
    }

    /// Return a list of releases for this package
    pub fn releases(&self) -> &Vec<Release> {
        &self.releases
    }

    /// Convert into a packages latest release
    pub fn into_latest(mut self) -> Option<Release> {
        self.releases.pop()
    }

    /// Get a packages latest release
    pub fn latest(&self) -> Option<&Release> {
        self.releases.last()
    }

    /// Convert to a package release from a given version requirement
    ///
    /// This will find the latest possible release that matches the version requirement
    ///
    /// For example, with a version requirement of `^0.1.0`, this will return `0.1.9` before it
    /// will return `0.1.8`
    pub fn into_version(self, version_req: &semver::VersionReq) -> Option<Release> {
        self.releases
            .into_iter()
            .rev()
            .find(|release| version_req.matches(&release.vers))
    }

    /// Find a package release from a given version requirement
    ///
    /// This will find the latest possible release that matches the version requirement
    ///
    /// For example, with a version requirement of `^0.1.0`, this will return `0.1.9` before it
    /// will return `0.1.8`
    pub fn version(&self, version_req: &semver::VersionReq) -> Option<&Release> {
        self.releases
            .iter()
            .rev()
            .find(|release| version_req.matches(&release.vers))
    }

    /// Parse a package from it's index file
    pub fn from_index<T>(content: T) -> Result<Self>
    where
        T: AsRef<str>,
    {
        let content = content.as_ref();

        let releases: Result<Vec<Release>> = content
            .lines()
            .map(|release| serde_json::from_str(release).map_err(Error::Deserialize))
            .collect();
        let releases = releases?;

        let name = releases
            .last()
            .ok_or(Error::FromIndexFile("empty"))?
            .name
            .clone();

        let index_path = get_index_path(&name);

        Ok(Package {
            name,
            index_path,
            releases,
        })
    }
}

// See: https://github.com/serde-rs/serde/issues/368
const fn one() -> u32 {
    1
}

/// An entry for a given release version of a package
///
/// A package index file contains one line for each release of a package in json format, from oldest to latest.
///
/// More info on the schema can be found in [The Cargo Book](https://doc.rust-lang.org/cargo/reference/registry-index.html#json-schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    /// The name of the package
    pub name: String,
    /// The specific version of the release
    pub vers: Version,
    /// List of direct dependencies of this package
    pub deps: Vec<Dependency>,
    /// The SHA256 checksum of the releases .crate
    pub cksum: String,
    /// A mapping of features' names to the features/dependencies they enable
    pub features: Features,
    /// Whether or not this release has been yanked
    pub yanked: bool,
    /// The `links` value from this packages manifest
    pub links: Option<String>,
    /// Value indicating the schema version for this entry
    ///
    /// Defaults to `1`
    #[serde(default = "one")]
    pub v: u32,
    /// A mapping of features with new extended syntax including
    /// namespaced features and weak dependencies
    pub features2: Option<Features>,
    /// The minimum supported rust version requirement without operator
    pub rust_version: Option<VersionReq>,
}

impl Release {
    /// Convert the release to it's json representation
    pub fn as_json_string(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Error::Serialize)
    }
}

pub type Features = BTreeMap<String, Vec<String>>;

/// A dependency of a package
///
/// The structure can be found in [The Cargo Book](https://doc.rust-lang.org/cargo/reference/registry-index.html#json-schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// The name of the dependency
    pub name: String,
    /// The SemVer requirement for the dependency
    pub req: VersionReq,
    /// List of features enabled for this dependency
    pub features: Vec<String>,
    /// Whether this dependency is optional or not
    pub optional: bool,
    /// Whether default features are enabled for this dependency or not
    pub default_features: bool,
    /// Target platform for the dependency
    pub target: Option<String>,
    /// The dependency kind (dev, build, or normal)
    pub kind: Option<String>,
    /// The URL of the index where this dependency is from. Defaults to current registry
    pub registry: Option<String>,
    /// If dependency is renamed, this is the name of the actual dependend upon package
    pub package: Option<String>,
}

/// Get the index path for a package
///
/// ## Examples
///
/// ```
/// use cargo_lookup::get_index_path;
///
/// assert_eq!(get_index_path("cargo"), "ca/rg/cargo");
/// assert_eq!(get_index_path("ice"), "3/i/ice");
/// ```
pub fn get_index_path<T>(package: T) -> String
where
    T: AsRef<str>,
{
    let package = package.as_ref();

    let path = match package.len() {
        1 => format!("1/{package}"),
        2 => format!("2/{package}"),
        3 => {
            let first_char = &package[..1];
            format!("3/{first_char}/{package}")
        }
        _ => {
            let first_two_chars = &package[..2];
            let next_two_chars = &package[2..4];
            format!("{first_two_chars}/{next_two_chars}/{package}")
        }
    };

    path.to_ascii_lowercase()
}
