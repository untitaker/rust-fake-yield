use std::mem;

#[macro_export]
macro_rules! fake_yield {
    () => (None.into_iter());

    (if ($cond:expr) { $($stuff:tt)* } else { $($stuff2:tt)* } 
     $($rest:tt)*) => {
        CallbackIterator::Uncalled(move || {
            fake_yield!(
                _yield_from!(if $cond {
                    itertools::Either::Left(fake_yield!($($stuff)*))
                } else {
                    itertools::Either::Right(fake_yield!($($stuff2)*))
                });

                $($rest)*
            )
        })
    };

    (if ($cond:expr) { $($stuff:tt)* }
     $($rest:tt)*) => {
        fake_yield!(
            if ($cond) { $($stuff)* } else {}
            $($rest)*
        )
    };

    (_yield_from!($stuff:expr); $($rest:tt)*) => {
        CallbackIterator::Uncalled(move || {
            ::std::iter::Iterator::chain(
                $stuff,
                fake_yield!($($rest)*)
            )
        })
    };

    (_yield!($stuff:expr); $($rest:tt)*) => {
        fake_yield!(_yield_from!(Some($stuff).into_iter()); $($rest)*)
    };

    ($stuff:expr; $($rest:tt)*) => {
        CallbackIterator::Uncalled(move || {
            $stuff;
            fake_yield!($($rest)*)
        })
    };
    ($stuff:stmt; $($rest:tt)*) => {
        CallbackIterator::Uncalled(move || {
            $stuff;
            fake_yield!($($rest)*)
        })
    };

    ($stuff:expr) => ($stuff);
}

pub enum CallbackIterator<I, F> {
    Uncalled(F),
    Called(I),
    Empty,
}

impl<T, I: Iterator<Item = T>, F: FnOnce() -> I> Iterator for CallbackIterator<I, F> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let rv;

        match mem::replace(self, CallbackIterator::Empty) {
            CallbackIterator::Called(mut iter) => {
                rv = iter.next()?;
                *self = CallbackIterator::Called(iter)
            }
            CallbackIterator::Uncalled(f) => {
                let mut iter = f();
                rv = iter.next()?;
                *self = CallbackIterator::Called(iter)
            }
            CallbackIterator::Empty => return None,
        };
        Some(rv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        fn foo() -> impl Iterator<Item = usize> {
            fake_yield! {
                println!("hi");
                _yield!(1usize);
                println!("ho");
                _yield!(2usize);
            }
        }

        assert_eq!(foo().collect::<Vec<_>>(), vec![1, 2]);
    }

    #[test]
    fn if_statements() {
        fn foo() -> impl Iterator<Item = usize> {
            fake_yield! {
                println!("hi");
                if (false) {
                    _yield!(1usize);
                }
                println!("ho");
                if (true) {
                    _yield!(2usize);
                }

                if (false) {
                    _yield!(99usize);
                } else {
                    _yield!(3usize);
                }
            }
        }

        assert_eq!(foo().collect::<Vec<_>>(), vec![2, 3]);
    }

    #[test]
    fn if_statements_scoping() {
        #[allow(unused_variables)]
        fn foo() -> impl Iterator<Item = usize> {
            fake_yield! {
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
            }
        }

        assert_eq!(foo().collect::<Vec<_>>(), vec![1, 2, 3]);
    }
    
}
