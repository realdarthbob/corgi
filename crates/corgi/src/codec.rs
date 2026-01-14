use bytes::Bytes;
use wincode::{SchemaReadOwned, SchemaWrite};

use crate::protocol::RpcError;

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
