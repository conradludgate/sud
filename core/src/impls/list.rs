use crate::{
    event::{Atom, Event},
    tri,
    try_polyfill::Try,
    Cursor, Serializer,
};

impl<T: Serializer> Serializer for [T] {
    #[inline]
    fn estimate_size(&self) -> usize {
        if T::__private_slice_as_bytes(self).is_some() {
            1
        } else {
            self.len() + 2
        }
    }

    fn fold_events<'a, B, F>(&'a self, mut stack: Cursor<'_>, mut init: B, mut f: F) -> B
    where
        F: FnMut(B, Event<'a>) -> B,
    {
        if let Some(slice) = T::__private_slice_as_bytes(self) {
            init = f(init, Event::Atom(Atom::Bytes(slice)));
            stack.complete();
            init
        } else {
            if stack.get() == 0 {
                init = f(init, Event::SeqStart(Some(self.len())));
                stack.next();
            }

            for t in &self[stack.get() - 1..] {
                init = t.fold_events(stack.deeper(), init, &mut f);
                stack.next();
            }

            init = f(init, Event::SeqEnd);
            stack.complete();
            init
        }
    }

    fn try_fold_events<'a, B, R, F>(&'a self, mut stack: Cursor<'_>, mut init: B, mut f: F) -> R
    where
        R: Try<Continue = B>,
        F: FnMut(B, Event<'a>) -> R,
    {
        if let Some(slice) = T::__private_slice_as_bytes(self) {
            init = tri!(f(init, Event::Atom(Atom::Bytes(slice))));
            stack.complete();
            R::from_continue(init)
        } else {
            if stack.get() == 0 {
                init = tri!(f(init, Event::SeqStart(Some(self.len()))));
                stack.next();
            }

            for t in &self[stack.get() - 1..] {
                init = tri!(t.try_fold_events(stack.deeper(), init, &mut f));
                stack.next();
            }

            init = tri!(f(init, Event::SeqEnd));
            stack.complete();
            R::from_continue(init)
        }
    }
}

impl<T: Serializer> Serializer for Vec<T> {
    #[inline]
    fn estimate_size(&self) -> usize {
        <[T] as Serializer>::estimate_size(self)
    }

    fn fold_events<'a, B, F>(&'a self, stack: Cursor<'_>, init: B, f: F) -> B
    where
        F: FnMut(B, Event<'a>) -> B,
    {
        <[T] as Serializer>::fold_events(self, stack, init, f)
    }

    fn try_fold_events<'a, B, R, F>(&'a self, stack: Cursor<'_>, init: B, f: F) -> R
    where
        R: Try<Continue = B>,
        F: FnMut(B, Event<'a>) -> R,
    {
        <[T] as Serializer>::try_fold_events(self, stack, init, f)
    }
}

impl<T: Serializer, const N: usize> Serializer for [T; N] {
    #[inline]
    fn estimate_size(&self) -> usize {
        <[T] as Serializer>::estimate_size(self)
    }

    fn fold_events<'a, B, F>(&'a self, stack: Cursor<'_>, init: B, f: F) -> B
    where
        F: FnMut(B, Event<'a>) -> B,
    {
        <[T] as Serializer>::fold_events(self, stack, init, f)
    }

    fn try_fold_events<'a, B, R, F>(&'a self, stack: Cursor<'_>, init: B, f: F) -> R
    where
        R: Try<Continue = B>,
        F: FnMut(B, Event<'a>) -> R,
    {
        <[T] as Serializer>::try_fold_events(self, stack, init, f)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::{
        event::{Atom, Event},
        Serializer, Stack,
    };

    #[test]
    fn serialize() {
        let mut stack = Stack::new();
        let mut v = Vec::new();

        let a = &["abc", "def"][..];
        a.fold_events(stack.start(), (), |(), event| v.push(event));

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
        let mut stack = Stack::new();
        let mut v = Vec::new();

        let a = &["abc", "def"][..];

        a.try_fold_events(stack.start(), (), |(), event| {
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

        a.try_fold_events(stack.start(), (), |(), event| {
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
        let mut stack = Stack::new();
        let mut v = Vec::new();

        let a: &[u8] = &b"abcdef"[..];
        a.fold_events(stack.start(), (), |(), event| v.push(event));

        assert_eq!(v, [Event::Atom(Atom::Bytes(Cow::Borrowed(b"abcdef"))),]);
    }
}
