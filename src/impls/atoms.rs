use std::borrow::Cow;

use crate::{
    event::{Atom, Event},
    Cursor, Serializer,
};

macro_rules! signed_int {
    ($int:ty) => {
        impl Serializer for $int {
            #[inline]
            fn estimate_size(&self) -> usize {
                1
            }

            #[inline]
            fn try_fold<'a, B, E, F>(
                &'a self,
                stack: Cursor<'_>,
                mut init: B,
                mut f: F,
            ) -> Result<B, E>
            where
                F: FnMut(B, Event<'a>) -> Result<B, E>,
            {
                init = f(init, Event::Atom(Atom::I64(*self as i64)))?;
                stack.complete();
                Ok(init)
            }
        }
    };
}

macro_rules! unsigned_int {
    ($int:ty) => {
        impl Serializer for $int {
            #[inline]
            fn estimate_size(&self) -> usize {
                1
            }

            #[inline]
            fn try_fold<'a, B, E, F>(
                &'a self,
                stack: Cursor<'_>,
                mut init: B,
                mut f: F,
            ) -> Result<B, E>
            where
                F: FnMut(B, Event<'a>) -> Result<B, E>,
            {
                init = f(init, Event::Atom(Atom::U64(*self as u64)))?;
                stack.complete();
                Ok(init)
            }
        }
    };
}

signed_int!(i8);
signed_int!(i16);
signed_int!(i32);
signed_int!(i64);
signed_int!(isize);
unsigned_int!(u16);
unsigned_int!(u32);
unsigned_int!(u64);
unsigned_int!(usize);

impl Serializer for u8 {
    #[inline]
    fn estimate_size(&self) -> usize {
        1
    }

    #[inline]
    fn try_fold<'a, B, E, F>(&'a self, stack: Cursor<'_>, mut init: B, mut f: F) -> Result<B, E>
    where
        F: FnMut(B, Event<'a>) -> Result<B, E>,
    {
        init = f(init, Event::Atom(Atom::U64(*self as u64)))?;
        stack.complete();
        Ok(init)
    }

    #[inline]
    fn __private_slice_as_bytes(_val: &[u8]) -> Option<Cow<'_, [u8]>>
    where
        Self: Sized,
    {
        Some(Cow::Borrowed(_val))
    }
}

impl Serializer for str {
    #[inline]
    fn estimate_size(&self) -> usize {
        1
    }

    #[inline]
    fn try_fold<'a, B, E, F>(&'a self, stack: Cursor<'_>, mut init: B, mut f: F) -> Result<B, E>
    where
        F: FnMut(B, Event<'a>) -> Result<B, E>,
    {
        init = f(init, Event::Atom(Atom::Str(Cow::Borrowed(self))))?;
        stack.complete();
        Ok(init)
    }
}

impl Serializer for String {
    #[inline]
    fn estimate_size(&self) -> usize {
        1
    }

    #[inline]
    fn try_fold<'a, B, E, F>(&'a self, stack: Cursor<'_>, init: B, f: F) -> Result<B, E>
    where
        F: FnMut(B, Event<'a>) -> Result<B, E>,
    {
        <str as Serializer>::try_fold(self, stack, init, f)
    }
}
