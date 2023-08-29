use std::collections::HashMap;

use crate::{
    event::{Atom, Event},
    tri,
    try_polyfill::Try,
    Cursor, Serializer,
};

impl<K: Serializer, V: Serializer> Serializer for HashMap<K, V> {
    #[inline]
    fn estimate_size(&self) -> usize {
        self.len() * 2 + 2
    }

    fn try_fold_events<'a, B, R, F>(&'a self, mut stack: Cursor<'_>, mut init: B, mut f: F) -> R
    where
        R: Try<Continue = B>,
        F: FnMut(B, Event<'a>) -> R,
    {
        if stack.get() == 0 {
            init = tri!(f(init, Event::MapStart(Some(self.len()))));
            stack.next();
        }

        for t in &self[stack.get() - 1..] {
            init = tri!(t.try_fold_events(stack.deeper(), init, &mut f));
            stack.next();
        }

        init = tri!(f(init, Event::MapEnd));
        stack.complete();
        R::from_continue(init)
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
}
