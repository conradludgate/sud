//! Alternative to SerDe. WIP

use std::{borrow::Cow, convert::Infallible};

use event::Event;

mod event;
mod impls;

pub struct Stack {
    stack: Vec<usize>,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            stack: Vec::with_capacity(8),
        }
    }

    pub fn start(&mut self) -> Cursor<'_> {
        if self.stack.is_empty() {
            self.stack.push(0);
        }
        Cursor {
            inner: &mut self.stack,
            depth: 0,
        }
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Cursor<'a> {
    inner: &'a mut Vec<usize>,
    depth: usize,
}

impl<'a> Cursor<'a> {
    #[inline]
    pub fn get(&self) -> usize {
        unsafe { *self.inner.get_unchecked(self.depth) }
    }

    #[inline]
    pub fn next(&mut self) {
        unsafe { *self.inner.get_unchecked_mut(self.depth) += 1 }
    }

    #[inline]
    pub fn deeper(&mut self) -> Cursor<'_> {
        let depth = self.depth + 1;
        if self.inner.len() == depth {
            self.inner.push(0);
        }
        Cursor {
            inner: self.inner,
            depth,
        }
    }

    #[inline]
    pub fn complete(self) {
        self.inner.truncate(self.depth);
    }
}

pub trait Serializer {
    // lower bound on how many events this layer will emit
    fn estimate_size(&self) -> usize {
        0
    }

    fn fill_buffer<'a>(&'a self, stack: Cursor<'_>, buf: &mut Vec<Event<'a>>) {
        let _ = self.try_fold(stack, (), |(), event| {
            if buf.len() < buf.capacity() {
                buf.push(event);
                Ok(())
            } else {
                Err(())
            }
        });
    }

    #[inline]
    fn fold<'a, B, F>(&'a self, stack: Cursor<'_>, init: B, mut f: F) -> B
    where
        F: FnMut(B, Event<'a>) -> B,
    {
        match self.try_fold(stack, init, |acc, x| Ok::<B, Infallible>(f(acc, x))) {
            Ok(r) => r,
            Err(x) => match x {},
        }
    }

    fn try_fold<'a, B, E, F>(&'a self, stack: Cursor<'_>, init: B, f: F) -> Result<B, E>
    where
        F: FnMut(B, Event<'a>) -> Result<B, E>;

    /// Hidden internal trait method to allow specializations of bytes.
    ///
    /// This method is used by `u8` and `Vec<T>` / `&[T]` to achieve special
    /// casing of bytes for the serialization system.  It allows a vector of
    /// bytes to be emitted as `Chunk::Bytes` rather than a `Seq`.
    #[doc(hidden)]
    #[inline]
    fn __private_slice_as_bytes(_val: &[Self]) -> Option<Cow<'_, [u8]>>
    where
        Self: Sized,
    {
        None
    }
}

pub fn bytes(x: &[u8]) -> Vec<Event<'_>> {
    let mut stack = Stack::new();
    let mut v = Vec::with_capacity(128);
    x.fill_buffer(stack.start(), &mut v);
    v
}

pub fn ints(x: &[i32]) -> Vec<Event<'_>> {
    let mut stack = Stack::new();
    let mut v = Vec::with_capacity(128);
    x.fill_buffer(stack.start(), &mut v);
    v
}
