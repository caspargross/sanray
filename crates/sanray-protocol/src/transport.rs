//! Byte-level link to a device. Deliberately dumb: a `Transport` just moves
//! opaque reports in and out (mirroring the original app's USB HID
//! `SetFeature`/`GetFeature`/read/write calls); all protocol meaning lives
//! in a `Codec` on top.

use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("device not connected")]
    NotConnected,
    #[error("timed out waiting for a report")]
    Timeout,
    #[error("transport I/O error: {0}")]
    Io(String),
}

pub trait Transport: Send {
    /// Send one report (e.g. a HID feature/output report) to the device.
    fn send_report(&mut self, report: &[u8]) -> Result<(), TransportError>;

    /// Block for up to `timeout` waiting for one report from the device.
    fn recv_report(&mut self, timeout: Duration) -> Result<Vec<u8>, TransportError>;
}
