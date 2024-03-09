#[derive(Debug)]
pub enum Error {
    InvalidVersion(semver::Error),
    Request(Box<ureq::Error>),
    Io(std::io::Error),
    Serialize(serde_json::Error),
    Deserialize(serde_json::Error),
    FromIndexFile(&'static str),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidVersion(error) => write!(f, "failed parsing version: {error}"),
            Error::Request(error) => write!(f, "request failed: {error}"),
            Error::Io(error) => write!(f, "IO error: {error}"),
            Error::Serialize(error) => write!(f, "failed to serialize: {error}"),
            Error::Deserialize(error) => write!(f, "failed to deserialize: {error}"),
            Error::FromIndexFile(error) => write!(f, "failed to populate from index file: {error}"),
        }
    }
}
