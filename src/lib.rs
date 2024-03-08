#![deny(clippy::all)]

pub mod error;

use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub const CRATES_IO_INDEX_URL: &str = "https://index.crates.io";

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub struct Query {
    name: String,
    version_req: Option<VersionReq>,
}

impl Query {
    pub fn parse(name: &str) -> Result<Self> {
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
        })
    }

    pub fn package(&self) -> Result<Package> {
        let index_path = get_index_path(&self.name);
        let response = ureq::get(&format!("{CRATES_IO_INDEX_URL}/{index_path}"))
            .call()
            .map_err(|err| Error::Request(Box::new(err)))?
            .into_string()
            .map_err(Error::Io)?;

        Package::from_index_file(response)
    }

    pub fn submit(&self) -> Result<Option<Release>> {
        let package = self.package()?;

        match self.version_req {
            Some(ref version_req) => Ok(package.version(version_req).cloned()),
            None => Ok(package.latest().cloned()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Package {
    name: String,
    index_path: String,
    releases: Vec<Release>,
}

impl Package {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn index_path(&self) -> &str {
        self.index_path.as_str()
    }

    pub fn latest(&self) -> Option<&Release> {
        self.releases.last()
    }

    pub fn version(&self, version_req: &semver::VersionReq) -> Option<&Release> {
        self.releases
            .iter()
            .rev()
            .find(|release| version_req.matches(&release.vers))
    }

    pub fn releases(&self) -> &Vec<Release> {
        &self.releases
    }

    pub fn from_index_file<T>(content: T) -> Result<Self>
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
            .take()
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub name: String,
    pub vers: Version,
    pub deps: Vec<Dependency>,
    pub cksum: String,
    pub features: Features,
    pub yanked: bool,
    pub links: Option<String>,
    #[serde(default = "one")]
    pub v: u32,
    pub features2: Option<Features>,
}

impl Release {
    pub fn as_json_string(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Error::Serialize)
    }
}

const fn one() -> u32 {
    1
}

pub type Features = BTreeMap<String, Vec<String>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    name: String,
    req: VersionReq,
    features: Vec<String>,
    optional: bool,
    default_features: bool,
    target: Option<String>,
    kind: Option<String>,
    registry: Option<String>,
    package: Option<String>,
}

pub fn get_index_path<T>(package: T) -> String
where
    T: AsRef<str>,
{
    let package = package.as_ref();

    match package.len() {
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
    }
}
