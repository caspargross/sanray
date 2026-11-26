//! Addressing a single settable parameter on a device, independent of how
//! it's eventually encoded on the wire.

use serde::{Deserialize, Serialize};

use crate::model::{CrossoverType, EqBandType, LimiterAttackMs, LimiterReleaseMs, ThresholdDb};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Param {
    Gain,
    Delay,
    Mute,
    Name,
    HpfFreq,
    HpfType,
    LpfFreq,
    LpfType,
    EqFreq(u8),
    EqGain(u8),
    EqQ(u8),
    EqType(u8),
    EqBypass(u8),
    /// Output channels only.
    LimiterThreshold,
    /// Output channels only.
    LimiterAttack,
    /// Output channels only.
    LimiterRelease,
}

impl Param {
    /// Whether this parameter only makes sense on an output channel
    /// (the limiter doesn't exist on inputs in the original UI).
    pub fn output_only(&self) -> bool {
        matches!(self, Param::LimiterThreshold | Param::LimiterAttack | Param::LimiterRelease)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ParamValue {
    F32(f32),
    Bool(bool),
    Text(String),
    Crossover(CrossoverType),
    EqType(EqBandType),
    Threshold(ThresholdDb),
    Attack(LimiterAttackMs),
    Release(LimiterReleaseMs),
}
