//! A codec for talking to [`sanray_mock`](../../sanray-mock)'s simulated
//! device. **This is not the real Sanway wire protocol** -- that's still
//! unreverse-engineered (see `docs/REVERSE_ENGINEERING.md`). It exists so
//! the rest of the framework (device model, app logic, UI) can be built and
//! exercised end-to-end today; swap in a real codec later without touching
//! anything above the `Codec` trait.
//!
//! Each `Transport` report is one `bincode`-encoded [`WireMsg`]; framing is
//! left to the transport (the mock transport is message-based already, so
//! no length-prefixing is needed here).

use serde::{Deserialize, Serialize};

use crate::codec::{Codec, CodecError, DecodedFrame};
use crate::model::{ChannelId, DeviceAddress, DeviceState};
use crate::param::{Param, ParamValue};

#[derive(Serialize, Deserialize, Debug, Clone)]
enum WireMsg {
    Set { address: DeviceAddress, channel: ChannelId, param: Param, value: ParamValue },
    GetState { address: DeviceAddress },
    StateSnapshot { address: DeviceAddress, state: DeviceState },
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SanrayNativeCodec;

impl Codec for SanrayNativeCodec {
    fn encode_set(
        &self,
        address: DeviceAddress,
        channel: ChannelId,
        param: Param,
        value: &ParamValue,
    ) -> Vec<u8> {
        bincode::serialize(&WireMsg::Set { address, channel, param, value: value.clone() })
            .expect("WireMsg::Set is always serializable")
    }

    fn encode_get_state(&self, address: DeviceAddress) -> Vec<u8> {
        bincode::serialize(&WireMsg::GetState { address })
            .expect("WireMsg::GetState is always serializable")
    }

    fn decode(&self, frame: &[u8]) -> Result<DecodedFrame, CodecError> {
        let msg: WireMsg = bincode::deserialize(frame).map_err(|_| CodecError::Unrecognized)?;
        match msg {
            WireMsg::Set { address, channel, param, value } => {
                Ok(DecodedFrame::ParamChanged { address, channel, param, value })
            }
            WireMsg::StateSnapshot { address, state } => {
                Ok(DecodedFrame::StateSnapshot { address, state })
            }
            WireMsg::GetState { .. } => Err(CodecError::Unrecognized),
        }
    }
}

/// Also exposed so `sanray-mock` can decode requests and encode responses
/// using the exact same wire format, without duplicating it.
pub fn encode_state_snapshot(address: DeviceAddress, state: &DeviceState) -> Vec<u8> {
    bincode::serialize(&WireMsg::StateSnapshot { address, state: state.clone() })
        .expect("WireMsg::StateSnapshot is always serializable")
}

pub fn decode_get_state(frame: &[u8]) -> Option<DeviceAddress> {
    match bincode::deserialize::<WireMsg>(frame).ok()? {
        WireMsg::GetState { address } => Some(address),
        _ => None,
    }
}
