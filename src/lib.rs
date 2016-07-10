//! Required features
//!
//! * depth-first traversal of all node-modules contents to find package-json
//!   while ignoring sylinks
//! * move non-root module into the npm-repo and symlink it's original location
//!   with a relative or absolute path to the npm-repo.
//! * npm-repo structure follows '<package-name>/<version>'
//!   - Optionally allow to insert the <platform> in case the repository is
//!     shared by multiple platforms
//! * read package.json to determine all required (dev)dependencies.
//!   Creation of these ones must be deferred until we know the best suitable
//!   package version is already in the repository.
//! * re-evaluate all symlinks within the npm-repo and update them to the best
//!   suitable version.
//!   As new packages and/or versions are added, this might change. Can be
//!   based on packages within
//!   a project's node_modules dir, or on all the ones in the npm-repo.
//! * collect and remove packages in the npm-repo which are not used anymore.
//!   - This should be based on a list of repos which use it ... this can at
//!   least help to efficiently optimize the repo.
//! * Revert all changes to the node_modules directory to allow npm to operate
//!   naturally on it.
//! - **IMPORTANT**: figure out whether it will mess with an existing setup -
//! it could
//!   very well be that
//!   it updates packages in place, which are actually living in our
//!   npm-repo, and thus
//!   messes with the versions. If that's possible, one would need a
//!   **sanity check/fix** for the repo as well.
extern crate serde_json;
#[macro_use]
extern crate quick_error;

mod dedup;

pub use dedup::*;
