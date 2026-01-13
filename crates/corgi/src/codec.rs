use bytes::Bytes;
use wincode::{SchemaReadOwned, SchemaWrite};

use crate::protocol::RpcError;

pub trait Codec: Send + Sync + 'static {
    fn encode<T: SchemaWrite<Src = T>>(&self, value: &T) -> Result<Bytes, RpcError>;
    fn decode<T: SchemaReadOwned<Dst = T>>(&self, bytes: Bytes) -> Result<T, RpcError>;
}

#[derive(Default)]
pub struct BincodeCodec;

impl Codec for BincodeCodec {
    fn encode<T: SchemaWrite<Src = T>>(&self, value: &T) -> Result<Bytes, RpcError> {
        wincode::serialize(value)
            .map(Bytes::from)
            .map_err(|_| RpcError::Encode)
    }

    fn decode<T: SchemaReadOwned<Dst = T>>(&self, bytes: Bytes) -> Result<T, RpcError> {
        wincode::deserialize(&bytes).map_err(|_| RpcError::Decode)
    }
}
