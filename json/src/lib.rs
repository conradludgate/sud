use std::io::{self, Write};

use sud_core::{Atom, Event};

mod frame;

pub struct JsonSerializer<W> {
    enc: JsonEncoder,
    out: W,
}

#[derive(Default)]
struct JsonEncoder {
    stack: Vec<State>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct State {
    pos: Position,
    object: Object,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Position {
    First,
    NotFirst,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Object {
    MapKey,
    MapValue,
    Value,
}

impl Object {
    fn prefix(self) -> &'static [u8; 1] {
        match self {
            Object::MapKey | Object::Value => b",",
            Object::MapValue => b":",
        }
    }

    fn next(self) -> Self {
        match self {
            Object::MapKey => Object::MapValue,
            Object::MapValue => Object::MapKey,
            Object::Value => Object::Value,
        }
    }
}

impl<W> JsonSerializer<W> {
    pub fn new(output: W) -> Self {
        Self {
            enc: JsonEncoder::default(),
            out: output,
        }
    }
}

impl<W: Write> JsonSerializer<W> {
    pub fn write(&mut self, event: Event<'_>) -> io::Result<()> {
        self.enc.write(event, &mut self.out)
    }
}

impl JsonEncoder {
    pub fn write<W: Write>(&mut self, event: Event<'_>, mut dst: W) -> io::Result<()> {
        match event {
            Event::Atom(atom) => {
                if let Some(State { pos, object }) = self.stack.last_mut() {
                    if *pos == Position::NotFirst {
                        dst.write_all(object.prefix())?;
                    } else {
                        *pos = Position::NotFirst;
                    }
                }

                match atom {
                    Atom::Null => dst.write_all(b"null")?,
                    Atom::Bool(true) => dst.write_all(b"true")?,
                    Atom::Bool(false) => dst.write_all(b"false")?,
                    Atom::Char(c) => format_escaped_str(&mut dst, c.encode_utf8(&mut [0; 4]))?,
                    Atom::Str(s) => format_escaped_str(&mut dst, &s)?,
                    Atom::Bytes(b) => {
                        let mut buf = itoa::Buffer::new();
                        let mut not_first = false;
                        dst.write_all(b"[")?;
                        for b in &*b {
                            if not_first {
                                dst.write_all(b",")?;
                                not_first = true;
                            }
                            dst.write_all(buf.format(*b).as_bytes())?;
                        }
                        dst.write_all(b"]")?;
                    }
                    Atom::U64(i) => {
                        let mut buf = itoa::Buffer::new();
                        dst.write_all(buf.format(i).as_bytes())?;
                    }
                    Atom::I64(i) => {
                        let mut buf = itoa::Buffer::new();
                        dst.write_all(buf.format(i).as_bytes())?;
                    }
                    Atom::F64(i) => {
                        let mut buf = ryu::Buffer::new();
                        dst.write_all(buf.format(i).as_bytes())?;
                    }
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            "unsupported atom in JSON",
                        ))
                    }
                }

                if let Some(State { object, .. }) = self.stack.last_mut() {
                    *object = object.next();
                }

                Ok(())
            }
            Event::MapStart(_) => {
                dst.write_all(b"{")?;
                self.stack.push(State {
                    pos: Position::First,
                    object: Object::MapKey,
                });
                Ok(())
            }
            Event::MapEnd => {
                dst.write_all(b"}")?;
                self.stack.pop();
                Ok(())
            }
            Event::SeqStart(_) => {
                dst.write_all(b"[")?;
                self.stack.push(State {
                    pos: Position::First,
                    object: Object::Value,
                });
                Ok(())
            }
            Event::SeqEnd => {
                dst.write_all(b"]")?;
                self.stack.pop();
                Ok(())
            }
        }
    }
}

fn format_escaped_str<W>(writer: &mut W, value: &str) -> io::Result<()>
where
    W: ?Sized + io::Write,
{
    writer.write_all(b"\"")?;
    format_escaped_str_contents(writer, value)?;
    writer.write_all(b"\"")
}

/// Represents a character escape code in a type-safe manner.
enum CharEscape {
    /// An escaped quote `"`
    Quote,
    /// An escaped reverse solidus `\`
    ReverseSolidus,
    // /// An escaped solidus `/`
    // Solidus,
    /// An escaped backspace character (usually escaped as `\b`)
    Backspace,
    /// An escaped form feed character (usually escaped as `\f`)
    FormFeed,
    /// An escaped line feed character (usually escaped as `\n`)
    LineFeed,
    /// An escaped carriage return character (usually escaped as `\r`)
    CarriageReturn,
    /// An escaped tab character (usually escaped as `\t`)
    Tab,
    /// An escaped ASCII plane control character (usually escaped as
    /// `\u00XX` where `XX` are two hex characters)
    AsciiControl(u8),
}

