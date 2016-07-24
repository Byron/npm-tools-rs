use std::path::Path;
use std::path::PathBuf;
use serde_json::{self, Value, from_reader, Map};
use quick_error::ResultExt;
use std::ffi::OsStr;

use std;
use std::fs;
use std::io;

struct ReadPackageFile<'a>(&'a Path);
struct DecodePackageFile<'a>(&'a Path);

quick_error!{
    #[derive(Debug)]
    pub enum Error {
        ReadPackageFile(p: PathBuf, err: io::Error) {
            description("The package.json could not be opened for reading")
            display("Failed to open '{}'", p.display())
            context(p: ReadPackageFile<'a>, err: io::Error) -> (p.0.to_path_buf(), err)
            cause(err)
        }
        JsonStructure(package_json_dir: PathBuf, expectation: String) {
            description("The data structure within package.json was unexpected")
            display("Unexpected Json Strucut in {}/package.json: {}", package_json_dir.display(), expectation)
        }
        DecodeJson(p: PathBuf, err: serde_json::Error) {
            description("The package.json could not be parsed as JSON")
            display("Failed to parse '{}'", p.display())
            context(p: DecodePackageFile<'a>, err: serde_json::Error) -> (p.0.to_path_buf(), err)
            cause(err)
        }
    }
}

pub enum Instruction {
    /// Move the directory at `from_here` to the `to_here` location, and create a symbolic link
    /// located at `from_here` which points to `to_here`, via the pre-computed `symlink_destination`
    MoveAndSymlink {
        from_here: PathBuf,
        to_here: PathBuf,
        symlink_destination: PathBuf,
    },
}

pub trait Visitor {
    /// Called whenever the package identified by `package` cannot be processed. The exact
    /// problem is stated in `err`.
    fn package_preprocessing_failed(&mut self, package: &PackageInfo, err: &Error);
    fn change(&mut self, action: Instruction);
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    /// the directory containing the package.json
    pub directory: PathBuf,
    /// the root directory at which all other node_modules are found
    pub root_directory: PathBuf,
}

impl PackageInfo {
    pub fn name(&self) -> &OsStr {
        self.directory.file_name().unwrap()
    }
}

pub fn deduplicate_into<'a, P, I, V>(repo: P, items: I, visitor: &mut V) -> Result<(), Vec<Error>>
    where P: AsRef<Path>,
          I: IntoIterator<Item = &'a PackageInfo>,
          V: Visitor
{
    fn read_package_json(p: &PackageInfo) -> std::result::Result<Map<String, Value>, Error> {
        let pjp = p.directory.join("package.json");
        let rd = try!(fs::File::open(&pjp).context(ReadPackageFile(&pjp)));
        match try!(from_reader(rd).context(DecodePackageFile((&pjp)))) {
            Value::Object(val) => Ok(val),
            _ => {
                Err(Error::JsonStructure(p.directory.clone(),
                                         String::from("Top level was not an object")))
            }
        }
    }

    fn handle_package(repo: &Path, p: &PackageInfo, errors: &mut Vec<Error>, visitor: &mut Visitor) {
        match read_package_json(p).and_then(|pj| {
            pj.get("version")
                .and_then(|v| match v.to_owned() {
                    Value::String(v) => Some(v),
                    _ => None,
                })
                .map(|v| (pj, v))
                .ok_or_else(|| {
                    Error::JsonStructure(p.directory.clone(),
                                         String::from("'version' key was not present, or its value was not a string"))
                })
        }) {
            Ok((pj, version)) => {
                for dep_key in &["dependencies", "devDependencies"] {
                    if let Some(deps) = pj.get(*dep_key) {
                        match deps.as_object().ok_or_else(|| {
                            Error::JsonStructure(p.directory.clone(),
                                                 format!("Key {} was not an object", dep_key))
                        }) {
                            Ok(deps) => {
                                for (dep_name, dep_version) in deps.iter() {
                                    let sub_modules_path = p.directory.join("node_modules").join(dep_name);
                                    let root_submodule_path = p.root_directory.join(dep_name);
                                    for potential_root in &[sub_modules_path, root_submodule_path] {
                                        if potential_root.is_dir() {
                                            let sub_package = PackageInfo {
                                                directory: potential_root.clone(),
                                                root_directory: p.root_directory.clone(),
                                            };
                                            handle_package(repo, &sub_package, errors, visitor);
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(err) => errors.push(err),
                        }
                    }
                }
                visitor.change(Instruction::MoveAndSymlink {
                    from_here: p.directory.clone(),
                    to_here: repo.join(p.name()),
                    symlink_destination: PathBuf::from("tobedone"),
                });
            }
            Err(err) => {
                visitor.package_preprocessing_failed(p, &err);
                errors.push(err);
            }
        }
    }

    let mut errors = Vec::new();
    for p in items {
        handle_package(repo.as_ref(), p, &mut errors, visitor);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
