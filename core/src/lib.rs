//! Alternative to SerDe. WIP

use std::{borrow::Cow, convert::Infallible};

pub use event::{Atom, Event};
use try_polyfill::Try;

mod event;
mod impls;

pub trait Serializer {
    type State<'a> where Self: 'a;

    // lower bound on how many events this layer will emit
    fn estimate_size(&self) -> usize {
        0
    }

    fn fill_buffer<'a>(&'a self, state: &mut Self::State<'a>, buf: &mut Vec<Event<'a>>) {
        let _ = self.try_fold_events(state, (), |(), event| {
            if buf.len() < buf.capacity() {
                buf.push(event);
                Ok(())
            } else {
                Err(())
            }
        });
    }

    #[inline]
    fn for_each_event<'a, F>(&'a self, state: &mut Self::State<'a>, mut f: F)
    where
        F: FnMut(Event<'a>),
    {
        self.fold_events(state, (), |(), x| f(x))
    }

    #[inline]
    fn fold_events<'a, B, F>(&'a self, state: &mut Self::State<'a>, init: B, mut f: F) -> B
    where
        F: FnMut(B, Event<'a>) -> B,
    {
        match self.try_fold_events(state, init, |acc, x| Ok::<B, Infallible>(f(acc, x))) {
            Ok(r) => r,
            Err(x) => match x {},
        }
    }

    #[inline]
    fn try_for_each_event<'a, R, F>(&'a self, state: &mut Self::State<'a>, mut f: F) -> R
    where
        R: Try<Continue = ()>,
        F: FnMut(Event<'a>) -> R,
    {
        self.try_fold_events(state, (), |(), x| f(x))
    }

    fn try_fold_events<'a, B, R, F>(&'a self, state: &mut Self::State<'a>, init: B, f: F) -> R
    where
        R: Try<Continue = B>,
        F: FnMut(B, Event<'a>) -> R;

    fn get_state(&self) -> Self::State<'_>;

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

pub mod try_polyfill {
    use std::{convert::Infallible, ops::ControlFlow, task::Poll};

    pub trait Try {
        type Break;
        type Continue;
        fn branch(self) -> ControlFlow<Self::Break, Self::Continue>;
        fn from_break(b: Self::Break) -> Self;
        fn from_continue(c: Self::Continue) -> Self;
    }

    impl<T, E> Try for Result<T, E> {
        type Break = Result<Infallible, E>;
        type Continue = T;

        fn branch(self) -> ControlFlow<Self::Break, Self::Continue> {
            match self {
                Ok(t) => ControlFlow::Continue(t),
                Err(e) => ControlFlow::Break(Err(e)),
            }
        }

        fn from_break(e: Result<Infallible, E>) -> Self {
            match e {
                Err(e) => Err(e),
                Ok(i) => match i {},
            }
        }

        fn from_continue(t: T) -> Self {
            Ok(t)
        }
    }

    impl<T> Try for Option<T> {
        type Break = Option<Infallible>;
        type Continue = T;

        fn branch(self) -> ControlFlow<Self::Break, Self::Continue> {
            match self {
                Some(t) => ControlFlow::Continue(t),
                None => ControlFlow::Break(None),
            }
        }

        fn from_break(_: Option<Infallible>) -> Self {
            None
        }
        fn from_continue(t: T) -> Self {
            Some(t)
        }
    }

    impl<T, E> Try for Poll<Result<T, E>> {
        type Break = Poll<Result<Infallible, E>>;
        type Continue = T;

        fn branch(self) -> ControlFlow<Self::Break, Self::Continue> {
            match self {
                Poll::Ready(Ok(t)) => ControlFlow::Continue(t),
                Poll::Ready(Err(e)) => ControlFlow::Break(Poll::Ready(Err(e))),
                Poll::Pending => ControlFlow::Break(Poll::Pending),
            }
        }

        fn from_break(b: Poll<Result<Infallible, E>>) -> Self {
            match b {
                Poll::Ready(Ok(a)) => match a {},
                Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                Poll::Pending => Poll::Pending,
            }
        }
        fn from_continue(t: T) -> Self {
            Poll::Ready(Ok(t))
        }
    }

    #[macro_export]
    macro_rules! tri {
        ($expr:expr) => {
            match $crate::try_polyfill::Try::branch($expr) {
                ::std::ops::ControlFlow::Break(b) => {
                    return $crate::try_polyfill::Try::from_break(b)
                }
                ::std::ops::ControlFlow::Continue(c) => c,
            }
        };
    }
}
