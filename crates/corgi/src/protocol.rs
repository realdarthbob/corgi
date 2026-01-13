use std::{any::TypeId, net::SocketAddr, sync::Arc};

use bytes::Bytes;
use futures::future::BoxFuture;

use crate::codec::BincodeCodec;

#[derive(Debug)]
struct IncomingPackage {
    local_addr: SocketAddr,
    peer_addr: SocketAddr,
    payload: Bytes,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub type_id: TypeId,
}

type Handler =
    dyn Fn(Bytes, BincodeCodec) -> BoxFuture<'static, Result<Bytes, RpcError>> + Send + Sync;

#[derive(Clone)]
pub struct RpcFunction {
    pub name: &'static str,
    pub params: Vec<Param>,
    pub return_type: TypeId,
    pub handler: Arc<Handler>,
}

#[derive(Debug)]
pub enum RpcError {
    Decode,
    Encode,
}
