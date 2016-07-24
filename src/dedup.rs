use std::path::Path;
use std::path::PathBuf;
use serde_json::{self, Value, from_reader, Map};
use quick_error::ResultExt;
use std::ffi::OsStr;
use std::collections::btree_map::BTreeMap;
use std::collections::hash_set::HashSet;
use semver::{VersionReq, Version, SemVerError, ReqParseError};

use std;
use std::fs;
use std::io;

struct ReadPackageFile<'a>(&'a Path);
struct DecodePackageFile<'a>(&'a Path);
struct PathAndVersion<'a>(&'a Path, &'a str);

quick_error!{
    #[derive(Debug)]
    pub enum Error {
        ReadPackageFile(p: PathBuf, err: io::Error) {
            description("The package.json could not be opened for reading")
            display("Failed to open '{}'", p.display())
            context(p: ReadPackageFile<'a>, err: io::Error) -> (p.0.to_path_buf(), err)
            cause(err)
        }
        InvalidVersionRequirement(package_json_dir: PathBuf, version_req: String, err: ReqParseError) {
            description("A semantic version requirement could not be parsed")
            display("Unexpected version requirement string '{}' in {}/package.json: {}", version_req, package_json_dir.display(), err)
            context(a: PathAndVersion<'a>, err: ReqParseError) -> (a.0.to_path_buf(), a.1.to_owned(), err)
            cause(err)
        }
        InvalidVersion(package_json_dir: PathBuf, version: String, err: SemVerError) {
            description("A semantic version could not be parsed")
            display("Unexpected version string '{}' in {}/package.json: {}", version, package_json_dir.display(), err)
            context(a: PathAndVersion<'a>, err: SemVerError) -> (a.0.to_path_buf(), a.1.to_owned(), err)
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

/// Something to be done.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// Called with an instruction on what to do next. Must never panic, and is expected to keep
    /// all error handling internal.
    fn change(&mut self, action: Instruction);
}

#[derive(Ord, Eq, PartialEq, PartialOrd)]
struct PackageKey {
    name: String,
    version: Version,
}

#[derive(Hash, Eq, PartialEq)]
struct PackageDependency {
    name: String,
    version_req: String,
}

struct PackageDependencies {
    package_info: PackageInfo,
    deps: HashSet<PackageDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Iterate `items` and read all package.json files contained therein to collect enough information
/// to compute all changes required to sym-link or update the respective packages in `repo`.
/// `visitor` will be called whenever something goes wrong, or whenever there is something to do.
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

    fn fetch_string(m: &Map<String, Value>, p: &PackageInfo, field_name: &str) -> Result<String, Error> {
        m.get(field_name)
            .and_then(|v| match v {
                &Value::String(ref v) => Some(v.to_owned()),
                _ => None,
            })
            .ok_or_else(|| {
                Error::JsonStructure(p.directory.clone(),
                                     format!("'{}' key was not present, or its value was not a string",
                                             field_name))
            })
    }

    fn handle_package(repo: &Path, p: &PackageInfo, errors: &mut Vec<Error>, deps: &mut BTreeMap<PackageKey, PackageDependencies>, visitor: &mut Visitor) {
        match read_package_json(p).and_then(|pj| {
            fetch_string(&pj, p, "version")
                .and_then(|v| fetch_string(&pj, p, "name").map(|n| (v, n)))
                .and_then(|(v, n)| Version::parse(&v).context(PathAndVersion(&p.directory, &v)).map_err(|e| e.into()).map(|sv| (pj, sv, v, n)))
        }) {
            Ok((pj, semantic_version, version, name)) => {
                let mut dep_info = deps.entry(PackageKey {
                        name: name,
                        version: semantic_version,
                    })
                    .or_insert_with(|| {
                        PackageDependencies {
                            package_info: p.clone(),
                            deps: Default::default(),
                        }
                    });
                for dep_key in &["dependencies", "devDependencies"] {
                    if let Some(deps) = pj.get(*dep_key) {
                        match deps.as_object().ok_or_else(|| {
                            Error::JsonStructure(p.directory.to_owned(),
                                                 format!("Key {} was not an object", dep_key))
                        }) {
                            Ok(deps) => {
                                for (dep_name, dep_version) in deps.iter() {
                                    let normalized_req = match dep_version.as_string()
                                        .ok_or_else(|| {
                                            Error::JsonStructure(p.directory.clone(),
                                                                 String::from("version of dependency was not a string"))
                                        })
                                        .and_then(|v| VersionReq::parse(v).context(PathAndVersion(&p.directory, v)).map_err(|err| err.into())) {
                                        Ok(vr) => vr,
                                        Err(err) => {
                                            errors.push(err);
                                            continue;
                                        }
                                    };
                                    dep_info.deps.insert(PackageDependency {
                                        name: dep_name.to_owned(),
                                        version_req: format!("{}", normalized_req),
                                    });
                                }
                            }
                            Err(err) => errors.push(err),
                        }
                    }
                }
                let destination = repo.join(p.name()).join(version);
                visitor.change(Instruction::MoveAndSymlink {
                    from_here: p.directory.clone(),
                    to_here: destination.clone(),
                    symlink_destination: destination,
                });
            }
            Err(err) => {
                visitor.package_preprocessing_failed(p, &err);
                errors.push(err);
            }
        }
    }

    let mut errors = Vec::new();
    let mut deps = BTreeMap::new();
    for p in items {
        handle_package(repo.as_ref(), p, &mut errors, &mut deps, visitor);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
