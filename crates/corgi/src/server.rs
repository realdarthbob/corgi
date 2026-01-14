use core::fmt;
use std::net::SocketAddr;

use bytes::BytesMut;
use tokio::net::UdpSocket;

const UDP_CHUNK_SIZE: usize = 1200;

use crate::{
    Container,
    protocol::{
        parser::Parser,
        types::{Package, RpcError},
    },
};

#[derive(Debug)]
struct RpcCall {
    local_address: SocketAddr,
    peer_address: SocketAddr,
    package: Package,
}

impl RpcCall {
    fn new(local_address: SocketAddr, peer_address: SocketAddr, package: Package) -> Self {
        Self {
            local_address,
            peer_address,
            package,
        }
    }
}

impl fmt::Display for RpcCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RpcCall(local_address={}, peer_address={}, package={})",
            self.local_address, self.peer_address, self.package
        )
    }
}

pub struct RpcServer<'a, T> {
    container: &'a Container,
    connection: T,
}

impl<'a> RpcServer<'a, UdpSocket> {
    pub async fn create_udp(
        container: &'a Container,
        address: SocketAddr,
    ) -> Result<Self, RpcError> {
        tracing::trace!("Creating RpcServer. establishing UDP socket binding on address {address}");
        let socket = UdpSocket::bind(address)
            .await
            .map_err(RpcError::SocketBinding)?;
        let instance = Self {
            container,
            connection: socket,
        };
        tracing::debug!("Successfully established UDP socket binding on address {address}.");
        Ok(instance)
    }

    pub fn local_address(&self) -> Result<SocketAddr, RpcError> {
        let address = self
            .connection
            .local_addr()
            .map_err(RpcError::LocalAddress)?;

        Ok(address)
    }

    pub async fn start(&self) -> Result<(), RpcError> {
        let mut buf = BytesMut::with_capacity(UDP_CHUNK_SIZE);
        let mut parser = Parser::default();
        let local_address = self.local_address()?;

        loop {
            tracing::trace!("Waiting for accepting RPC call for address {local_address}");

            buf.clear();
            buf.resize(UDP_CHUNK_SIZE, 0);
            let (len, peer_address) = match self.connection.recv_from(&mut buf).await {
                Ok(data) => data,
                Err(error) => {
                    tracing::error!("Failed to receive from socket connection. Error: {error}");
                    continue;
                }
            };
            buf.truncate(len);

            if let Some(package) = parser.apply(&buf)? {
                let rpc_call = RpcCall::new(local_address, peer_address, package);
                tracing::trace!("Received RpcCall {rpc_call}");
            }
        }
    }
}
