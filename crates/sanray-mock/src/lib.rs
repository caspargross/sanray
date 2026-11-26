//! An in-process "fake DSP USB device": a [`sanray_protocol::Transport`]
//! backed by channels, with a background thread that holds simulated
//! [`DeviceState`] per device address and understands the
//! `sanray_native` codec well enough to apply `Set` commands and answer
//! `GetState` requests.
//!
//! This does **not** speak the real Sanway USB HID protocol (which isn't
//! reverse engineered yet, see `docs/REVERSE_ENGINEERING.md`) -- it's here
//! so the rest of the stack (device model, app logic, UI) has something to
//! run against today. Once the real protocol is known, a real `Transport`
//! (USB HID) + the real `Codec` replace this pair without touching
//! anything built on top of `sanray_protocol::Device`.

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

use sanray_protocol::codec::{Codec, DecodedFrame};
use sanray_protocol::codecs::sanray_native::{self, SanrayNativeCodec};
use sanray_protocol::model::{ChannelId, ChannelKind, DeviceAddress, DeviceModel, DeviceState};
use sanray_protocol::param::{Param, ParamValue};
use sanray_protocol::transport::{Transport, TransportError};

pub struct MockTransport {
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
}

impl Transport for MockTransport {
    fn send_report(&mut self, report: &[u8]) -> Result<(), TransportError> {
        self.tx.send(report.to_vec()).map_err(|_| TransportError::NotConnected)
    }

    fn recv_report(&mut self, timeout: Duration) -> Result<Vec<u8>, TransportError> {
        match self.rx.recv_timeout(timeout) {
            Ok(frame) => Ok(frame),
            Err(RecvTimeoutError::Timeout) => Err(TransportError::Timeout),
            Err(RecvTimeoutError::Disconnected) => Err(TransportError::NotConnected),
        }
    }
}

/// Starts a background "mock device" thread simulating the given initial
/// per-address states (an address not in `initial` is created on first
/// touch as a fresh `default_model` device), and returns the client-side
/// `Transport` to talk to it with.
pub fn spawn(
    initial: HashMap<DeviceAddress, DeviceState>,
    default_model: DeviceModel,
) -> MockTransport {
    let (client_tx, server_rx) = mpsc::channel::<Vec<u8>>();
    let (server_tx, client_rx) = mpsc::channel::<Vec<u8>>();

    thread::spawn(move || serve(initial, default_model, server_rx, server_tx));

    MockTransport { tx: client_tx, rx: client_rx }
}

fn serve(
    mut state: HashMap<DeviceAddress, DeviceState>,
    default_model: DeviceModel,
    rx: Receiver<Vec<u8>>,
    tx: Sender<Vec<u8>>,
) {
    let codec = SanrayNativeCodec;
    while let Ok(frame) = rx.recv() {
        if let Some(address) = sanray_native::decode_get_state(&frame) {
            let snapshot = state
                .entry(address)
                .or_insert_with(|| DeviceState::new(default_model))
                .clone();
            let resp = sanray_native::encode_state_snapshot(address, &snapshot);
            if tx.send(resp).is_err() {
                return;
            }
            continue;
        }
        if let Ok(DecodedFrame::ParamChanged { address, channel, param, value }) =
            codec.decode(&frame)
        {
            let dev = state.entry(address).or_insert_with(|| DeviceState::new(default_model));
            apply_param(dev, channel, param, value);
        }
    }
}

fn apply_param(state: &mut DeviceState, channel: ChannelId, param: Param, value: ParamValue) {
    use sanray_protocol::model::{Limiter, LimiterAttackMs, LimiterReleaseMs, ThresholdDb};

    if param.output_only() {
        if channel.kind != ChannelKind::Output {
            return;
        }
        let limiter: &mut Limiter = &mut state.outputs[channel.index as usize].limiter;
        match (param, value) {
            (Param::LimiterThreshold, ParamValue::Threshold(t)) => {
                let _: ThresholdDb = t;
                limiter.threshold_db = t;
            }
            (Param::LimiterAttack, ParamValue::Attack(a)) => {
                let _: LimiterAttackMs = a;
                limiter.attack = a;
            }
            (Param::LimiterRelease, ParamValue::Release(r)) => {
                let _: LimiterReleaseMs = r;
                limiter.release = r;
            }
            _ => {}
        }
        return;
    }

    let cfg = match channel.kind {
        ChannelKind::Input => &mut state.inputs[channel.index as usize],
        ChannelKind::Output => &mut state.outputs[channel.index as usize].channel,
    };
    apply_channel_param(cfg, param, value);
}

fn apply_channel_param(
    cfg: &mut sanray_protocol::model::ChannelConfig,
    param: Param,
    value: ParamValue,
) {
    match (param, value) {
        (Param::Gain, ParamValue::F32(v)) => cfg.gain_db = v,
        (Param::Delay, ParamValue::F32(v)) => cfg.delay_ms = v,
        (Param::Mute, ParamValue::Bool(v)) => cfg.mute = v,
        (Param::Name, ParamValue::Text(v)) => cfg.name = v,
        (Param::HpfFreq, ParamValue::F32(v)) => cfg.hpf.freq_hz = v,
        (Param::HpfType, ParamValue::Crossover(v)) => cfg.hpf.kind = v,
        (Param::LpfFreq, ParamValue::F32(v)) => cfg.lpf.freq_hz = v,
        (Param::LpfType, ParamValue::Crossover(v)) => cfg.lpf.kind = v,
        (Param::EqFreq(b), ParamValue::F32(v)) => cfg.eq[b as usize].freq_hz = v,
        (Param::EqGain(b), ParamValue::F32(v)) => cfg.eq[b as usize].gain_db = v,
        (Param::EqQ(b), ParamValue::F32(v)) => cfg.eq[b as usize].q = v,
        (Param::EqType(b), ParamValue::EqType(v)) => cfg.eq[b as usize].kind = v,
        (Param::EqBypass(b), ParamValue::Bool(v)) => cfg.eq[b as usize].bypass = v,
        // mismatched param/value pairing; the mock is deliberately
        // tolerant rather than panicking on a malformed set.
        _ => {}
    }
}
