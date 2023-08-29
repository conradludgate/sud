use std::{
    collections::{btree_map, hash_map, BTreeMap, HashMap},
    hash::BuildHasher,
};

use crate::{event::Event, tri, try_polyfill::Try, Serializer};

pub enum MapState<'a, K: Serializer, V: Serializer, I: Iterator<Item = (&'a K, &'a V)>> {
    Start,
    Entries(Entry<'a, K, V>, I),
    Empty,
}

pub enum Entry<'a, K: Serializer, V: Serializer> {
    Key(&'a K, K::State<'a>, &'a V),
    Value(&'a V, V::State<'a>),
}

impl<K: Serializer, V: Serializer, S: BuildHasher> Serializer for HashMap<K, V, S> {
    type State<'a>  = MapState<'a, K, V, hash_map::Iter<'a, K, V>> where Self: 'a;
    fn get_state(&self) -> Self::State<'_> {
        MapState::Start
    }

    #[inline]
    fn estimate_size(&self) -> usize {
        self.len() * 2 + 2
    }

    fn try_fold_events<'a, B, R, F>(
        &'a self,
        state: &mut Self::State<'a>,
        mut init: B,
        mut f: F,
    ) -> R
    where
        R: Try<Continue = B>,
        F: FnMut(B, Event<'a>) -> R,
    {
        loop {
            match state {
                MapState::Start => {
                    init = tri!(f(init, Event::MapStart(Some(self.len()))));
                    let mut iter = self.iter();

                    match iter.next() {
                        Some((key, value)) => {
                            *state =
                                MapState::Entries(Entry::Key(key, key.get_state(), value), iter);
                        }
                        None => *state = MapState::Empty,
                    }
                }
                MapState::Entries(entry, iter) => match entry {
                    Entry::Key(k, s, v) => {
                        init = tri!(k.try_fold_events(s, init, &mut f));
                        *entry = Entry::Value(v, v.get_state());
                    }
                    Entry::Value(v, s) => {
                        init = tri!(v.try_fold_events(s, init, &mut f));
                        match iter.next() {
                            Some((key, value)) => {
                                *entry = Entry::Key(key, key.get_state(), value);
                            }
                            None => *state = MapState::Empty,
                        }
                    }
                },
                MapState::Empty => break f(init, Event::MapEnd),
            }
        }
    }
}

impl<K: Serializer, V: Serializer> Serializer for BTreeMap<K, V> {
    type State<'a>  = MapState<'a, K, V, btree_map::Iter<'a, K, V>> where Self: 'a;
    fn get_state(&self) -> Self::State<'_> {
        MapState::Start
    }

    #[inline]
    fn estimate_size(&self) -> usize {
        self.len() * 2 + 2
    }

    fn try_fold_events<'a, B, R, F>(
        &'a self,
        state: &mut Self::State<'a>,
        mut init: B,
        mut f: F,
    ) -> R
    where
        R: Try<Continue = B>,
        F: FnMut(B, Event<'a>) -> R,
    {
        loop {
            match state {
                MapState::Start => {
                    init = tri!(f(init, Event::MapStart(Some(self.len()))));
                    let mut iter = self.iter();

                    match iter.next() {
                        Some((key, value)) => {
                            *state =
                                MapState::Entries(Entry::Key(key, key.get_state(), value), iter);
                        }
                        None => *state = MapState::Empty,
                    }
                }
                MapState::Entries(entry, iter) => match entry {
                    Entry::Key(k, s, v) => {
                        init = tri!(k.try_fold_events(s, init, &mut f));
                        *entry = Entry::Value(v, v.get_state());
                    }
                    Entry::Value(v, s) => {
                        init = tri!(v.try_fold_events(s, init, &mut f));
                        match iter.next() {
                            Some((key, value)) => {
                                *entry = Entry::Key(key, key.get_state(), value);
                            }
                            None => *state = MapState::Empty,
                        }
                    }
                },
                MapState::Empty => break f(init, Event::MapEnd),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        event::{Atom, Event},
        Serializer,
    };

    #[test]
    fn hash() {
        let mut v = Vec::new();

        let a = HashMap::from([("abc", 1), ("def", 2)]);
        a.fold_events(&mut a.get_state(), (), |(), event| v.push(event));

        let order_a = [
            Event::MapStart(Some(2)),
            Event::Atom(Atom::Str("abc".into())),
            Event::Atom(Atom::I64(1)),
            Event::Atom(Atom::Str("def".into())),
            Event::Atom(Atom::I64(2)),
            Event::MapEnd,
        ];
        let order_b = [
            Event::MapStart(Some(2)),
            Event::Atom(Atom::Str("def".into())),
            Event::Atom(Atom::I64(2)),
            Event::Atom(Atom::Str("abc".into())),
            Event::Atom(Atom::I64(1)),
            Event::MapEnd,
        ];

        if v != order_a {
            assert_eq!(v, order_b);
        }
    }

    #[test]
    fn btree() {
        let mut v = Vec::new();

        let a = HashMap::from([("abc", 1), ("def", 2)]);
        a.fold_events(&mut a.get_state(), (), |(), event| v.push(event));

        let exp = [
            Event::MapStart(Some(2)),
            Event::Atom(Atom::Str("abc".into())),
            Event::Atom(Atom::I64(1)),
            Event::Atom(Atom::Str("def".into())),
            Event::Atom(Atom::I64(2)),
            Event::MapEnd,
        ];
        assert_eq!(v, exp);
    }
}
