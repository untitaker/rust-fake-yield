use fake_yield::fake_yield;
use itertools::Itertools;

#[test]
fn demo_case() {
    fn to_html_stream<'a>(xs: impl IntoIterator<Item = &'a str>) -> impl Iterator<Item = &'a str> {
        fake_yield! {{
            _yield!("<ul>\n");
            for x in xs {
                _yield!("<li>");
                _yield!(x);  // Don't try this at home
                _yield!("</li>\n");
            }
            _yield!("</ul>");
        }}
    }

    assert_eq!(
        to_html_stream(vec!["1", "2", "3"]).join(""),
        r#"<ul>
<li>1</li>
<li>2</li>
<li>3</li>
</ul>"#
    );
}

#[test]
fn basecase() {
    fn foo() -> impl Iterator<Item = usize> {
        fake_yield!()
    }

    assert_eq!(foo().next(), None);
}

#[test]
fn simple_print() {
    fn foo() -> impl Iterator<Item = usize> {
        fake_yield! {{
            println!("hi");
        }}
    }

    assert_eq!(foo().next(), None);
}

#[test]
fn basic() {
    fn foo() -> impl Iterator<Item = usize> {
        fake_yield!(_yield!(42usize))
    }

    assert_eq!(foo().next(), Some(42));
}

#[test]
fn basic2() {
    fn foo() -> impl Iterator<Item = usize> {
        fake_yield! {{
            println!("hi");
            _yield!(1usize);
            println!("ho");
            _yield!(2usize);
        }}
    }

    assert_eq!(foo().collect::<Vec<_>>(), vec![1, 2]);
}

#[test]
fn if_statements() {
    fn foo() -> impl Iterator<Item = usize> {
        fake_yield! {{
            if false {
                _yield!(1usize);
            }
            if true {
                _yield!(2usize);
            }

            if false {
                _yield!(99usize);
            } else {
                _yield!(3usize);
            }
        }}
    }

    assert_eq!(foo().collect::<Vec<_>>(), vec![2, 3]);
}

#[test]
fn if_statements_scoping() {
    #[allow(unused_variables)]
    fn foo() -> impl Iterator<Item = usize> {
        fake_yield! {{
            let x = 2usize;
            if (true) {
                let x = 1usize;
                _yield!(x);
            }

            _yield!(x);

            if(false) {
                let x = 99usize;
            }

            _yield!(x + 1);
        }}
    }

    assert_eq!(foo().collect::<Vec<_>>(), vec![1, 2, 3]);
}

#[test]
fn if_let_statements() {
    #[allow(unused_variables)]
    fn foo() -> impl Iterator<Item = usize> {
        fake_yield! {{
            let one = Some(1usize);

            if let Some(x) = one {
                _yield!(x);
            } else {
                _yield!(99usize);
            }

            if let Some(x) = (None::<usize>) {
                _yield!(98usize);
            } else if true {
                _yield!(2usize);
            } else {
                _yield!(97usize);
            }

            _yield!(3usize);
        }}
    }

    assert_eq!(foo().collect::<Vec<_>>(), vec![1, 2, 3]);
}

#[test]
fn match_statements() {
    #[allow(unreachable_code)]
    fn foo() -> impl Iterator<Item = usize> {
        fake_yield! {
            match Some(42usize) {
                None => _yield!(2usize),
                Some(1) => panic!("LOL"),
                Some(x) => _yield!(x),
            }
        }
    }

    assert_eq!(foo().collect::<Vec<_>>(), vec![42]);
}

#[test]
fn void_exprs() {
    fn foo(c1: bool) -> impl Iterator<Item = usize> {
        fake_yield! {{
            if c1 {
                ()
            } else {
                _yield!(42);
            }

            if true {
                ()
            } else {
                ()
            }
        }}
    }

    assert_eq!(foo(true).collect::<Vec<_>>(), vec![]);
    assert_eq!(foo(false).collect::<Vec<_>>(), vec![42]);
}

#[test]
fn for_loops() {
    fn foo() -> impl Iterator<Item = usize> {
        fake_yield! {{
            for &i in (&[1, 99, 2, 60]) {
                if (i == 1 || i == 2) {
                    _yield!(i);
                }
            }

            _yield!(3usize);
        }}
    }

    assert_eq!(foo().collect::<Vec<_>>(), vec![1, 2, 3]);
}
