use crate::core::DriverError;
use std::{
    net::{SocketAddr, TcpStream},
    time::Duration,
};

/// Connects to a service on this machine only. Short timeouts: a local server either answers at
/// once or is not running.
pub(super) fn connect_loopback(port: u16) -> Result<TcpStream, DriverError> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))
        .map_err(|e| DriverError::Unavailable(format!("127.0.0.1:{port}: {e}")))?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    stream.set_write_timeout(Some(Duration::from_secs(3)))?;
    stream.set_nodelay(true)?;
    Ok(stream)
}
