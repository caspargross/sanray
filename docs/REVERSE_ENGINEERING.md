# Reverse engineering notes

Source binary: `win_binaries/plate amplifier with APF F2311_V1.0.5_20180525.exe`
(despite the filename, this is the **active top / plate-amp DSP** control app —
the standalone subwoofer-amp app will be added later).

## What the binary is

- Win32 PE, built with an old Delphi (VCL). Uses the **JVCL** `JvHidController`
  component and the Win32 `HidD_*` / `HidP_*` API directly — so the PC talks
  to the DSP as a **USB HID device**, not a virtual COM port. The HID product
  string is literally `DSPSERIALSUSB08 (Serial=0000007)` (see strings dump).
- Also references `cbb_comport` / `cbb_comport2` (COM port pickers) and a
  `cbb_deviceid` combo box with **255 entries** ("1".."255"). Working theory:
  units can be daisy-chained on an RS-485 bus, and the HID-attached unit acts
  as a USB↔RS485 bridge, with a per-unit device/address ID distinguishing
  which chained unit a command targets. This matters a lot for us: our
  `Transport`/protocol layer should treat "device address" as a routing field
  independent of the physical USB connection, so one USB link can address
  several physically-chained DSPs — exactly the "manage several devices from
  one interface" requirement.
- UI skin libraries (Raize/TurboPower `RzXxx` components, "BlueGlass"/"DeepBlue"
  skins) are irrelevant to the protocol — purely cosmetic VCL third-party
  components, ignore them when reading the dumped forms.

## Method used so far: static extraction of the compiled DFM forms

The actual wire protocol (byte layout of the USB HID reports) is **not**
visible in strings or resources — that requires dynamic capture (see
"What's still needed" below). But a surprising amount of ground-truth detail
*is* recoverable statically, because Delphi compiles each form's designed
properties into a binary resource (`RT_RCDATA`, signature `TPF0`) baked into
the `.rsrc` section, and that resource records every control's design-time
properties **by name** — including combo box item lists, slider min/max,
default values, edit max-lengths, tab order, etc. That's effectively the
original developer's UI spec, minus the Pascal event-handler code.

Tools written for this (kept in `re/`):

- `re/dfm_parse.py` — a from-scratch parser for the binary `TPF0` DFM format.
  There is no official spec; this was reconstructed from a community write-up
  of the general grammar (is4code.blogspot.com, March 2022) plus the classic
  Delphi `TValueType` tag enum, then **debugged against the actual file**
  until every resource in this exe parses with **zero leftover bytes**
  (checked for all 11 `TPF0` resources, including the two big forms at
  848,460 and 321,168 bytes) — that's a strong correctness signal, not a
  guess. Two corrections were needed versus the naive spec:
  - Tag `0x0E` (`vaCollection`) items are prefixed by a marker byte: `0x00`
    ends the collection, `0x01` introduces one item (itself a plain
    name/value property list).
  - Tag `0x12` (`vaWString`) stores its length as a **character count**, not
    a byte count — read `len*2` bytes and decode as UTF-16LE.
  Run it as `python3 re/dfm_parse.py <file.bin> [--json]`.
- `re/extracted_f2311/*.bin` — every `RT_RCDATA` resource extracted from the exe via
  `pefile` (see inline script used; redo with `pefile` if resources need
  re-extracting). `TFORM1` = the connect/device-list/login shell window.
  `TFORM2` = the actual per-device DSP control panel (gain/delay/crossover/
  EQ/limiter/system tabs) — this is the one that matters for the protocol.

See `docs/DATA_MODEL.md` for what this extraction revealed about the actual
parameter set, ranges, and defaults.

`win_binaries/D10Q DSP amplifier software_V2.2.5_APF_20180508.exe` (the
subwoofer amp app) is the same Delphi/JVCL/HID codebase -- same
`DSPSERIALSUSB08` HID product string (different serial suffix), same form
shapes -- and parses cleanly with the same tool (`re/extracted_d10q/`). This
is good evidence the two device families share one wire protocol with
different channel/band counts, not two unrelated protocols. See
`docs/DATA_MODEL.md` for the (small) set of confirmed differences.

## What's still needed: the actual wire bytes

Static analysis cannot show what bytes go over the HID `SetFeature`/
`GetFeature`/write calls when, say, you drag the gain slider. For that we
need **dynamic capture**, and neither is available on this machine (`adula`)
right now:

- No `wine` installed, and this session's `sudo` has no usable password, so
  `apt install wine` (or `radare2`/`ghidra` for a real disassembly) needs to
  be run by the user directly (`!sudo apt install wine ghidra` etc.), not by
  Claude.
- No device plugged in (`lsusb` shows only the root hubs + a Bluetooth
  adapter) — even with Wine, there's nothing to talk to without the real
  active-top hardware attached, or a way to fake HID enumeration convincingly
  enough to fool `HidD_GetAttributes`/`HidD_GetFeature` calls.

Recommended next steps, in order of effort:

1. **Capture on real hardware, in whatever way is most convenient** — plug a
   top into a Windows machine (or this machine via Wine once installed) and
   sniff the USB traffic with Wireshark+usbmon (Linux) or USBPcap (Windows),
   while using the original app normally. This is the highest-confidence,
   lowest-effort path once hardware is accessible, and doesn't require
   understanding the Delphi binary at all.
2. Cross-check captured bytes against this document's data model (which
   field changed when you moved which slider) to reverse the framing.
3. Only if (1) isn't practical: static disassembly of the HID read/write call
   sites (Ghidra/radare2) to recover the packet-building code directly. Much
   slower; the strings/DFM work above already did most of the *useful* part
   of this (knowing the parameter set) without needing it.

Until real bytes are captured, `sanray-protocol`'s wire codec for the real
device is a stub (`codec::sanway_f2311`, marked `todo!()`), and the mock
device speaks a separate, explicitly-fake `codec::sanray_native` protocol so
the rest of the framework (transport abstraction, app logic, UI) can be built
and tested now, then pointed at the real codec once it's known.
