#![no_std]

#[cfg(doctest)]
#[macro_use]
extern crate doc_comment;

#[cfg(doctest)]
doctest!("../readme.md");

use core::cmp::{Eq, Ord, Ordering};
use core::fmt::Debug;
use core::mem::{forget, ManuallyDrop};
use core::ops::{Deref, DerefMut};

/// Container that holds a value and a customestructor
#[derive(Clone, Debug, Eq, Ord)]
pub struct WithDrop<T, F: FnOnce(T)> {
    data: ManuallyDrop<(T, F)>,
}

impl<T, F: FnOnce(T)> WithDrop<T, F> {
    pub fn new(inner: T, drop_fn: F) -> Self {
        Self {
            data: ManuallyDrop::new((inner, drop_fn)),
        }
    }

    /// This extracts the contained value while dropping the closure
    /// and the container.
    ///
    /// The custom closure will *not* be executed.
    pub fn into_inner(mut self) -> T {
        let (v, _) = unsafe { ManuallyDrop::take(&mut self.data) };
        forget(self);
        v
    }
}

impl<T: PartialEq<T>, F1: FnOnce(T), F2: FnOnce(T)> PartialEq<WithDrop<T, F2>> for WithDrop<T, F1> {
    fn eq(&self, other: &WithDrop<T, F2>) -> bool {
        self.deref().eq(other.deref())
    }
}

impl<T: PartialOrd<T>, F1: FnOnce(T), F2: FnOnce(T)> PartialOrd<WithDrop<T, F2>>
    for WithDrop<T, F1>
{
    fn partial_cmp(&self, other: &WithDrop<T, F2>) -> Option<Ordering> {
        self.deref().partial_cmp(other.deref())
    }
}

impl<T, F: FnOnce(T)> Drop for WithDrop<T, F> {
    fn drop(&mut self) {
        let (v, f) = unsafe { ManuallyDrop::take(&mut self.data) };
        f(v);
    }
}

impl<T, F: FnOnce(T)> Deref for WithDrop<T, F> {
    type Target = T;
    fn deref(&self) -> &T {
        &(*self.data).0
    }
}

impl<T, F: FnOnce(T)> DerefMut for WithDrop<T, F> {
    fn deref_mut(&mut self) -> &mut T {
        &mut (*self.data).0
    }
}

/// Alias for WithDrop::new
pub fn with_drop<T, F: FnOnce(T)>(inner: T, drop_fn: F) -> WithDrop<T, F> {
    WithDrop::new(inner, drop_fn)
}
