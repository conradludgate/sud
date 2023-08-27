use crate::{event::Event, Cursor, Serializer};

mod atoms;
mod list;

impl<T: Serializer + ?Sized> Serializer for &T {
    fn try_fold<'a, B, E, F>(&'a self, stack: Cursor<'_>, init: B, f: F) -> Result<B, E>
    where
        F: FnMut(B, Event<'a>) -> Result<B, E>,
    {
        <T as Serializer>::try_fold(self, stack, init, f)
    }
}
