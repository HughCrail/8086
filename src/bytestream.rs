use std::io::{BufReader, ErrorKind, Read, Result, Seek};

#[derive(Debug)]
pub(crate) struct ByteStream<T> {
    pub(crate) reader: BufReader<T>,
}

impl<T: Read> ByteStream<T> {
    pub(crate) fn next(&mut self) -> Result<u8> {
        let mut buf = [0_u8; 1];
        match self.reader.read_exact(&mut buf) {
            Ok(_) => Ok(buf[0]),
            Err(e) => Err(e),
        }
    }
    pub(crate) fn maybe_next(&mut self) -> Result<Option<u8>> {
        match self.next() {
            Ok(r) => Ok(Some(r)),
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(e),
        }
    }
}
impl<T: Seek> ByteStream<T> {
    pub(crate) fn get_iptr(&mut self) -> Result<u64> {
        self.reader.stream_position()
    }
    // pub(crate) fn set_iptr(&mut self) -> Result<u64, Error> {
    //     self.reader.seek_relative()
    // }
}
