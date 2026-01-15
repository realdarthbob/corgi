use core::fmt;
use std::cmp;

use bytes::Bytes;
use tokio::io;

pub type CallId = u64;

#[derive(Debug, Eq)]
pub struct ChunkHeader {
    call_id: CallId,
    index: u16,
    total: u16,
    len: u32,
}

impl ChunkHeader {
    pub fn new(call_id: CallId, index: u16, total: u16, len: u32) -> Self {
        Self {
            call_id,
            index,
            total,
            len,
        }
    }

    pub fn call_id(&self) -> CallId {
        self.call_id
    }

    pub fn index(&self) -> u16 {
        self.index
    }

    pub fn total(&self) -> u16 {
        self.total
    }

    pub fn payload_len(&self) -> u32 {
        self.len
    }
}

impl PartialEq for ChunkHeader {
    fn eq(&self, other: &Self) -> bool {
        self.call_id == other.call_id && self.index == other.index
    }
}

impl Ord for ChunkHeader {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (self.call_id, self.index).cmp(&(other.call_id, other.index))
    }
}

impl PartialOrd for ChunkHeader {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for ChunkHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ChunkHeader(call_id={}, index={}, total={}, len={})",
            self.call_id, self.index, self.total, self.len
        )
    }
}

#[derive(Debug, Eq)]
pub struct PackageChunk {
    header: ChunkHeader,
    payload: Bytes,
}

impl PackageChunk {
    pub fn new(header: ChunkHeader, payload: Bytes) -> Self {
        Self { header, payload }
    }

    pub fn header(&self) -> &ChunkHeader {
        &self.header
    }

    pub fn payload(&self) -> &Bytes {
        &self.payload
    }
}

impl Ord for PackageChunk {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.header().cmp(other.header())
    }
}

impl PartialOrd for PackageChunk {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.header().partial_cmp(&other.header())
    }
}

impl PartialEq for PackageChunk {
    fn eq(&self, other: &Self) -> bool {
        self.header == other.header
    }
}

impl fmt::Display for PackageChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PackageChunk(header={}, payload=Bytes[{}])",
            self.header,
            self.payload.len()
        )
    }
}

#[derive(Debug)]
pub struct Envelope {
    fn_name: Bytes,
    parameters: Vec<Bytes>,
}

impl Envelope {
    pub fn new(fn_name: Bytes, parameters: Vec<Bytes>) -> Self {
        Self {
            fn_name,
            parameters,
        }
    }

    pub fn fn_name(&self) -> &Bytes {
        &self.fn_name
    }

    pub fn parameters(&self) -> &Vec<Bytes> {
        &self.parameters
    }
}

impl fmt::Display for Envelope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Envelope(fn_name=Byyes[{}], parameters={})",
            self.fn_name.len(),
            self.parameters().len(),
        )
    }
}

#[derive(Debug)]
pub struct RpcCall {
    call_id: CallId,
    envelope: Envelope,
}

impl RpcCall {
    pub fn new(call_id: CallId, envelope: Envelope) -> Self {
        RpcCall { call_id, envelope }
    }
}

impl fmt::Display for RpcCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RpcCall(call_id={}, envelope={})",
            self.call_id, self.envelope,
        )
    }
}

#[derive(Debug)]
pub enum RpcError {
    Decode,
    Encode,
    MaxFunctionNameConstraintViolation,
    MaxArgumentsConstraintViolation,
    MaxArgumentSizeConstraintViolation,
    ChunkHeaderSizeConstraintViolation,
    GarbageBytes,
    SocketBinding(io::Error),
    LocalAddress(io::Error),
}
