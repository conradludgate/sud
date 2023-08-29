use bytes::{BufMut, BytesMut};
use sud_core::Event;
use tokio_util::codec::Encoder;

use crate::JsonEncoder;

impl<'a> Encoder<Event<'a>> for JsonEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: Event<'a>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.write(item, dst.writer())
    }
}
