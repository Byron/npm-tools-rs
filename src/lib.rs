use std::path::Path;

pub trait Visitor {
    
}

pub fn deduplicate_into<'a, P, I, S, V>(repo: P, items: I, visitor: &mut V) 
    where P: AsRef<Path>,
          S: AsRef<str>,
          V: Visitor,
          I: IntoIterator<Item=&'a(S, S)> {
    
}

