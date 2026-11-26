use std::time::Duration;

use crate::codec::{Codec, CodecError, DecodedFrame};
use crate::model::{ChannelId, ChannelKind, DeviceAddress, DeviceState};
use crate::param::{Param, ParamValue};
use crate::transport::{Transport, TransportError};

#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error(transparent)]
    Codec(#[from] CodecError),
    #[error("{0:?} is only valid on an output channel")]
    InvalidParamForChannel(Param),
    #[error("expected a state snapshot, got something else")]
    UnexpectedFrame,
}

/// A device reachable through `transport`, speaking whatever wire format
/// `codec` implements. One `transport` may carry several chained device
/// addresses (see docs/DATA_MODEL.md); `address` is passed per call rather
/// than fixed at construction so one `Device` can drive a whole chain.
pub struct Device<T, C> {
    transport: T,
    codec: C,
}

impl<T: Transport, C: Codec> Device<T, C> {
    pub fn new(transport: T, codec: C) -> Self {
        Self { transport, codec }
    }

    pub fn set_param(
        &mut self,
        address: DeviceAddress,
        channel: ChannelId,
        param: Param,
        value: ParamValue,
    ) -> Result<(), DeviceError> {
        if param.output_only() && channel.kind != ChannelKind::Output {
            return Err(DeviceError::InvalidParamForChannel(param));
        }
        let frame = self.codec.encode_set(address, channel, param, &value);
        self.transport.send_report(&frame)?;
        Ok(())
    }

    pub fn get_state(&mut self, address: DeviceAddress) -> Result<DeviceState, DeviceError> {
        let frame = self.codec.encode_get_state(address);
        self.transport.send_report(&frame)?;
        let resp = self.transport.recv_report(Duration::from_millis(500))?;
        match self.codec.decode(&resp)? {
            DecodedFrame::StateSnapshot { state, .. } => Ok(state),
            _ => Err(DeviceError::UnexpectedFrame),
        }
    }
}
