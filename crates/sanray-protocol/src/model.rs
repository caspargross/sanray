//! Device parameter model.
//!
//! Recovered from the original Delphi app's compiled UI forms (see
//! `docs/DATA_MODEL.md`) rather than from the real wire protocol, which is
//! still unknown (see `docs/REVERSE_ENGINEERING.md`). Ranges/defaults here
//! match what the original UI ships with; exact slider-to-physical-unit
//! scaling for gain/delay is still unconfirmed pending a real device.

use serde::{Deserialize, Serialize};

pub const INPUTS_PER_DEVICE: usize = 4;
pub const OUTPUTS_PER_DEVICE: usize = 4;

/// The two device families seen so far both expose 4 inputs / 4 outputs and
/// the same gain scale (DFM trackbar `Max=520`), but differ in per-channel
/// EQ band count and delay range (see docs/DATA_MODEL.md) -- so those are a
/// property of the model, not a global constant.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceModel {
    /// "plate amplifier with APF F2311" -- the active-top plate amp.
    PlateAmpF2311,
    /// "D10Q DSP amplifier" -- the subwoofer amp.
    D10qAmp,
}

impl DeviceModel {
    /// Per-channel parametric EQ band count.
    pub fn eq_bands(&self) -> usize {
        match self {
            DeviceModel::PlateAmpF2311 => 16,
            DeviceModel::D10qAmp => 8,
        }
    }
}

