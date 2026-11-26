# Parameter model (recovered from the DFM extraction)

Everything below was read directly off `TFORM2` (the device control panel)
via `re/dfm_parse.py`; defaults/ranges are the values the Delphi app itself
ships with. One physical unit exposes **4 input channels** (`ip0`-`ip3`) and
**4 output channels** (`op0`-`op3`). For our actual system (2x active 3-way
top + 1x sub amp), logical roles (sub / kick-bass / mid-high per side) get
mapped onto these physical in/out channels per device — that mapping is a
system-profile concern layered above this per-device model, not part of it.

## Per input channel (`ip{0..3}`)

- **Gain**: vertical trackbar, range `0..520`, default position `399`,
  displayed as `"0.0 dB"`. 520 steps for what's almost certainly a 0.1 dB
  step size over an asymmetric range (more cut available than boost) —
  **unconfirmed scale/offset**, needs a real-device check (move the slider
  to an extreme, read the displayed dB value, solve for scale/offset).
- **Delay**: vertical trackbar, range `0..8191` (= 2^13-1, i.e. a 13-bit raw
  value), default `0`, displayed as `"0.00 ms"`. Consistent with a raw
  sample-count delay line; at 48 kHz that's up to ~170 ms.
- **Mute**: a button/toggle per channel (`btn_ip{N}_mute`).
- **Channel name**: free text, max length 13 chars (`edt_ip{N}_chname`).
- **Crossover** (HPF *and* LPF, independently configurable):
  - Freq: free-text edit, e.g. default HPF `"19.7"` Hz, default LPF
    `"20100.0"` Hz (i.e. effectively off/full-range by default).
  - Type: 12-entry dropdown — all combinations of slope × filter family:
    `12db-Bworth, 12db-LR, 12db-Bessel, 18db-Bworth, 18db-LR, 18db-Bessel,
    24db-Bworth, 24db-LR, 24db-Bessel, 48db-Bworth, 48db-LR, 48db-Bessel`
    (Bworth = Butterworth, LR = Linkwitz-Riley).
- **Parametric EQ**: **16 bands** per input, each with:
  - Freq (free-text edit, Hz)
  - Gain (free-text edit, dB; default `"0.0"`)
  - Q (free-text edit; default `"1.0"`)
  - Type, 6-entry dropdown: `PEQ, LS 6dB, LS 12dB, HS 6dB, HS 12dB, APF`
    (parametric / low-shelf 6 or 12 dB-per-oct / high-shelf 6 or 12 dB-per-oct
    / all-pass filter)
  - Bypass: checkbox, per band

## Per output channel (`op{0..3}`)

Same Gain/Delay/Mute/Channel-name/Crossover/16-band-PEQ shape as inputs,
with two differences:

- **Delay** trackbar range is `0..960` instead of `0..8191` — outputs get a
  much smaller fine-trim delay range than inputs (inputs presumably carry
  the bulk of driver-alignment delay).
- Outputs additionally have a **3-stage limiter/compressor**, as three
  separate dropdowns (not free sliders):
  - Threshold: 26-entry, `-25dB, -24dB, ... -1dB, off`
  - Attack: 6-entry, `0.5ms, 1ms, 5ms, 10ms, 20ms, 50ms`
  - Release: 8-entry, `100ms, 200ms, 400ms, 600ms, 800ms, 1sec, 2sec, 3sec`

## System-level (per physical unit)

- **Connection**: COM-port picker + a **device/address ID dropdown, values
  1-255** (`cbb_deviceid` on the connect form) — strong evidence units are
  individually addressable on a shared bus (RS-485 daisy-chain off the
  USB-HID-attached unit), not each needing their own USB port. Important for
  our `Transport` design: "which physical USB handle" and "which device
  address on that link" are two different axes.
- **Presets**: save/load across **20 device-resident preset slots**
  (`Pre1:".."Pre20:"`), separate save and load dropdowns.
- **Channel copy**: copy all settings from one input to another, or one
  output to another (`cbb_incopy_src/dst`, `cbb_outcopy_src/dst`).
- **Noise gate**: dropdown, `off` or `-79` down to `-40` (dBFS threshold,
  1 dB steps), system-wide.
- **Security**: a 6-char numeric lock/unlock password, plus a separate
  "set new password" field (max 20 chars for the new value, 6 for confirm).
- **Device identity**: rename the device (max 15 chars), reassign its chain
  address (0-250 per `cbb_chaneID`, slightly different upper bound than the
  connect-form's 1-255 — not yet reconciled, probably just two independently
  authored combo boxes with similar but not-identically-reviewed ranges).

## The D10Q sub amp shares the same shape, with two differences

`win_binaries/D10Q DSP amplifier software_V2.2.5_APF_20180508.exe` (added
after the above was written) is HID-device-string-compatible
(`DSPSERIALSUSB08`, same component set, same form structure -- `TFORM1`/
`TFORM2` sizes are within a few percent of the plate-amp exe) and parses
cleanly with the same `re/dfm_parse.py`. Same 4 inputs / 4 outputs, same
HPF/LPF 12-slope-type set, same limiter dropdowns (threshold/attack/release
-- identical item lists), same gain trackbar (`Max=520`). Two confirmed
differences:

- **EQ bands per channel: 8, not 16** (`cbb_op0eq_type_00`..`07` only).
- **Delay trackbar ranges differ**: input `Max=4095` (vs. 8191 on the plate
  amp), output `Max=5055` (vs. 960). Interesting that here outputs get
  *more* delay range than inputs get on the plate amp, and more than inputs
  get here too -- plausibly because a sub amp's output stage is where
  driver-to-driver alignment actually happens, whereas the plate amp aligns
  at its input stage. Unconfirmed; matches the general pattern that delay
  budget allocation is a per-model, not a universal, constant.

This is why `sanray_protocol::model` makes EQ band count a per-`DeviceModel`
property (`DeviceModel::eq_bands()`) rather than a hardcoded constant --
learned from this diff, not assumed up front.

## Open questions for dynamic capture

- Exact gain slider → dB mapping (scale + offset; 520 steps, default at 399).
- Exact delay slider → ms mapping (0..8191 raw units; presumed sample count
  at some fixed internal rate — confirm the rate).
- Byte-level framing of the USB HID reports: report ID(s), command/parameter
  IDs per control, checksum/CRC if any, and how "device address" (1-255) is
  carried when a unit is mid-chain.
- Whether per-band EQ freq/gain/Q free-text edits have hidden clamped ranges
  enforced in code (not visible in the DFM) — e.g. plausible 20 Hz-20 kHz,
  ±18 dB, Q 0.1-10, but unconfirmed.
