use std::path::{Path, PathBuf};
use npm_tools::PackageInfo;
use tempdir::TempDir;

pub fn fixture_at<P>(path: P) -> PathBuf
    where P: AsRef<Path>{
    Path::new(file!()).parent().unwrap().parent().unwrap().join("fixtures").join(path)
}

pub fn transient_repo_path() -> TempDir {
    TempDir::new("npm_repo_path").unwrap()
}

pub struct PackageMaker {
    root: PathBuf
}

impl PackageMaker {
    pub fn new(root: &str) -> PackageMaker {
        PackageMaker {
            root: PathBuf::from(root).join("node_modules")
        }
    }

    pub fn package_at(&self, sub_path: &str) -> PackageInfo {
        PackageInfo {
            root_directory: self.root.clone(),
            directory: fixture_at(&self.root).join(sub_path)
        }
    }
}
