extern crate hamcrest;
extern crate tempdir;
extern crate npm_tools;
extern crate fs_utils;
#[macro_use]
extern crate quick_error;

mod utils;

use std::path::PathBuf;
use npm_tools::{deduplicate_into, Visitor, PackageInfo, InstructionOwned, Instruction, Error};
use hamcrest::*;
use tempdir::TempDir;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::os::unix::fs::symlink;

#[derive(Default)]
struct Collector {
    preprocessed_packages: Vec<PackageInfo>,
    instructions: Vec<InstructionOwned>,
    fail_on_change: bool,
}

quick_error!{
    #[derive(Debug)]
    pub enum FakeError {
        Action(action: InstructionOwned) { }
    }
}

impl Visitor for Collector {
    type Error = FakeError;

    fn error(&mut self, package: &PackageInfo, _: &npm_tools::Error) {
        self.preprocessed_packages.push(package.clone());
    }

    fn change(&mut self, action: Instruction) -> Result<(), Self::Error> {
        if self.fail_on_change {
            Err(FakeError::Action(action.into()))
        } else {
            self.instructions.push(action.into());
            Ok(())
        }
    }
}

fn setup(root: &str) -> (TempDir, Collector, utils::PackageMaker) {
    (utils::transient_repo_path(), Collector::default(), utils::PackageMaker::new(root))
}

#[test]
fn it_does_not_tell_visitor_to_symlink_a_direct_dependency_to_repo_if_it_is_a_symlink_to_correct_destination_already
    () {
    let repo = utils::transient_repo_path();
    let repo_sigmund_destination = repo.path().join("sigmund").join("1.0.1");
    create_dir_all(&repo_sigmund_destination).unwrap();
    File::create(repo_sigmund_destination.join("package.json"))
        .unwrap()
        .write_all(r#"{"version":"1.0.1", "name":"sigmund"}"#.as_ref())
        .unwrap();

    let package_source = TempDir::new("package").unwrap();
    let package_sigmund_dir = package_source.path().join("node_modules");

    create_dir_all(&package_sigmund_dir).unwrap();
    symlink(&repo_sigmund_destination,
            &package_sigmund_dir.join("sigmund"))
        .unwrap();

    let make = utils::PackageMaker::new(package_source.path().to_str().unwrap());

    let ps = [make.package_at("sigmund")];
    let mut cl = Collector::default();
    let r = deduplicate_into(repo.path(), &ps, &mut cl);

    assert_that(r.unwrap(), equal_to(()));
    assert_that(&cl.instructions, of_len(0));
}

#[test]
fn it_tells_visitor_to_symlink_a_direct_dependency_to_repo_if_version_does_exist_there() {
    let (repo, mut cl, make) = setup("reveal.js-unnested");
    let abs_destination = repo.path().join("sigmund").join("1.0.1");
    create_dir_all(&abs_destination).unwrap();

    let ps = [make.package_at("sigmund")];
    let r = deduplicate_into(repo.path(), &ps, &mut cl);
    assert_that(r.unwrap(), equal_to(()));
    assert_that(&cl.instructions, of_len(1));

    match cl.instructions[0] {
        InstructionOwned::ReplaceWithSymlink { ref this_directory, ref symlink_destination } => {
            assert_that(this_directory, equal_to(&ps[0].directory));
            assert_that(symlink_destination, equal_to(&abs_destination));
        }
        _ => unreachable!(),
    }
}

#[test]
fn it_tells_visitor_to_move_and_symlink_a_direct_dependency_to_repo_if_version_does_not_exist() {
    let (repo, mut cl, make) = setup("reveal.js-unnested");

    let ps = [make.package_at("sigmund")];
    let r = deduplicate_into(repo.path(), &ps, &mut cl);
    assert_that(r.unwrap(), equal_to(()));
    assert_that(&cl.instructions, of_len(1));

    match cl.instructions[0] {
        InstructionOwned::MoveAndSymlink { ref from_here, ref to_here, ref symlink_destination } => {
            assert_that(from_here, equal_to(&ps[0].directory));
            let expected_to_here = repo.path().join("sigmund").join("1.0.1");
            assert_that(to_here, equal_to(&expected_to_here));
            assert_that(symlink_destination, equal_to(&expected_to_here));
        }
        _ => unreachable!(),
    }
}

#[test]
fn a_package_can_produce_its_name() {
    let p = PackageInfo {
        directory: PathBuf::from("some/path/package-name"),
        root_directory: PathBuf::new(),
    };

    assert_that(p.name(), equal_to("package-name".as_ref()));
}

#[test]
fn it_indicates_error_if_the_visitor_has_at_least_one_failure() {
    let (repo, mut cl, make) = setup("reveal.js-unnested");
    cl.fail_on_change = true;

    let ve = deduplicate_into(repo.path(), &[make.package_at("sigmund")], &mut cl).err().unwrap();
    assert_that(&ve, of_len(1))
}

#[test]
fn it_informs_the_visitor_right_after_something_went_wrong() {
    let (repo, mut cl, make) = setup("reveal.js-unnested");

    let ve = deduplicate_into(repo.path(), &[make.package_at("is-not-there")], &mut cl).err().unwrap();
    assert_that(&ve, of_len(1));
    assert_eq!(cl.preprocessed_packages[0].directory.file_name().unwrap(),
               "is-not-there");
}

/// Even though duplicate packages can be returned, those should always be on different spots in
/// the hierarchy (i.e. the actual directories are different from each other, like `a` and `c/a`).
#[test]
fn it_rejects_duplicate_packages() {
    let (repo, mut cl, make) = setup("reveal.js-unnested");

    let p = make.package_at("sigmund");
    let ps = [p.clone(), p];
    let ve = deduplicate_into(repo.path(), &ps, &mut cl).err().unwrap();
    assert_that(&ve, of_len(1));
    match ve[0] {
        Error::DuplicatePackageInformation(ref pd) => assert_that(&ps[1], equal_to(pd)),
        _ => assert!(false),
    }
}
