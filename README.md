# Cargo Query

A command-line utility for querying Rust crate registry indexes

[![github]](https://github.com/collinoc/cargo-query)

[github]: https://img.shields.io/badge/github-blue?style=for-the-badge&logo=github&link=https%3A%2F%2Fgithub.com%2Fcollinoc%2Fcargo-query

Cargo Query allows you to query [crate indexes](https://crates.io/data-access#crate-index) for info about different crates, including features and dependencies. You can also specify [semantic versioning requirements](https://doc.rust-lang.org/cargo/reference/resolver.html#semver-compatibility) to query info for a specific release of a crate!

# Installation

The quickest way to install Cargo Query is via [`Cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html)

```console
$ cargo install cargo-query
```

# Usage

Below are some common usage examples that you might find use of. Run `cargo query --help` to see what else it can do!

### List the features of a package

```console
$ cargo query libc --type=features
libc:align const-extern-fn default extra_traits rustc-dep-of-std std use_std
```

### Add all features for a package to your project

```console
$ cargo add syn --features=$(cargo query syn --format=cargo-add-all)
Updating crates.io index
    Adding syn v2.0.52 to dependencies.
            Features:
            + clone-impls
            + derive
            + extra-traits
            + fold
            + full
            + parsing
            + printing
            + proc-macro
            + quote
            + test
            + visit
            + visit-mut
Updating crates.io index
```

### List the dependencies of packages

```console
$ cargo query serde libc --type=deps
libc:rustc-std-workspace-core
serde:serde_derive serde_derive serde_derive
```

### List package info in pretty printed JSON

```console
$ cargo query semver --type=json --format=pretty
[
  {
    "name": "semver",
    "vers": "1.0.22",
    "deps": [
      {
        "name": "serde",
        "req": "^1.0.194",
        "features": [],
        "optional": true,
        "default_features": false,
        "target": null,
        "kind": "normal",
        "registry": null,
        "package": null
      }
    ],
    "cksum": "92d43fe69e652f3df9bdc2b85b2854a0825b86e4fb76bc44d945137d053639ca",
    "features": {
      "default": [
        "std"
      ],
      "std": []
    },
    "yanked": false,
    "links": null,
    "v": 1,
    "features2": null,
    "rust_version": "^1.31"
  }
]
```
