#![recursion_limit = "128"]

use std::mem;

use proc_macro_hack::proc_macro_hack;

/// Starts yielding
#[proc_macro_hack]
pub use fake_yield_inner::fake_yield;

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
