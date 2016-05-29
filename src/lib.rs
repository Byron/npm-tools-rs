//! Required features
//! * depth-first traversal of all node-modules contents to find package-json while ignoring sylinks
//! * move non-root module into the npm-repo and symlink it's original location with a relative
//!   path to the npm-repo.
//! * npm-repo structure follows '<package-name>/<version>'
//! * read package.json to determine all required (dev)dependencies.
//!   Creation of these ones must be deferred until we know the best suitable package version 
//!   is already in the repository.
//! * re-evaluate all symlinks within the npm-repo and update them to the best suitable version.
//!   As new packages and/or versions are added, this might change. Can be based on packages within
//!   a project's node_modules dir, or on all the ones in the npm-repo.
//! * collect and remove packages in the npm-repo which are not used anymore.
//! * Revert all changes to the node_modules directory to allow npm to operate naturally on it.
//!  - TODO: figure out whether it will mess with an existing setup - it could very well be that 
//!          it updates packages in place, which are actually living in our npm-repo, and thus 
//!          messes with the versions. If that's possible, one would need a sanity check/fix for
//!          the repo as well.
use std::path::Path;

pub trait Visitor {
    
}

struct PackageInfo<'a> {
    /// the package name
    name: &'a str,
    /// the directory containing the package.json
    directory: &'a Path,
    /// the root directory at which all other node_modules are found
    root_directory: &'a Path,
}

pub fn deduplicate_into<'a, P, I, V>(repo: P, items: I, visitor: &mut V) 
    where P: AsRef<Path>,
          I: IntoIterator<Item=&'a PackageInfo<'a>>,
          V: Visitor {
    
}


