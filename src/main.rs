#![deny(clippy::all)]

use anyhow::{anyhow, bail, Result};
use cargo_lookup::{Query, Release};
use clap::Parser;
use std::ops::Deref;

mod cli;

use cli::{Cli, Format, Options, Type};

fn main() -> Result<()> {
    let Cli::Lookup(options) = Cli::parse();
    let packages = options.packages.as_slice();

    let mut resolved = Vec::new();
    let resolve_depth = options
        .max_depth
        .map(Depth::Restricted)
        .unwrap_or(Depth::Infinite);

    for package in packages {
        resolve(
            package,
            options.index_url.as_deref(),
            resolve_depth,
            &options,
            &mut resolved,
        )?;
    }

    if options.kind == Some(Type::Json) {
        // Print all resolved items in one JSON list
        let json = if options.format == Format::Pretty {
            serde_json::to_string_pretty(&resolved)?
        } else {
            serde_json::to_string(&resolved)?
        };

        println!("{json}");
    } else {
        for release in resolved {
            let use_prefix = !matches!(options.format, Format::CargoAddAll | Format::NoPrefix);
            let (kind, delim) = match options.format {
                Format::CargoAddAll => (Some(Type::Features).as_ref(), ","),
                _ => (options.kind.as_ref(), options.delim.as_str()),
            };

            let info_string = match kind {
                Some(Type::Features) => release
                    .features
                    .keys()
                    .map(Deref::deref)
                    .collect::<Vec<&str>>()
                    .join(delim),
                Some(Type::Deps) => release
                    .deps
                    .iter()
                    .map(|dep| dep.name.as_str())
                    .collect::<Vec<&str>>()
                    .join(delim),
                Some(Type::Json) | None => release.as_json_string()?,
            };

            if use_prefix {
                let package = &release.name;
                println!("{package}:{info_string}");
            } else {
                println!("{info_string}");
            }
        }
    }

    Ok(())
}

fn resolve(
    package: &str,
    index: Option<&str>,
    depth: Depth,
    options: &Options,
    resolved: &mut Vec<Release>,
) -> Result<()> {
    let query: Query = match index {
        Some(custom) => package.parse::<Query>()?.with_index(custom),
        None => package.parse()?,
    };

    let result = match query.submit() {
        Ok(Some(result)) => result,
        _ if options.ignore_missing => return Ok(()),
        Ok(None) => bail!("failed to find a matching release of `{package}`"),
        Err(other) => return Err(anyhow!(other)),
    };

    let deps = result.deps.clone();

    resolved.push(result);

    if options.recursive
        && (depth == Depth::Infinite || matches!(depth, Depth::Restricted(max) if max > 1))
    {
        let depth = match depth {
            Depth::Infinite => Depth::Infinite,
            Depth::Restricted(max) => Depth::Restricted(max - 1),
        };

        for sub in deps {
            let name = sub.package.as_deref().unwrap_or(sub.name.as_str());
            let version_req = sub.req;
            let sub_query = format!("{name}@{version_req}");

            // Stop cyclic dependencies from being infinitely resolved
            if resolved
                .iter()
                .any(|res| name == res.name && version_req.matches(&res.vers))
            {
                continue;
            }

            resolve(&sub_query, index, depth, options, resolved)?;
        }
    }

    Ok(())
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Depth {
    Infinite,
    Restricted(usize),
}
