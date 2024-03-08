use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    /// Packages to query
    pub(crate) packages: Vec<String>,
    /// Show dependencies for each package
    #[clap(short, long, conflicts_with = "features")]
    pub(crate) deps: bool,
    /// Show features for each package
    #[clap(short, long, conflicts_with = "deps")]
    pub(crate) features: bool,
    /// Print output in json format
    #[clap(short, long)]
    pub(crate) json: bool,
    /// Pretty print output
    #[clap(short, long)]
    pub(crate) pretty: bool,
}
