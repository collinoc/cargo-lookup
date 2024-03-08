#![deny(clippy::all)]

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::BTreeMap;
mod cli;

use cargo_query::Query;

fn main() -> Result<()> {
    let args = cli::Args::parse();
    let packages = args.packages;

    let mut results = Vec::with_capacity(packages.len());

    for package in packages {
        let query: Query = package.parse()?;
        let release = query
            .submit()?
            .with_context(|| format!("package `{package}` not found"))?;

        results.push((package, release));
    }

    if args.json {
        let map = BTreeMap::from_iter(results);
        let json = if args.pretty {
            serde_json::to_string_pretty(&map)?
        } else {
            serde_json::to_string(&map)?
        };

        println!("{json}");
    } else {
        for (package, release) in results {
            if args.features {
                let feature_string = release
                    .features
                    .keys()
                    .fold(String::new(), |list, feature| list + " " + feature);
                let feature_string = feature_string.trim();

                println!("{package}:{feature_string}");
            } else if args.deps {
                let deps_string = release
                    .deps
                    .into_iter()
                    .map(|dep| dep.name)
                    .collect::<Vec<String>>()
                    .join(" ");

                println!("{package}:{deps_string}");
            } else {
                let json = release.as_json_string()?;
                println!("{package}:{json}");
            }
        }
    }

    Ok(())
}
