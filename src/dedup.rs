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
          I: IntoIterator<Item = &'a PackageInfo<'a>>,
          V: Visitor
{

}
