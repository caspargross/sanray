//! Smoke test / demo for the sanray framework: spawns the mock device,
//! drives it through `sanray_protocol::Device` exactly like a real HID
//! device would be driven, and prints the round trip. This is the
//! "framework with an artificial DSP USB device to test our software
//! implementation" deliverable -- not a real CLI yet.

use std::collections::HashMap;

use sanray_protocol::codecs::sanray_native::SanrayNativeCodec;
use sanray_protocol::model::{ChannelId, CrossoverSlope, CrossoverType, DeviceModel, FilterFamily};
use sanray_protocol::param::ParamValue;
use sanray_protocol::{Device, Param};

const LEFT_TOP: u8 = 1;
const RIGHT_TOP: u8 = 2;

fn main() {
    let transport = sanray_mock::spawn(HashMap::new(), DeviceModel::PlateAmpF2311);
    let mut device = Device::new(transport, SanrayNativeCodec);

    for &addr in &[LEFT_TOP, RIGHT_TOP] {
        let state = device.get_state(addr).expect("mock should answer GetState");
        println!(
            "device {addr}: input0 gain={:.1}dB mute={} | output0 limiter.threshold={:?}",
            state.inputs[0].gain_db, state.inputs[0].mute, state.outputs[0].limiter.threshold_db
        );
    }

    println!("\nsetting left top: mute input0, gain -6dB, 48dB Linkwitz-Riley HPF at 80Hz on output0 (kick-bass)...");
    let kick_bass = ChannelId::output(0);
    device
        .set_param(LEFT_TOP, ChannelId::input(0), Param::Mute, ParamValue::Bool(true))
        .unwrap();
    device
        .set_param(LEFT_TOP, ChannelId::input(0), Param::Gain, ParamValue::F32(-6.0))
        .unwrap();
    device
        .set_param(
            LEFT_TOP,
            kick_bass,
            Param::HpfType,
            ParamValue::Crossover(CrossoverType {
                slope: CrossoverSlope::Db48,
                family: FilterFamily::LinkwitzRiley,
            }),
        )
        .unwrap();
    device.set_param(LEFT_TOP, kick_bass, Param::HpfFreq, ParamValue::F32(80.0)).unwrap();

    for &addr in &[LEFT_TOP, RIGHT_TOP] {
        let state = device.get_state(addr).expect("mock should answer GetState");
        println!(
            "device {addr}: input0 gain={:.1}dB mute={} | output0 HPF={} @ {:.1}Hz",
            state.inputs[0].gain_db,
            state.inputs[0].mute,
            state.outputs[0].channel.hpf.kind.label(),
            state.outputs[0].channel.hpf.freq_hz
        );
    }
    println!("\n(right top is untouched, as expected -- each device address has independent state)");
}
