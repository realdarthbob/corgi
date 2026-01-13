use std::collections::HashMap;

use crate::protocol::RpcFunction;

#[derive(Default)]
pub struct Container {
    functions: HashMap<&'static str, RpcFunction>,
}

impl Container {
    pub fn register(mut self, function: RpcFunction) -> Self {
        let _ = self.functions.entry(function.name).or_insert(function);
        self
    }
}
