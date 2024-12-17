#![deny(clippy::all)]

use anyhow::{bail, Result};
use cargo_lookup::{Query, Release};
use clap::Parser;
use reqwest::blocking::Client;
use std::collections::{HashSet, VecDeque};
use std::ops::Deref;
use std::sync::Arc;
use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::Duration;

mod cli;
use cli::{Cli, Format, Options, Type};

#[derive(Debug, Default)]
struct ResolutionContext {
    resolved: Vec<Release>,
    pending: HashSet<String>,
}

type Context = Arc<Mutex<ResolutionContext>>;

#[tokio::main]
async fn main() -> Result<()> {
    let Cli::Lookup(options) = Cli::parse();
    let packages = options.packages.clone();
    let options = Arc::new(options);

    let context = Context::default();
    let resolve_depth = options
        .max_depth
        .map(Depth::Restricted)
        .unwrap_or(Depth::Infinite);

    let client = Arc::new(Client::new());
    let mut pending = VecDeque::new();

    for package in packages.into_iter() {
        let query: Query = package.parse()?;
        pending.push_back((resolve_depth, query));
    }

    while let Some((depth, query)) = pending.pop_front() {
        let next_depth = match depth {
            Depth::Infinite => Depth::Infinite,
            Depth::Restricted(0) => break,
            Depth::Restricted(depth) => Depth::Restricted(depth - 1),
        };

        let mut context = context.lock();

        if let Some(release) = resolve(query, Arc::clone(&client), Arc::clone(&options))? {
            let deps = release.deps.clone();

            // Try taking this release, quickly resolving all of it's dependencies via the threadpool,
            // and then for each nested dep, shove that to the pending deque along with the depth

            for dep in deps {
                let name = dep.package.unwrap_or(dep.name);
                let req = dep.req;

                pending.push_back((next_depth, Query::new_req(name, req)));
            }

            context.resolved.push(release);
        }
    }

    let resolved = &context.lock().resolved;

    if options.count {
        println!("{}", resolved.len());
    } else if options.kind == Some(Type::Json) {
        // Print all resolved items in one JSON list
        let json = if options.format == Format::Pretty {
            serde_json::to_string_pretty(resolved)?
        } else {
            serde_json::to_string(resolved)?
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

#[derive(Debug, PartialEq, Clone, Copy)]
enum Depth {
    Infinite,
    Restricted(usize),
}