/// 1..=255: a unit's address on the shared bus behind one USB/HID link.
/// (The connect dialog offers 1-255; a separate "change address" combo box
/// elsewhere in the app offers 0-250 -- not yet reconciled against real
/// hardware, see docs/DATA_MODEL.md.)
pub type DeviceAddress = u8;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelKind {
    Input,
    Output,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelId {
    pub kind: ChannelKind,
    /// 0..=3
    pub index: u8,
}

impl ChannelId {
    pub fn input(index: u8) -> Self {
        Self { kind: ChannelKind::Input, index }
    }

    pub fn output(index: u8) -> Self {
        Self { kind: ChannelKind::Output, index }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilterFamily {
    Butterworth,
    LinkwitzRiley,
    Bessel,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrossoverSlope {
    Db12,
    Db18,
    Db24,
    Db48,
}

/// One of the 12 HPF/LPF filter types in the original UI
/// ("12db-Bworth" .. "48db-Bessel").
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CrossoverType {
    pub slope: CrossoverSlope,
    pub family: FilterFamily,
}

impl CrossoverType {
    pub const ALL: [CrossoverType; 12] = {
        use CrossoverSlope::*;
        use FilterFamily::*;
        [
            CrossoverType { slope: Db12, family: Butterworth },
            CrossoverType { slope: Db12, family: LinkwitzRiley },
            CrossoverType { slope: Db12, family: Bessel },
            CrossoverType { slope: Db18, family: Butterworth },
            CrossoverType { slope: Db18, family: LinkwitzRiley },
            CrossoverType { slope: Db18, family: Bessel },
            CrossoverType { slope: Db24, family: Butterworth },
            CrossoverType { slope: Db24, family: LinkwitzRiley },
            CrossoverType { slope: Db24, family: Bessel },
            CrossoverType { slope: Db48, family: Butterworth },
            CrossoverType { slope: Db48, family: LinkwitzRiley },
            CrossoverType { slope: Db48, family: Bessel },
        ]
    };

    /// Exact label text as shown in the original UI (e.g. "12db-Bworth").
    pub fn label(&self) -> &'static str {
        use CrossoverSlope::*;
        use FilterFamily::*;
        match (self.slope, self.family) {
            (Db12, Butterworth) => "12db-Bworth",
            (Db12, LinkwitzRiley) => "12db-LR",
            (Db12, Bessel) => "12db-Bessel",
            (Db18, Butterworth) => "18db-Bworth",
            (Db18, LinkwitzRiley) => "18db-LR",
            (Db18, Bessel) => "18db-Bessel",
            (Db24, Butterworth) => "24db-Bworth",
            (Db24, LinkwitzRiley) => "24db-LR",
            (Db24, Bessel) => "24db-Bessel",
            (Db48, Butterworth) => "48db-Bworth",
            (Db48, LinkwitzRiley) => "48db-LR",
            (Db48, Bessel) => "48db-Bessel",
        }
    }
}

impl Default for CrossoverType {
    fn default() -> Self {
        CrossoverType { slope: CrossoverSlope::Db12, family: FilterFamily::Butterworth }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EqBandType {
    Peaking,
    LowShelf6,
    LowShelf12,
    HighShelf6,
    HighShelf12,
    AllPass,
}

impl EqBandType {
    pub const ALL: [EqBandType; 6] = [
        EqBandType::Peaking,
        EqBandType::LowShelf6,
        EqBandType::LowShelf12,
        EqBandType::HighShelf6,
        EqBandType::HighShelf12,
        EqBandType::AllPass,
    ];

    /// Exact label as shown in the original UI (e.g. "LS 6dB").
    pub fn label(&self) -> &'static str {
        match self {
            EqBandType::Peaking => "PEQ",
            EqBandType::LowShelf6 => "LS 6dB",
            EqBandType::LowShelf12 => "LS 12dB",
            EqBandType::HighShelf6 => "HS 6dB",
            EqBandType::HighShelf12 => "HS 12dB",
            EqBandType::AllPass => "APF",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct EqBand {
    pub freq_hz: f32,
    pub gain_db: f32,
    pub q: f32,
    pub kind: EqBandType,
    pub bypass: bool,
}

impl Default for EqBand {
    fn default() -> Self {
        // matches band 0's shipped defaults in the original UI
        Self { freq_hz: 19.7, gain_db: 0.0, q: 1.0, kind: EqBandType::Peaking, bypass: false }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Crossover {
    pub freq_hz: f32,
    pub kind: CrossoverType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ChannelConfig {
    pub name: String,
    pub gain_db: f32,
    pub delay_ms: f32,
    pub mute: bool,
    pub hpf: Crossover,
    pub lpf: Crossover,
    pub eq: Vec<EqBand>,
}

impl ChannelConfig {
    pub fn new(eq_bands: usize) -> Self {
        Self {
            name: String::new(),
            gain_db: 0.0,
            delay_ms: 0.0,
            mute: false,
            hpf: Crossover { freq_hz: 19.7, kind: CrossoverType::default() },
            lpf: Crossover { freq_hz: 20100.0, kind: CrossoverType::default() },
            eq: vec![EqBand::default(); eq_bands],
        }
    }
}

/// `off` or -25..=-1 dBFS, 1 dB steps -- as offered by the limiter threshold
/// and noise gate dropdowns.
pub type ThresholdDb = Option<i8>;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum LimiterAttackMs {
    Half,
    One,
    Five,
    Ten,
    Twenty,
    Fifty,
}

impl LimiterAttackMs {
    pub fn as_ms(&self) -> f32 {
        match self {
            Self::Half => 0.5,
            Self::One => 1.0,
            Self::Five => 5.0,
            Self::Ten => 10.0,
            Self::Twenty => 20.0,
            Self::Fifty => 50.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum LimiterReleaseMs {
    Hundred,
    TwoHundred,
    FourHundred,
    SixHundred,
    EightHundred,
    OneSec,
    TwoSec,
    ThreeSec,
}

impl LimiterReleaseMs {
    pub fn as_ms(&self) -> u32 {
        match self {
            Self::Hundred => 100,
            Self::TwoHundred => 200,
            Self::FourHundred => 400,
            Self::SixHundred => 600,
            Self::EightHundred => 800,
            Self::OneSec => 1000,
            Self::TwoSec => 2000,
            Self::ThreeSec => 3000,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Limiter {
    pub threshold_db: ThresholdDb,
    pub attack: LimiterAttackMs,
    pub release: LimiterReleaseMs,
}

impl Default for Limiter {
    fn default() -> Self {
        Self {
            threshold_db: Some(-25),
            attack: LimiterAttackMs::Half,
            release: LimiterReleaseMs::Hundred,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OutputChannelConfig {
    pub channel: ChannelConfig,
    pub limiter: Limiter,
}

impl OutputChannelConfig {
    pub fn new(eq_bands: usize) -> Self {
        Self { channel: ChannelConfig::new(eq_bands), limiter: Limiter::default() }
    }
}

/// Full state of one physical unit (one device address).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DeviceState {
    pub model: DeviceModel,
    pub device_name: String,
    pub noise_gate_db: ThresholdDb,
    pub inputs: [ChannelConfig; INPUTS_PER_DEVICE],
    pub outputs: [OutputChannelConfig; OUTPUTS_PER_DEVICE],
}

impl DeviceState {
    pub fn new(model: DeviceModel) -> Self {
        let bands = model.eq_bands();
        Self {
            model,
            device_name: String::new(),
            noise_gate_db: None,
            inputs: std::array::from_fn(|_| ChannelConfig::new(bands)),
            outputs: std::array::from_fn(|_| OutputChannelConfig::new(bands)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crossover_labels_match_original_ui() {
        // Exact 12 strings from the "cbb_ip0hpf_type" combo box
        // (docs/DATA_MODEL.md), order as shipped.
        let expected = [
            "12db-Bworth", "12db-LR", "12db-Bessel",
            "18db-Bworth", "18db-LR", "18db-Bessel",
            "24db-Bworth", "24db-LR", "24db-Bessel",
            "48db-Bworth", "48db-LR", "48db-Bessel",
        ];
        let actual: Vec<&str> = CrossoverType::ALL.iter().map(|c| c.label()).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn eq_band_type_labels_match_original_ui() {
        let expected = ["PEQ", "LS 6dB", "LS 12dB", "HS 6dB", "HS 12dB", "APF"];
        let actual: Vec<&str> = EqBandType::ALL.iter().map(|t| t.label()).collect();
        assert_eq!(actual, expected);
    }
}
