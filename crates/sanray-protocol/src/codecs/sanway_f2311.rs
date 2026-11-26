//! Stub for the real device protocol ("plate amplifier with APF F2311" /
//! the D10Q amp -- both speak USB HID as `DSPSERIALSUSB08`, see
//! `docs/REVERSE_ENGINEERING.md`, and are suspected to share one wire
//! format). Not implemented: the actual HID report byte layout is unknown
//! until it's captured from real hardware (static analysis of the Delphi
//! binary only recovered the *parameter model*, in `docs/DATA_MODEL.md` --
//! not the bytes). Fill this in once a capture exists.

use crate::codec::{Codec, CodecError, DecodedFrame};
use crate::model::{ChannelId, DeviceAddress};
use crate::param::{Param, ParamValue};

#[derive(Debug, Default, Clone, Copy)]
pub struct SanwayF2311Codec;

impl Codec for SanwayF2311Codec {
    fn encode_set(
        &self,
        _address: DeviceAddress,
        _channel: ChannelId,
        _param: Param,
        _value: &ParamValue,
    ) -> Vec<u8> {
        todo!("real HID report format not yet reverse engineered, see docs/REVERSE_ENGINEERING.md")
    }

    fn encode_get_state(&self, _address: DeviceAddress) -> Vec<u8> {
        todo!("real HID report format not yet reverse engineered, see docs/REVERSE_ENGINEERING.md")
    }

    fn decode(&self, _frame: &[u8]) -> Result<DecodedFrame, CodecError> {
        Err(CodecError::NotImplemented(
            "real HID report format not yet reverse engineered, see docs/REVERSE_ENGINEERING.md",
        ))
    }
}
