use crate::{event::Event, try_polyfill::Try, Serializer};

mod atoms;
mod list;
// mod map;

impl<T: Serializer + ?Sized> Serializer for &T {
    type State<'a>  = T::State<'a> where Self: 'a;

    fn try_fold_events<'a, B, R, F>(&'a self, state: &mut T::State<'a>, init: B, f: F) -> R
    where
        R: Try<Continue = B>,
        F: FnMut(B, Event<'a>) -> R,
    {
        <T as Serializer>::try_fold_events(self, state, init, f)
    }

    fn get_state<'a>(&'a self) -> Self::State<'a> {
        T::get_state(self)
    }
}
