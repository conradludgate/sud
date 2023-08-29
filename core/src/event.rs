use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum Atom<'a> {
    Null,
    Bool(bool),
    Str(Cow<'a, str>),
    Bytes(Cow<'a, [u8]>),
    Char(char),
    U64(u64),
    I64(i64),
    F64(f64),
}

impl<'a> Atom<'a> {
    /// Makes a static clone of the atom decoupling the lifetimes.
    pub fn to_static(&self) -> Atom<'static> {
        match *self {
            Atom::Null => Atom::Null,
            Atom::Bool(v) => Atom::Bool(v),
            Atom::Str(ref v) => Atom::Str(Cow::Owned(v.to_string())),
            Atom::Bytes(ref v) => Atom::Bytes(Cow::Owned(v.to_vec())),
            Atom::Char(v) => Atom::Char(v),
            Atom::U64(v) => Atom::U64(v),
            Atom::I64(v) => Atom::I64(v),
            Atom::F64(v) => Atom::F64(v),
        }
    }

    /// Makes a static clone of the atom decoupling the lifetimes.
    pub fn into_static(self) -> Atom<'static> {
        match self {
            Atom::Null => Atom::Null,
            Atom::Bool(v) => Atom::Bool(v),
            Atom::Str(v) => Atom::Str(Cow::Owned(v.into_owned())),
            Atom::Bytes(v) => Atom::Bytes(Cow::Owned(v.into_owned())),
            Atom::Char(v) => Atom::Char(v),
            Atom::U64(v) => Atom::U64(v),
            Atom::I64(v) => Atom::I64(v),
            Atom::F64(v) => Atom::F64(v),
        }
    }
}

macro_rules! impl_from {
    ($ty:ty, $atom:ident) => {
        impl From<$ty> for Event<'static> {
            fn from(value: $ty) -> Self {
                Event::Atom(Atom::$atom(value as _))
            }
        }
    };
}

impl_from!(u64, U64);
impl_from!(i64, I64);
impl_from!(f64, F64);
impl_from!(usize, U64);
impl_from!(isize, I64);
impl_from!(bool, Bool);
impl_from!(char, Char);

impl From<()> for Event<'static> {
    fn from(_: ()) -> Event<'static> {
        Event::Atom(Atom::Null)
    }
}

impl<'a> From<&'a str> for Event<'a> {
    fn from(value: &'a str) -> Event<'a> {
        Event::Atom(Atom::Str(Cow::Borrowed(value)))
    }
}

impl<'a> From<Cow<'a, str>> for Event<'a> {
    fn from(value: Cow<'a, str>) -> Event<'a> {
        Event::Atom(Atom::Str(value))
    }
}

impl<'a> From<&'a [u8]> for Event<'a> {
    fn from(value: &'a [u8]) -> Event<'a> {
        Event::Atom(Atom::Bytes(Cow::Borrowed(value)))
    }
}

impl From<String> for Event<'static> {
    fn from(value: String) -> Event<'static> {
        Event::Atom(Atom::Str(Cow::Owned(value)))
    }
}

impl<'a> From<Atom<'a>> for Event<'a> {
    fn from(atom: Atom<'a>) -> Self {
        Event::Atom(atom)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Event<'a> {
    Atom(Atom<'a>),
    MapStart(Option<usize>),
    MapEnd,
    SeqStart(Option<usize>),
    SeqEnd,
}

impl<'a> Event<'a> {
    /// Makes a static clone of the event decoupling the lifetimes.
    pub fn to_static(&self) -> Event<'static> {
        match *self {
            Event::Atom(ref atom) => Event::Atom(atom.to_static()),
            Event::MapStart(x) => Event::MapStart(x),
            Event::MapEnd => Event::MapEnd,
            Event::SeqStart(x) => Event::SeqStart(x),
            Event::SeqEnd => Event::SeqEnd,
        }
    }

    /// Makes a static clone of the event decoupling the lifetimes.
    pub fn into_static(self) -> Event<'static> {
        match self {
            Event::Atom(atom) => Event::Atom(atom.into_static()),
            Event::MapStart(x) => Event::MapStart(x),
            Event::MapEnd => Event::MapEnd,
            Event::SeqStart(x) => Event::SeqStart(x),
            Event::SeqEnd => Event::SeqEnd,
        }
    }
}