impl CharEscape {
    #[inline]
    fn from_escape_table(escape: u8, byte: u8) -> CharEscape {
        match escape {
            self::BB => CharEscape::Backspace,
            self::TT => CharEscape::Tab,
            self::NN => CharEscape::LineFeed,
            self::FF => CharEscape::FormFeed,
            self::RR => CharEscape::CarriageReturn,
            self::QU => CharEscape::Quote,
            self::BS => CharEscape::ReverseSolidus,
            self::UU => CharEscape::AsciiControl(byte),
            _ => unreachable!(),
        }
    }
}

/// Writes a character escape code to the specified writer.
fn write_char_escape<W>(writer: &mut W, char_escape: CharEscape) -> io::Result<()>
where
    W: ?Sized + io::Write,
{
    use self::CharEscape::*;

    let s = match char_escape {
        Quote => b"\\\"",
        ReverseSolidus => b"\\\\",
        // Solidus => b"\\/",
        Backspace => b"\\b",
        FormFeed => b"\\f",
        LineFeed => b"\\n",
        CarriageReturn => b"\\r",
        Tab => b"\\t",
        AsciiControl(byte) => {
            static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
            let bytes = &[
                b'\\',
                b'u',
                b'0',
                b'0',
                HEX_DIGITS[(byte >> 4) as usize],
                HEX_DIGITS[(byte & 0xF) as usize],
            ];
            return writer.write_all(bytes);
        }
    };

    writer.write_all(s)
}

fn format_escaped_str_contents<W>(writer: &mut W, value: &str) -> io::Result<()>
where
    W: ?Sized + io::Write,
{
    let bytes = value.as_bytes();

    let mut start = 0;

    for (i, &byte) in bytes.iter().enumerate() {
        let escape = ESCAPE[byte as usize];
        if escape == 0 {
            continue;
        }

        if start < i {
            writer.write_all(&bytes[start..i])?;
        }

        let char_escape = CharEscape::from_escape_table(escape, byte);
        write_char_escape(writer, char_escape)?;

        start = i + 1;
    }

    if start == bytes.len() {
        return Ok(());
    }

    writer.write_all(&bytes[start..])
}

const BB: u8 = b'b'; // \x08
const TT: u8 = b't'; // \x09
const NN: u8 = b'n'; // \x0A
const FF: u8 = b'f'; // \x0C
const RR: u8 = b'r'; // \x0D
const QU: u8 = b'"'; // \x22
const BS: u8 = b'\\'; // \x5C
const UU: u8 = b'u'; // \x00...\x1F except the ones above
const __: u8 = 0;

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in JSON. A value of 0 means that byte i is not escaped.
static ESCAPE: [u8; 256] = [
    //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    UU, UU, UU, UU, UU, UU, UU, UU, BB, TT, NN, UU, FF, RR, UU, UU, // 0
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 1
    __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
    __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, future::poll_fn, pin::pin, task::Poll};

    use futures_util::{sink::Sink, SinkExt};
    use sud_core::{tri, Serializer};
    use tokio_util::codec::FramedWrite;

    use crate::{JsonEncoder, JsonSerializer};

    #[test]
    fn int_slice() {
        let data = [1, 2, 3, 4];

        let mut serializer = JsonSerializer::new(Vec::new());

        data.try_for_each_event(&mut data.get_state(), |event| serializer.write(event))
            .unwrap();

        let output = String::from_utf8(serializer.out).unwrap();
        assert_eq!(output, "[1,2,3,4]");

        assert_eq!(serializer.enc.stack, &[]);
    }

    #[test]
    fn str_map() {
        let data = HashMap::from([("abc", 1), ("def", 2)]);

        let mut serializer = JsonSerializer::new(Vec::new());

        data.try_for_each_event(&mut data.get_state(), |event| serializer.write(event))
            .unwrap();

        let order_a = r#"{"abc":1,"def":2}"#;
        let order_b = r#"{"def":2,"abc":1}"#;

        let output = String::from_utf8(serializer.out).unwrap();
        if output != order_a {
            assert_eq!(output, order_b);
        }

        assert_eq!(serializer.enc.stack, &[]);
    }

    #[tokio::test]
    async fn async_str_map() {
        let data = HashMap::from([("abc", 1), ("def", 2)]);

        let mut serializer = pin!(FramedWrite::new(Vec::new(), JsonEncoder::default()));

        let mut state = data.get_state();
        poll_fn(|cx| {
            data.try_for_each_event(&mut state, |event| {
                tri!(serializer.as_mut().poll_ready(cx));
                Poll::Ready(serializer.as_mut().start_send(event))
            })
        })
        .await
        .unwrap();

        serializer.close().await.unwrap();

        let order_a = r#"{"abc":1,"def":2}"#;
        let order_b = r#"{"def":2,"abc":1}"#;

        let output = String::from_utf8(serializer.get_ref().to_owned()).unwrap();
        if output != order_a {
            assert_eq!(output, order_b);
        }

        assert_eq!(serializer.encoder().stack, &[]);
    }
}
