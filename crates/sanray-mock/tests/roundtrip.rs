use std::collections::HashMap;

use sanray_protocol::codecs::sanray_native::SanrayNativeCodec;
use sanray_protocol::model::{
    ChannelId, CrossoverSlope, CrossoverType, DeviceModel, DeviceState, EqBandType, FilterFamily,
};
use sanray_protocol::param::ParamValue;
use sanray_protocol::{Device, Param};

#[test]
fn set_then_get_round_trips_across_independent_addresses() {
    let transport = sanray_mock::spawn(HashMap::new(), DeviceModel::PlateAmpF2311);
    let mut device = Device::new(transport, SanrayNativeCodec);

    let out0 = ChannelId::output(0);
    device
        .set_param(1, out0, Param::EqGain(3), ParamValue::F32(-4.5))
        .unwrap();
    device
        .set_param(
            1,
            out0,
            Param::EqType(3),
            ParamValue::EqType(EqBandType::HighShelf12),
        )
        .unwrap();
    device
        .set_param(
            1,
            out0,
            Param::LpfType,
            ParamValue::Crossover(CrossoverType {
                slope: CrossoverSlope::Db24,
                family: FilterFamily::LinkwitzRiley,
            }),
        )
        .unwrap();

    let state1 = device.get_state(1).unwrap();
    assert_eq!(state1.outputs[0].channel.eq[3].gain_db, -4.5);
    assert_eq!(state1.outputs[0].channel.eq[3].kind, EqBandType::HighShelf12);
    assert_eq!(state1.outputs[0].channel.lpf.kind.label(), "24db-LR");

    // a different address was never touched, so it must still be all-defaults
    let state2 = device.get_state(2).unwrap();
    assert_eq!(state2, DeviceState::new(DeviceModel::PlateAmpF2311));
}

#[test]
fn limiter_param_is_rejected_on_an_input_channel() {
    let transport = sanray_mock::spawn(HashMap::new(), DeviceModel::PlateAmpF2311);
    let mut device = Device::new(transport, SanrayNativeCodec);

    let err = device.set_param(
        1,
        ChannelId::input(0),
        Param::LimiterThreshold,
        ParamValue::Threshold(Some(-10)),
    );
    assert!(err.is_err(), "inputs have no limiter in the original UI");
}
