use crate::{get_index_path, Package, Query};
use std::path::PathBuf;

fn read_test_file(path: &str) -> String {
    let path = PathBuf::from(file!())
        .parent()
        .expect("test file parent")
        .join("data")
        .join(path);

    std::fs::read_to_string(path).expect("read data file")
}

#[test]
fn test_get_index_path_1() {
    assert_eq!(get_index_path("a"), "1/a");
}

#[test]
fn test_get_index_path_2() {
    assert_eq!(get_index_path("ab"), "2/ab");
}

#[test]
fn test_get_index_path_3() {
    assert_eq!(get_index_path("abc"), "3/a/abc");
}

#[test]
fn test_get_index_path_4() {
    assert_eq!(get_index_path("abcd"), "ab/cd/abcd");
}

#[test]
fn test_get_index_path_long() {
    assert_eq!(get_index_path("abcdefgh"), "ab/cd/abcdefgh");
}

#[test]
fn test_get_index_path_caps() {
    assert_eq!(get_index_path("AbcDefGH"), "ab/cd/abcdefgh");
}

#[test]
fn make_query_no_version() {
    let query: Query = "cargo".parse().expect("parse query");
    assert_eq!(query.name, "cargo");
    assert!(query.version_req.is_none());
}

#[test]
fn make_query_with_version() {
    let query: Query = "cargo@0.12".parse().expect("parse query");
    assert_eq!(query.name, "cargo");
    assert_eq!(
        query.version_req,
        Some("0.12".parse().expect("version req"))
    );
}

#[test]
fn test_get_specific_release() {
    let data = read_test_file("libc.index");
    let pkg = Package::from_index(data).expect("package from index");

    assert_eq!(pkg.name(), "libc");
    assert_eq!(pkg.index_path(), "li/bc/libc");
    assert!(
        pkg.version(&"=0.1.11".parse().expect("semver"))
            .expect("matching libc version")
            .yanked
    );
}

#[test]
fn test_get_latest_matching_release() {
    let data = read_test_file("libc.index");
    let pkg = Package::from_index(data).expect("package from index");

    assert_eq!(pkg.name(), "libc");
    assert_eq!(pkg.index_path(), "li/bc/libc");
    assert_eq!(
        pkg.version(&"0.1.0".parse().expect("semver"))
            .expect("release")
            .vers,
        "0.1.12".parse().expect("version")
    );
}
