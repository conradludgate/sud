use std::borrow::Cow;

use crate::{
    event::{Atom, Event},
    try_polyfill::Try,
    Serializer,
};

macro_rules! signed_int {
    ($int:ty) => {
        impl Serializer for $int {
            type State<'a> = ();
            fn get_state(&self) {}

            #[inline]
            fn estimate_size(&self) -> usize {
                1
            }

            #[inline]
            fn try_fold_events<'a, B, R, F>(&'a self, _state: &mut (), init: B, mut f: F) -> R
            where
                R: Try<Continue = B>,
                F: FnMut(B, Event<'a>) -> R,
            {
                f(init, Event::Atom(Atom::I64(*self as i64)))
            }
        }
    };
}

macro_rules! unsigned_int {
    ($int:ty) => {
        impl Serializer for $int {
            type State<'a> = ();
            fn get_state(&self) {}

            #[inline]
            fn estimate_size(&self) -> usize {
                1
            }

            #[inline]
            fn try_fold_events<'a, B, R, F>(&'a self, _state: &mut (), init: B, mut f: F) -> R
            where
                R: Try<Continue = B>,
                F: FnMut(B, Event<'a>) -> R,
            {
                f(init, Event::Atom(Atom::U64(*self as u64)))
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
    type State<'a> = ();
    fn get_state(&self) {}

    #[inline]
    fn estimate_size(&self) -> usize {
        1
    }

    #[inline]
    fn try_fold_events<'a, B, R, F>(&'a self, _state: &mut (), init: B, mut f: F) -> R
    where
        R: Try<Continue = B>,
        F: FnMut(B, Event<'a>) -> R,
    {
        f(init, Event::Atom(Atom::U64(*self as u64)))
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
    type State<'a> = ();
    fn get_state(&self) {}

    #[inline]
    fn estimate_size(&self) -> usize {
        1
    }

    #[inline]
    fn try_fold_events<'a, B, R, F>(&'a self, _state: &mut (), init: B, mut f: F) -> R
    where
        R: Try<Continue = B>,
        F: FnMut(B, Event<'a>) -> R,
    {
        f(init, Event::Atom(Atom::Str(Cow::Borrowed(self))))
    }
}

impl Serializer for String {
    type State<'a> = ();
    fn get_state(&self) {}

    #[inline]
    fn estimate_size(&self) -> usize {
        1
    }

    #[inline]
    fn try_fold_events<'a, B, R, F>(&'a self, _state: &mut (), init: B, f: F) -> R
    where
        R: Try<Continue = B>,
        F: FnMut(B, Event<'a>) -> R,
    {
        <str as Serializer>::try_fold_events(self, _state, init, f)
    }
}
