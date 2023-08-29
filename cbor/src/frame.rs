use bytes::{BufMut, BytesMut};
use sud_core::Event;
use tokio_util::codec::Encoder;

use crate::CborEncoder;

impl<'a> Encoder<Event<'a>> for CborEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: Event<'a>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.write(item, dst.writer())
    }
}
