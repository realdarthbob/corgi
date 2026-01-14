//! Binary encoding and decoding logic.
//!
//! This module defines:
//! - the on-wire binary format for `PackageChunk`
//! - serialization helpers for RPC payloads
//! - strict bounds-checked decoding of incoming packets
//!
//! All parsing logic in this module is designed to be
//! deterministic, panic-free, and safe for untrusted UDP input.
use bytes::{BufMut, Bytes, BytesMut};
use wincode::{SchemaReadOwned, SchemaWrite};

use crate::protocol::types::{ChunkHeader, PackageChunk, RpcError};

#[derive(Default, Clone)]
pub struct BincodeCodec;

impl BincodeCodec {
    pub fn encode<T: SchemaWrite<Src = T>>(&self, value: &T) -> Result<Bytes, RpcError> {
        wincode::serialize(value)
            .map(Bytes::from)
            .map_err(|_| RpcError::Encode)
    }

    pub fn decode<T: SchemaReadOwned<Dst = T>>(&self, bytes: Bytes) -> Result<T, RpcError> {
        wincode::deserialize(&bytes).map_err(|_| RpcError::Decode)
    }
}

/// CHUNK_HEADER_SIZE indicates protocol chunk header size, where call_id, chunk index, total
/// chunks and paylaod len is stored.
const CHUNK_HEADER_SIZE: usize = 16;

///
/// Binary wire format for a single RPC message chunk.
///
/// Layout (byte offsets):
///
/// ```text
/// 0        8       10      12      16
/// |---------|-------|-------|-------|-------------------|
/// | call_id | index | total | len   | payload bytes...  |
/// | u64     | u16   | u16   | u32   | len bytes         |
/// ```
///
/// Field descriptions:
///
/// - `call_id`
///   A unique identifier for the RPC call or message.
///   All chunks belonging to the same logical message share the same `call_id`.
///
/// - `index`
///   Zero-based index of this chunk within the message.
///
/// - `total`
///   Total number of chunks expected for the message.
///
/// - `len`
///   Length (in bytes) of the payload that immediately follows the header.
///
/// - `payload`
///   Raw binary payload bytes. The payload is opaque to the transport layer
///   and is interpreted by higher-level protocol logic.
///
/// Notes:
///
/// - All integer fields are encoded in **little-endian** order.
/// - The header size is fixed (`CHUNK_HEADER_SIZE = 16` bytes).
/// - The codec performs strict bounds checking to prevent malformed or
///   truncated packets from causing panics.
///
#[derive(Default, Clone)]
pub struct PackageChunkCodec;

impl PackageChunkCodec {
    pub fn encode(&self, value: PackageChunk) -> Result<Bytes, RpcError> {
        let header = value.header();
        let mut bytes = BytesMut::with_capacity(CHUNK_HEADER_SIZE + header.payload_len() as usize);

        bytes.put_u64(header.call_id());
        bytes.put_u16(header.index());
        bytes.put_u16(header.total());
        bytes.put_u32(header.payload_len());

        bytes.extend_from_slice(value.payload());

        Ok(bytes.freeze())
    }

    pub fn decode(&self, bytes: &[u8]) -> Result<PackageChunk, RpcError> {
        if bytes.len() < CHUNK_HEADER_SIZE {
            return Err(RpcError::Decode);
        }

        let len: [u8; 4] = bytes[12..16].try_into().map_err(|_| RpcError::Decode)?;
        let len = u32::from_le_bytes(len);

        if bytes.len() < CHUNK_HEADER_SIZE + len as usize {
            return Err(RpcError::Decode);
        }

        let call_id: [u8; 8] = bytes[..8].try_into().map_err(|_| RpcError::Decode)?;
        let call_id = u64::from_le_bytes(call_id);

        let index: [u8; 2] = bytes[8..10].try_into().map_err(|_| RpcError::Decode)?;
        let index = u16::from_le_bytes(index);

        let total: [u8; 2] = bytes[10..12].try_into().map_err(|_| RpcError::Decode)?;
        let total = u16::from_le_bytes(total);

        let header = ChunkHeader::new(call_id, index, total, len);

        let payload_start = CHUNK_HEADER_SIZE;
        let payload_end = payload_start + len as usize;
        let payload = Bytes::copy_from_slice(&bytes[payload_start..payload_end + len as usize]);

        Ok(PackageChunk::new(header, payload))
    }
}
