use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[clap(name = "cargo", bin_name = "cargo")]
pub enum Cli {
    #[clap(version, about, name = "lookup")]
    Lookup(Options),
}

#[derive(Debug, Parser)]
pub struct Options {
    /// Packages to query
    pub(crate) packages: Vec<String>,
    /// Output type
    #[clap(short = 't', long = "type", value_name = "TYPE")]
    pub(crate) kind: Option<Type>,
    /// Output format
    #[clap(short, long, default_value = "default")]
    pub(crate) format: Format,
    /// Use a custom crate index URL
    #[clap(short, long)]
    pub(crate) index_url: Option<String>,
    /// Careful, this may take a while!
    /// Display info on queried package dependencies that are recursively resolved
    #[clap(short, long, verbatim_doc_comment)]
    pub(crate) recursive: bool,
    /// Maximum depth when recursively querying dependencies
    #[clap(short, long)]
    pub(crate) max_depth: Option<usize>,
    /// Delimiter when printing features or dependencies
    #[clap(short, long, default_value = " ")]
    pub(crate) delim: String,
    /// Ignore missing packages
    #[clap(short = 'g', long)]
    pub(crate) ignore_missing: bool,
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
pub enum Type {
    /// Show dependencies for each package
    Deps,
    /// Show features for each package
    Features,
    /// Print output in JSON format
    Json,
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
pub enum Format {
    /// Default format type: space separated with package name prefix
    Default,
    /// Pretty print output
    Pretty,
    /// No package name prefix when printing depenencies or features
    NoPrefix,
    /// A format for quickly adding all features to a cargo package
    ///
    /// Equivalent to passing `--type=features --format=no-prefix --delim=,`
    CargoAddAll,
}
