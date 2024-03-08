#![deny(clippy::all)]

pub mod error;

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, str::FromStr};

use error::Error;

pub const CRATES_IO_INDEX_URL: &str = "https://index.crates.io";

pub type Result<T> = std::result::Result<T, Error>;

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
    pub fn with_index<T>(mut self, custom_index: T) -> Self
    where
        String: From<T>,
    {
        self.custom_index = Some(String::from(custom_index));
        self
    }

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

    pub fn package(&self) -> Result<Package> {
        Package::from_index(self.raw_index()?)
    }

    pub fn submit(&self) -> Result<Option<Release>> {
        let package = self.package()?;

        match self.version_req {
            Some(ref version_req) => Ok(package.into_version(version_req)),
            None => Ok(package.into_latest()),
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

    pub fn releases(&self) -> &Vec<Release> {
        &self.releases
    }

    pub fn into_latest(mut self) -> Option<Release> {
        self.releases.pop()
    }

    pub fn latest(&self) -> Option<&Release> {
        self.releases.last()
    }

    pub fn into_version(self, version_req: &semver::VersionReq) -> Option<Release> {
        self.releases
            .into_iter()
            .rev()
            .find(|release| version_req.matches(&release.vers))
    }

    pub fn version(&self, version_req: &semver::VersionReq) -> Option<&Release> {
        self.releases
            .iter()
            .rev()
            .find(|release| version_req.matches(&release.vers))
    }

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

const fn one() -> u32 {
    1
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

pub type Features = BTreeMap<String, Vec<String>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub req: VersionReq,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: Option<String>,
    pub registry: Option<String>,
    pub package: Option<String>,
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
