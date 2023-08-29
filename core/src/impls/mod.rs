use crate::{event::Event, try_polyfill::Try, Cursor, Serializer};

mod atoms;
mod list;
// mod map;

impl<T: Serializer + ?Sized> Serializer for &T {
    fn try_fold_events<'a, B, R, F>(&'a self, stack: Cursor<'_>, init: B, f: F) -> R
    where
        R: Try<Continue = B>,
        F: FnMut(B, Event<'a>) -> R,
    {
        <T as Serializer>::try_fold_events(self, stack, init, f)
    }
}
