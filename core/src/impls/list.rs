use crate::{
    event::{Atom, Event},
    tri,
    try_polyfill::Try,
    Serializer,
};

pub enum ListState<'a, T: Serializer> {
    Start,
    List(&'a T, T::State<'a>, &'a [T]),
    Empty,
}

impl<T: Serializer> Serializer for [T] {
    type State<'a> = ListState<'a, T> where Self: 'a;

    #[inline]
    fn estimate_size(&self) -> usize {
        if T::__private_slice_as_bytes(self).is_some() {
            1
        } else {
            self.len() + 2
        }
    }

    fn fold_events<'a, B, F>(&'a self, state: &mut ListState<'a, T>, mut init: B, mut f: F) -> B
    where
        F: FnMut(B, Event<'a>) -> B,
    {
        if let Some(slice) = T::__private_slice_as_bytes(self) {
            *state = ListState::Empty;
            f(init, Event::Atom(Atom::Bytes(slice)))
        } else {
            loop {
                match state {
                    ListState::Start => {
                        init = f(init, Event::SeqStart(Some(self.len())));
                        match self.split_first() {
                            Some((first, rest)) => {
                                *state = ListState::List(first, first.get_state(), rest)
                            }
                            None => *state = ListState::Empty,
                        }
                    }
                    ListState::List(t, s, slice) => {
                        init = t.fold_events(s, init, &mut f);
                        match slice.split_first() {
                            Some((first, rest)) => {
                                *state = ListState::List(first, first.get_state(), rest)
                            }
                            None => *state = ListState::Empty,
                        }
                    }
                    ListState::Empty => break f(init, Event::SeqEnd),
                }
            }
        }
    }

    fn try_fold_events<'a, B, R, F>(
        &'a self,
        state: &mut ListState<'a, T>,
        mut init: B,
        mut f: F,
    ) -> R
    where
        R: Try<Continue = B>,
        F: FnMut(B, Event<'a>) -> R,
    {
        if let Some(slice) = T::__private_slice_as_bytes(self) {
            let init = tri!(f(init, Event::Atom(Atom::Bytes(slice))));
            *state = ListState::Empty;
            R::from_continue(init)
        } else {
            loop {
                match state {
                    ListState::Start => {
                        init = tri!(f(init, Event::SeqStart(Some(self.len()))));
                        match self.split_first() {
                            Some((first, rest)) => {
                                *state = ListState::List(first, first.get_state(), rest)
                            }
                            None => *state = ListState::Empty,
                        }
                    }
                    ListState::List(t, s, slice) => {
                        init = tri!(t.try_fold_events(s, init, &mut f));
                        match slice.split_first() {
                            Some((first, rest)) => {
                                *state = ListState::List(first, first.get_state(), rest)
                            }
                            None => *state = ListState::Empty,
                        }
                    }
                    ListState::Empty => break f(init, Event::SeqEnd),
                }
            }
        }
    }

    fn get_state(&self) -> Self::State<'_> {
        ListState::Start
    }
}

impl<T: Serializer> Serializer for Vec<T> {
    #[inline]
    fn estimate_size(&self) -> usize {
        <[T] as Serializer>::estimate_size(self)
    }

    fn fold_events<'a, B, F>(&'a self, state: &mut ListState<'a, T>, init: B, f: F) -> B
    where
        F: FnMut(B, Event<'a>) -> B,
    {
        <[T] as Serializer>::fold_events(self, state, init, f)
    }

    fn try_fold_events<'a, B, R, F>(&'a self, state: &mut ListState<'a, T>, init: B, f: F) -> R
    where
        R: Try<Continue = B>,
        F: FnMut(B, Event<'a>) -> R,
    {
        <[T] as Serializer>::try_fold_events(self, state, init, f)
    }

    type State<'a> = ListState<'a, T> where Self: 'a;
    fn get_state(&self) -> Self::State<'_> {
        <[T] as Serializer>::get_state(self)
    }
}

impl<T: Serializer, const N: usize> Serializer for [T; N] {
    #[inline]
    fn estimate_size(&self) -> usize {
        <[T] as Serializer>::estimate_size(self)
    }

    fn fold_events<'a, B, F>(&'a self, state: &mut ListState<'a, T>, init: B, f: F) -> B
    where
        F: FnMut(B, Event<'a>) -> B,
    {
        <[T] as Serializer>::fold_events(self, state, init, f)
    }

    fn try_fold_events<'a, B, R, F>(&'a self, state: &mut ListState<'a, T>, init: B, f: F) -> R
    where
        R: Try<Continue = B>,
        F: FnMut(B, Event<'a>) -> R,
    {
        <[T] as Serializer>::try_fold_events(self, state, init, f)
    }

    type State<'a> = ListState<'a, T> where Self: 'a;
    fn get_state(&self) -> Self::State<'_> {
        <[T] as Serializer>::get_state(self)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::{
        event::{Atom, Event},
        Serializer,
    };

    #[test]
    fn serialize() {
        let mut v = Vec::new();

        let a = &["abc", "def"][..];
        a.fold_events(&mut a.get_state(), (), |(), event| v.push(event));

        assert_eq!(
            v,
            [
                Event::SeqStart(Some(2)),
                Event::Atom(Atom::Str("abc".into())),
                Event::Atom(Atom::Str("def".into())),
                Event::SeqEnd,
            ]
        );
    }

    #[test]
    fn try_serialize() {
        let mut v = Vec::new();

        let a = &["abc", "def"][..];
        let mut state = a.get_state();

        a.try_fold_events(&mut state, (), |(), event| {
            if v.len() < 2 {
                v.push(event);
                Ok(())
            } else {
                Err(())
            }
        })
        .unwrap_err();

        assert_eq!(
            v,
            [
                Event::SeqStart(Some(2)),
                Event::Atom(Atom::Str("abc".into())),
            ]
        );

        v.clear();

        a.try_fold_events(&mut state, (), |(), event| {
            if v.len() < 2 {
                v.push(event);
                Ok(())
            } else {
                Err(())
            }
        })
        .unwrap();

        assert_eq!(v, [Event::Atom(Atom::Str("def".into())), Event::SeqEnd,]);
    }

    #[test]
    fn serialize_bytes() {
        let mut v = Vec::new();

        let a: &[u8] = &b"abcdef"[..];
        a.fold_events(&mut a.get_state(), (), |(), event| v.push(event));

        assert_eq!(v, [Event::Atom(Atom::Bytes(Cow::Borrowed(b"abcdef"))),]);
    }
}
