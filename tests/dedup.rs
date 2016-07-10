extern crate hamcrest;
extern crate tempdir;
extern crate npm_tools;
extern crate fs_utils;

mod util;

use npm_tools::{ deduplicate_into, Visitor, PackageInfo };
use hamcrest::*;
use tempdir::TempDir;

#[derive(Default)]
struct Collector {
    preprocessed_packages: Vec<PackageInfo>
}

impl Visitor for Collector {
    fn package_preprocessing_failed(&mut self, package: &PackageInfo, _: &npm_tools::Error) {
        self.preprocessed_packages.push(package.clone());
    }
}

fn setup(root: &str) -> (TempDir, Collector, util::PackageMaker) {
    (util::transient_repo_path(),
     Collector::default(),
     util::PackageMaker::new(root))
}

#[test]
fn it_can_tell_the_visitor_to_symlink_a_direct_dependency_to_repo_if_version_does_not_exist() {
    let (repo, mut cl, make) = setup("reveal.js-unnested");

    let r = deduplicate_into(repo.path(), &[make.package_at("sigmund")], &mut cl);
    assert_that(r.unwrap(), equal_to(()));
}

#[test]
#[ignore]
fn it_indicates_error_if_the_visitor_has_at_least_one_failure() {
    unimplemented!();
}

#[test]
fn it_informs_the_visitor_right_after_something_went_wrong() {
    let (repo, mut cl, make) = setup("reveal.js-unnested");

    let ve = deduplicate_into(repo.path(), &[make.package_at("is-not-there")], &mut cl).err().unwrap();
    assert_that(&ve, of_len(1));
    assert_eq!(cl.preprocessed_packages[0].directory.file_name().unwrap(), "is-not-there");
}
