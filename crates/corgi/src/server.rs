use std::net::SocketAddr;

use bytes::BytesMut;
use tokio::net::UdpSocket;

use crate::{
    Container,
    protocol::types::{IncomingPackage, RpcError},
};

const UDP_CHUNK_SIZE: usize = 1200;

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

    pub async fn start(&self) -> Result<(), RpcError> {
        let mut buf = BytesMut::with_capacity(UDP_CHUNK_SIZE);
        loop {
            let local_address = self
                .connection
                .local_addr()
                .map_err(RpcError::LocalAddress)?;

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

            let package = IncomingPackage::new(local_address, peer_address, buf.split().freeze());
            tracing::trace!("Received package from address: {peer_address}. Package: {package}");
        }
    }
}
