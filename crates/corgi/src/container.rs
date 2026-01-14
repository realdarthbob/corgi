use bytes::Bytes;
use std::{any::TypeId, collections::HashMap, sync::Arc};

use futures::future::BoxFuture;

use crate::protocol::{codec::BincodeCodec, types::RpcError};

#[derive(Debug, Clone)]
pub struct Param {
    pub name: &'static str,
    pub type_id: TypeId,
}

type Handler =
    dyn Fn(Bytes, BincodeCodec) -> BoxFuture<'static, Result<Bytes, RpcError>> + Send + Sync;

#[derive(Clone)]
pub struct RpcFunction {
    pub name: &'static str,
    pub params: Vec<Param>,
    pub return_type: Option<TypeId>,
    pub handler: Arc<Handler>,
}

#[derive(Default)]
pub struct Container {
    functions: HashMap<&'static str, &'static RpcFunction>,
}

impl Container {
    pub fn register(mut self, function: &'static RpcFunction) -> Self {
        let _ = self.functions.entry(function.name).or_insert(function);
        self
    }
}
