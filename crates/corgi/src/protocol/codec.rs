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

use crate::protocol::types::{ChunkHeader, Envelope, PackageChunk, RpcError};

/// CHUNK_HEADER_SIZE indicates protocol chunk header size, where call_id, chunk index, total
/// chunks and paylaod len is stored.
const CHUNK_HEADER_SIZE: usize = 16;

/// MAX_ARGUMENTS_COUNT indicates RPC function maxiumum arguments count
const MAX_ARGUMENTS_COUNT: usize = 16;

/// MAX_ARGUMENTS_SIZE indicates RPC function maximum arguments size which is equals to 16MB
const MAX_ARGUMENT_SIZE: usize = 16 * 1024 * 1024;

/// MAX_FUNCTION_NAME_SIZE indicates RPC function name length which must not exceed 65536
const MAX_FUNCTION_NAME_SIZE: usize = u16::MAX as usize;

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
            return Err(RpcError::ChunkHeaderSizeConstraintViolation);
        }

        let len = bytes[12..16]
            .try_into()
            .map(u32::from_le_bytes)
            .map_err(|_| RpcError::Decode)?;

        if bytes.len() < CHUNK_HEADER_SIZE + len as usize {
            return Err(RpcError::ChunkHeaderSizeConstraintViolation);
        }

        let call_id = bytes[..8]
            .try_into()
            .map(u64::from_le_bytes)
            .map_err(|_| RpcError::Decode)?;

        let index = bytes[8..10]
            .try_into()
            .map(u16::from_le_bytes)
            .map_err(|_| RpcError::Decode)?;

        let total = bytes[10..12]
            .try_into()
            .map(u16::from_le_bytes)
            .map_err(|_| RpcError::Decode)?;

        let header = ChunkHeader::new(call_id, index, total, len);

        let payload_start = CHUNK_HEADER_SIZE;
        let payload_end = payload_start + len as usize;
        let payload = Bytes::copy_from_slice(&bytes[payload_start..payload_end + len as usize]);

        Ok(PackageChunk::new(header, payload))
    }
}

#[derive(Default, Clone)]
pub struct EnvelopeCodec;

impl EnvelopeCodec {
    pub fn encode(&self, value: Envelope) -> Result<Bytes, RpcError> {
        let fn_name = value.fn_name();
        let args = value.parameters();

        if fn_name.len() > MAX_FUNCTION_NAME_SIZE {
            return Err(RpcError::MaxFunctionNameConstraintViolation);
        }

        if args.len() > MAX_ARGUMENTS_COUNT {
            return Err(RpcError::MaxArgumentsConstraintViolation);
        }

        for arg in args {
            if arg.len() > MAX_ARGUMENT_SIZE {
                return Err(RpcError::MaxArgumentSizeConstraintViolation);
            }
        }

        // fn name + fn len + args count
        let mut capacity = 2 + fn_name.len() + 2;

        // Allocation for each argument
        for arg in args {
            capacity += 8 + arg.len();
        }

        let mut buf = BytesMut::with_capacity(capacity);

        buf.put_u16(fn_name.len() as u16);

        buf.extend_from_slice(fn_name);

        buf.put_u16(args.len() as u16);

        for arg in args {
            buf.put_u64(arg.len() as u64);
            buf.extend_from_slice(arg);
        }

        Ok(buf.freeze())
    }

    pub fn decode(&self, bytes: &[u8]) -> Result<Envelope, RpcError> {
        let mut cursor = 0;

        // Function name length
        if bytes.len() < 2 {
            return Err(RpcError::Decode);
        }

        let fn_len = bytes[cursor..cursor + 2]
            .try_into()
            .map(u16::from_le_bytes)
            .map_err(|_| RpcError::Decode)? as usize;

        if fn_len > MAX_FUNCTION_NAME_SIZE {
            return Err(RpcError::MaxFunctionNameConstraintViolation);
        }

        cursor += 2;

        // Function name
        if bytes.len() < fn_len + cursor {
            return Err(RpcError::Decode);
        }

        let fn_name = Bytes::copy_from_slice(&bytes[cursor..cursor + fn_len]);
        cursor += fn_len;

        if bytes.len() < cursor + 2 {
            return Err(RpcError::Decode);
        }

        let arg_count = bytes[cursor..cursor + 2]
            .try_into()
            .map(u16::from_le_bytes)
            .map_err(|_| RpcError::Decode)? as usize;

        cursor += 2;

        if arg_count > MAX_ARGUMENTS_COUNT {
            return Err(RpcError::MaxArgumentsConstraintViolation);
        }

        // Arguments
        let mut parameters = Vec::with_capacity(arg_count);

        for _ in 0..arg_count {
            if bytes.len() < cursor + 8 {
                return Err(RpcError::Decode);
            }

            let arg_len = bytes[cursor..cursor + 8]
                .try_into()
                .map(u64::from_le_bytes)
                .map_err(|_| RpcError::Decode)? as usize;

            cursor += 8;

            if arg_len > MAX_ARGUMENT_SIZE {
                return Err(RpcError::MaxArgumentSizeConstraintViolation);
            }

            if bytes.len() < cursor + arg_len {
                return Err(RpcError::Decode);
            }

            let arg = &bytes[cursor..cursor + arg_len];
            cursor += arg_len;

            parameters.push(Bytes::copy_from_slice(arg));
        }

        if cursor != bytes.len() {
            return Err(RpcError::GarbageBytes);
        }

        let envelope = Envelope::new(fn_name, parameters);

        Ok(envelope)
    }
}
