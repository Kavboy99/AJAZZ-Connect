use crate::core::DriverError;
use std::io::{Read, Write};
use std::{
    net::{SocketAddr, TcpStream},
    time::Duration,
};

/// Connects to a service on this machine only. Short timeouts: a local server either answers at
/// once or is not running.
pub(super) fn connect_loopback(port: u16) -> Result<LoggedStream, DriverError> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let stream = crate::diagnostics::logging::io("tcp.connect", 0, || {
        TcpStream::connect_timeout(&address, Duration::from_secs(2))
            .map_err(|e| DriverError::Unavailable(format!("127.0.0.1:{port}: {e}")))
    })?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    stream.set_write_timeout(Some(Duration::from_secs(3)))?;
    stream.set_nodelay(true)?;
    Ok(LoggedStream(stream))
}

pub(super) struct LoggedStream(TcpStream);
impl Read for LoggedStream {
    fn read(&mut self, data: &mut [u8]) -> std::io::Result<usize> {
        crate::diagnostics::logging::stream_io("tcp.read", || self.0.read(data))
    }
}
impl Write for LoggedStream {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        crate::diagnostics::logging::stream_io("tcp.write", || self.0.write(data))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}
