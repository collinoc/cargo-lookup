#![deny(clippy::all)]

use anyhow::{Context, Result};
use clap::Parser;
mod cli;

use cargo_query::Query;

fn main() -> Result<()> {
    let args = cli::Args::parse();
    let packages = args.packages;

    for package in packages {
        let query = Query::parse(&package)?;
        let release = query
            .submit()?
            .with_context(|| format!("package `{package}` not found"))?
            .as_json_string()?;

        println!("{release}");
    }

    Ok(())
}
