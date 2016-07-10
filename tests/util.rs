use std::path::{Path, PathBuf};
use tempdir::TempDir;

pub fn fixture_at(path: &str) -> PathBuf {
    Path::new(file!()).parent().unwrap().join("fixtures").join(path)
}

pub fn transient_repo_path() -> TempDir {
    TempDir::new("npm_repo_path").unwrap()
}
