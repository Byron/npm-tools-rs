extern crate hamcrest;
extern crate tempdir;
extern crate npm_tools;
extern crate fs_utils;

mod util;

use npm_tools::{ deduplicate_into, Visitor };
use hamcrest::*;

struct Collector {}

impl Visitor for Collector {

}

#[test]
fn it_can_tell_the_visitor_to_symlink_a_direct_dependency_to_repo_if_version_does_not_exist() {
    let repo = util::transient_repo_path();
    let mut cl = Collector {};
    let make = util::PackageMaker::new("reveal.js-unnested");

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

}
