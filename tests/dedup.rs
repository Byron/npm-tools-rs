extern crate hamcrest;
extern crate tempdir;
extern crate npm_tools;
extern crate fs_utils;

mod utils;

use std::path::PathBuf;
use npm_tools::{deduplicate_into, Visitor, PackageInfo, Instruction};
use hamcrest::*;
use tempdir::TempDir;

#[derive(Default)]
struct Collector {
    preprocessed_packages: Vec<PackageInfo>,
    instructions: Vec<Instruction>,
}

impl Visitor for Collector {
    fn package_preprocessing_failed(&mut self, package: &PackageInfo, _: &npm_tools::Error) {
        self.preprocessed_packages.push(package.clone());
    }

    fn change(&mut self, action: Instruction) {
        self.instructions.push(action);
    }
}

fn setup(root: &str) -> (TempDir, Collector, utils::PackageMaker) {
    (utils::transient_repo_path(), Collector::default(), utils::PackageMaker::new(root))
}

#[test]
fn it_can_tell_the_visitor_to_symlink_a_direct_dependency_to_repo_if_version_does_not_exist() {
    let (repo, mut cl, make) = setup("reveal.js-unnested");

    let ps = [make.package_at("sigmund")];
    let r = deduplicate_into(repo.path(), &ps, &mut cl);
    assert_that(r.unwrap(), equal_to(()));
    assert_that(&cl.instructions, of_len(1));

    let ref op = cl.instructions[0];
    match op {
        &Instruction::MoveAndSymlink {ref from_here, ref to_here, ref symlink_destination} => {
            assert_that(from_here, equal_to(&ps[0].directory));
            let expected_to_here = repo.path().join("sigmund").join("1.0.1");
            assert_that(to_here, equal_to(&expected_to_here));
            assert_that(symlink_destination, equal_to(&expected_to_here));
        }
    }
}

#[test]
fn a_package_can_produce_its_name() {
   let p = PackageInfo {
       directory: PathBuf::from("some/path/package-name"),
       root_directory: PathBuf::new()
   };

    assert_that(p.name(), equal_to("package-name".as_ref()));
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
    assert_eq!(cl.preprocessed_packages[0].directory.file_name().unwrap(),
               "is-not-there");
}
