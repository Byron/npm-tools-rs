use std::path::Path;
use std::path::PathBuf;
use serde_json::{self, Value, from_reader};
use quick_error::ResultExt;

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
        DecodeJson(p: PathBuf, err: serde_json::Error) {
            description("The package.json could not be parsed as JSON")
            display("Failed to parse '{}'", p.display())
            context(p: DecodePackageFile<'a>, err: serde_json::Error) -> (p.0.to_path_buf(), err)
            cause(err)
        }
    }
}

pub type Result = std::result::Result<(), Vec<Error>>;

pub trait Visitor {

}

pub struct PackageInfo {
    /// the directory containing the package.json
    pub directory: PathBuf,
    /// the root directory at which all other node_modules are found
    pub root_directory: PathBuf
}

pub fn deduplicate_into<'a, P, I, V>(repo: P, items: I, visitor: &mut V) -> Result
    where P: AsRef<Path>,
          I: IntoIterator<Item = &'a PackageInfo>,
          V: Visitor
{
    fn read_package_json(p: &PackageInfo) -> std::result::Result<Value, Error> {
        let pjp = p.directory.join("package.json");
        let pj: Value = {
            let rd = try!(fs::File::open(&pjp).context(ReadPackageFile(&pjp)));
            try!(from_reader(rd).context(DecodePackageFile((&pjp))))
        };
        Ok(pj)
    }

    let mut errors = Vec::new();
    for p in items {
        match read_package_json(p) {
            Ok(pj) => {}
            Err(err) => {
                errors.push(err);
            }
        }
    }

    if (errors.is_empty()) {
        Ok(())
    } else {
        Err(errors)
    }
}
