use std::io::{self, Write};

use sud_core::{Atom, Event};

mod frame;

pub struct CborSerializer<W> {
    enc: CborEncoder,
    out: W,
}

#[derive(Default)]
struct CborEncoder {
    stack: Vec<State>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct State {
    expected_len: Option<usize>,
    actual_len: usize,
}

impl<W> CborSerializer<W> {
    pub fn new(output: W) -> Self {
        Self {
            enc: CborEncoder::default(),
            out: output,
        }
    }
}

impl<W: Write> CborSerializer<W> {
    pub fn write(&mut self, event: Event<'_>) -> io::Result<()> {
        self.enc.write(event, &mut self.out)
    }
}

impl CborEncoder {
    pub fn write<W: Write>(&mut self, event: Event<'_>, mut dst: W) -> io::Result<()> {
        match event {
            Event::Atom(atom) => {
                if let Some(State { actual_len, .. }) = self.stack.last_mut() {
                    *actual_len += 1;
                }

                match atom {
                    Atom::U64(i) => write_num(0x00, &mut dst, i),
                    Atom::I64(i) => {
                        if let Ok(i) = u64::try_from(i) {
                            write_num(0x00, &mut dst, i)
                        } else {
                            write_num(0x20, &mut dst, !i as u64)
                        }
                    }
                    Atom::Bytes(b) => write_encoded_bytes(0x40, &mut dst, &b),
                    Atom::Char(c) => {
                        write_encoded_bytes(0x60, &mut dst, c.encode_utf8(&mut [0; 4]).as_bytes())
                    }
                    Atom::Str(s) => write_encoded_bytes(0x60, &mut dst, s.as_bytes()),
                    Atom::Bool(false) => dst.write_all(&[0xf4]),
                    Atom::Bool(true) => dst.write_all(&[0xf5]),
                    Atom::Null => dst.write_all(&[0xf6]),
                    Atom::F64(i) => {
                        let mut buf = [0; 9];
                        buf[0] = 0xfb;
                        buf[1..9].copy_from_slice(&i.to_be_bytes());
                        dst.write_all(&buf)
                    }
                    _ => Err(io::Error::new(
                        io::ErrorKind::Other,
                        "unsupported atom in JSON",
                    )),
                }
            }
            Event::SeqStart(Some(len)) => {
                self.stack.push(State {
                    expected_len: Some(len),
                    actual_len: 0,
                });
                write_length(0x80, &mut dst, len)
            }
            Event::SeqStart(None) => {
                self.stack.push(State {
                    expected_len: None,
                    actual_len: 0,
                });
                dst.write_all(&[0x9f])
            }
            Event::SeqEnd => {
                let state = self.stack.pop().unwrap();
                if let Some(expected_len) = state.expected_len {
                    debug_assert_eq!(
                        expected_len, state.actual_len,
                        "the serialiser didn't produce the expected length array"
                    );
                    Ok(())
                } else {
                    dst.write_all(&[0xff])
                }
            }
            Event::MapStart(Some(len)) => {
                self.stack.push(State {
                    expected_len: Some(len * 2),
                    actual_len: 0,
                });
                write_length(0xa0, &mut dst, len)
            }
            Event::MapStart(None) => {
                self.stack.push(State {
                    expected_len: None,
                    actual_len: 0,
                });
                dst.write_all(&[0xbf])
            }
            Event::MapEnd => {
                let state = self.stack.pop().unwrap();
                if let Some(expected_len) = state.expected_len {
                    debug_assert_eq!(
                        expected_len, state.actual_len,
                        "the serialiser didn't produce the expected length map"
                    );
                    Ok(())
                } else {
                    dst.write_all(&[0xff])
                }
            }
        }
    }
}

fn write_length<W>(base: u8, writer: &mut W, value: usize) -> io::Result<()>
where
    W: ?Sized + io::Write,
{
    let Ok(len) = u64::try_from(value) else {
        panic!("bytes size should not exceed u64... how did you?!")
    };
    write_num(base, writer, len)
}

fn write_encoded_bytes<W>(base: u8, writer: &mut W, value: &[u8]) -> io::Result<()>
where
    W: ?Sized + io::Write,
{
    write_length(base, writer, value.len())?;
    writer.write_all(value)
}

fn write_num<W>(base: u8, writer: &mut W, n: u64) -> io::Result<()>
where
    W: ?Sized + io::Write,
{
    let mut buf = [0; 9];
    let (tag, len) = match n {
        n @ 0..=0x17 => (n as u8, 1),
        n @ ..=0xff => {
            buf[1] = n as u8;
            (0x18, 2)
        }
        n @ ..=0xffff => {
            buf[1..3].copy_from_slice(&(n as u16).to_be_bytes());
            (0x19, 3)
        }
        n @ ..=0xffff_ffff => {
            buf[1..5].copy_from_slice(&(n as u32).to_be_bytes());
            (0x1a, 5)
        }
        n @ ..=0xffff_ffff_ffff_ffff => {
            buf[1..9].copy_from_slice(&n.to_be_bytes());
            (0x1b, 9)
        }
    };
    buf[0] = base + tag;
    writer.write_all(&buf[..len])
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use sud_core::Serializer;

    use crate::CborSerializer;

    #[test]
    fn str_map() {
        let data = BTreeMap::from([("a", "A"), ("b", "B"), ("c", "C"), ("d", "D"), ("e", "E")]);

        let mut serializer = CborSerializer::new(Vec::new());

        data.try_for_each_event(&mut data.get_state(), |event| serializer.write(event))
            .unwrap();

        assert_eq!(
            serializer.out,
            hex::decode("a56161614161626142616361436164614461656145").unwrap()
        );
        assert_eq!(serializer.enc.stack, &[]);
    }
}
