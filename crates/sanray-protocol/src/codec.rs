//! Translating between the domain model (`model`/`param`) and raw bytes on
//! a `Transport`. Kept as its own trait so the *same* device model and the
//! *same* calling code work against the mock device now and the real
//! hardware once its wire format is known -- only the `Codec` impl changes.

use crate::model::{ChannelId, DeviceAddress, DeviceState};
use crate::param::{Param, ParamValue};

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("not enough bytes to decode a frame")]
    Truncated,
    #[error("unrecognized frame")]
    Unrecognized,
    #[error("not yet reverse engineered: {0}")]
    NotImplemented(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DecodedFrame {
    ParamChanged { address: DeviceAddress, channel: ChannelId, param: Param, value: ParamValue },
    StateSnapshot { address: DeviceAddress, state: DeviceState },
}

pub trait Codec {
    fn encode_set(
        &self,
        address: DeviceAddress,
        channel: ChannelId,
        param: Param,
        value: &ParamValue,
    ) -> Vec<u8>;

    fn encode_get_state(&self, address: DeviceAddress) -> Vec<u8>;

    fn decode(&self, frame: &[u8]) -> Result<DecodedFrame, CodecError>;
}
