use core::fmt;
use std::net::SocketAddr;

use bytes::Bytes;
use tokio::io;

#[derive(Debug)]
pub struct ChunkHeader {
    call_id: u64,
    index: u16,
    total: u16,
    len: u32,
}

impl ChunkHeader {
    pub fn new(call_id: u64, index: u16, total: u16, len: u32) -> Self {
        Self {
            call_id,
            index,
            total,
            len,
        }
    }

    pub fn call_id(&self) -> u64 {
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

impl fmt::Display for ChunkHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ChunkHeader(call_id={}, index={}, total={}, len={})",
            self.call_id, self.index, self.total, self.len
        )
    }
}

#[derive(Debug)]
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
pub struct IncomingPackage {
    local_address: SocketAddr,
    peer_address: SocketAddr,
    payload: Bytes,
}

impl IncomingPackage {
    pub fn new(local_address: SocketAddr, peer_address: SocketAddr, payload: Bytes) -> Self {
        IncomingPackage {
            local_address,
            peer_address,
            payload,
        }
    }
}

impl fmt::Display for IncomingPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IncomingPackage(local_address={}, peer_address={}, payload=Bytes[{}])",
            self.local_address,
            self.peer_address,
            self.payload.len()
        )
    }
}

#[derive(Debug)]
pub enum RpcError {
    Decode,
    Encode,
    SocketBinding(io::Error),
    LocalAddress(io::Error),
}
