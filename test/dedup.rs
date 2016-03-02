extern crate hamcrest;
extern crate npm_tools;

use npm_tools::*;
use hamcrest::*;


#[test]
fn it_cannot_deduplicate_versions_which_are_different() {
    #[derive(Default)]
    struct MockVisitor {
        put_calls: u32,
    }
    
    impl Visitor for MockVisitor {
        
    }
    
    let mut mock = MockVisitor::default();
    deduplicate_into("repo", [("foo", "0.1.0"),
                              ("bar", "0.2.0")].iter(), &mut mock);
                               
    assert_that(mock.put_calls, equal_to(2));
}
