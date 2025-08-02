use anyhow::anyhow;
use std::{
    fs::File,
    io::{BufReader, Bytes},
};

#[derive(Debug)]
pub(crate) struct ByteStream {
    pub(crate) bytes: Bytes<BufReader<File>>,
}

impl ByteStream {
    pub(crate) fn next(&mut self) -> anyhow::Result<u8> {
        Ok(self.bytes.next().ok_or(anyhow!("unexpected EOF"))??)
    }
    pub(crate) fn maybe_next(&mut self) -> anyhow::Result<Option<u8>> {
        self.bytes.next().transpose().map_err(anyhow::Error::from)
    }
}
