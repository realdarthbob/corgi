use std::collections::HashMap;

use bytes::BytesMut;

use crate::protocol::{
    codec::PackageChunkCodec,
    types::{CallId, Package, PackageChunk, RpcError},
};

#[derive(Default)]
pub(crate) struct Parser {
    chunks: HashMap<CallId, Vec<PackageChunk>>,
    codec: PackageChunkCodec,
}

impl Parser {
    pub(crate) fn apply(&mut self, data: &[u8]) -> Result<Option<Package>, RpcError> {
        if let Some(call_id) = self.feed(data)? {
            let package = self.build_package(call_id);
            return Ok(Some(package));
        }

        Ok(None)
    }

    fn feed(&mut self, data: &[u8]) -> Result<Option<CallId>, RpcError> {
        let chunk = self.codec.decode(data)?;
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

    fn build_package(&mut self, call_id: CallId) -> Package {
        let package_chunks = self.chunks.remove(&call_id).unwrap();
        let payload = package_chunks
            .iter()
            .map(|p| p.payload())
            .fold(BytesMut::new(), |mut acc, value| {
                acc.extend_from_slice(value);
                acc
            })
            .freeze();

        Package::new(call_id, payload)
    }
}
