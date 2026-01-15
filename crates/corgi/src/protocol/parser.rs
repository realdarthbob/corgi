use std::collections::HashMap;

use bytes::{Bytes, BytesMut};

use crate::protocol::{
    codec::{EnvelopeCodec, PackageChunkCodec},
    types::{CallId, PackageChunk, RpcCall, RpcError},
};

#[derive(Default)]
pub(crate) struct Parser {
    chunks: HashMap<CallId, Vec<PackageChunk>>,
    chunk_codec: PackageChunkCodec,
    envelope_codec: EnvelopeCodec,
}

impl Parser {
    pub(crate) fn apply(&mut self, data: &[u8]) -> Result<Option<RpcCall>, RpcError> {
        if let Some(call_id) = self.feed(data)? {
            let bytes = self.build_package(call_id);
            let envelope = self.envelope_codec.decode(&bytes)?;
            let call = RpcCall::new(call_id, envelope);
            return Ok(Some(call));
        }

        Ok(None)
    }

    fn feed(&mut self, data: &[u8]) -> Result<Option<CallId>, RpcError> {
        let chunk = self.chunk_codec.decode(data)?;
        let total = chunk.header().total() as usize;
        let call_id = chunk.header().call_id();
        let package_chunks = self
            .chunks
            .entry(chunk.header().call_id())
            .or_insert_with(|| {
                let mut chunks = Vec::with_capacity(chunk.header().total() as usize);
                chunks.push(chunk);
                chunks
            });

        if total == package_chunks.len() {
            package_chunks.sort();
            return Ok(Some(call_id));
        }

        Ok(None)
    }

    fn build_package(&mut self, call_id: CallId) -> Bytes {
        let package_chunks = self.chunks.remove(&call_id).unwrap();
        package_chunks
            .iter()
            .map(|p| p.payload())
            .fold(BytesMut::new(), |mut acc, value| {
                acc.extend_from_slice(value);
                acc
            })
            .freeze()
    }
}
