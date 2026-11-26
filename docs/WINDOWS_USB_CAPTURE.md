# Capturing the real protocol on native Windows

## Installing Claude Code on Windows first (optional, if you want it there)

Native install, no Node.js needed — **don't run as Administrator**:

- PowerShell: `irm https://claude.ai/install.ps1 | iex`
- CMD: `curl -fsSL https://claude.ai/install.cmd -o install.cmd && install.cmd && del install.cmd`
- (or WinGet: `winget install Anthropic.ClaudeCode`, but it won't auto-update —
  you'd periodically need `winget upgrade Anthropic.ClaudeCode`)

Then open a **new** terminal, `cd` into a project folder, and run `claude` —
first run prompts a browser login (Claude Pro/Max/Team/Enterprise, or an API
key). `claude --version` / `claude doctor` to sanity-check the install.

A few Windows-specific notes: install **Git for Windows** too (claude.ai
uses Git Bash for its Bash tool if present, otherwise falls back to
PowerShell — without either it won't start). If `claude` isn't recognized
after install, open a new terminal, or add `%USERPROFILE%\.local\bin` to
your PATH. A leftover Claude *Desktop* install can shadow `claude.exe` on
PATH with the desktop app — if `claude` opens the wrong thing, that's why.

**Important**: this won't be *this* conversation continued — a fresh
`claude` on Windows starts cold, with no memory of this session. That's
fine for the capture procedure below (it's simple enough to just follow by
hand, or hand this file to a fresh Windows Claude Code session as its
briefing — point it at `docs/WINDOWS_USB_CAPTURE.md` in this repo, or just
paste it in). The actual protocol *analysis* of whatever `.pcapng` you
capture is better done back here, where all the prior RE context already
is — but either works.

## Why native Windows

Both Sanway apps are 32-bit and this machine's Wine setup (via Docker,
since `apt install wine32` is blocked by an unrelated package conflict, see
`docs/REVERSE_ENGINEERING.md`) gets far enough to enumerate the device as
USB-HID but then fails inside Wine's own HID stack (`fixme:hid:pdo_ioctl
Unsupported ioctl 0xb01e2`, repeatable, not a UI/config issue) before the
app ever shows "Connected". Native Windows sidesteps this category of
problem entirely, so this is the more reliable path to a real capture.

## What you'll need on the Windows side

- The two exe files from `win_binaries/` (copy them over however's easiest —
  USB stick, cloud storage, email to yourself; they're small, ~3MB each).
- **Wireshark for Windows**, installed *with the USBPcap component ticked*
  during setup (it's an optional checkbox in the Wireshark Windows
  installer — without it, there's no USB capture capability at all, only
  network interfaces show up).
- The DSP unit, connected via USB.

## Capture procedure

1. Install Wireshark (with USBPcap checked), reboot if the installer asks.
2. Plug in the DSP, then launch the original app (`D10Q DSP amplifier
   software_V2.2.5_APF_20180508.exe` or the plate-amp one, whichever matches
   the unit you connected). It should show **Connected**, not Disconnected —
   if it doesn't, something's actually wrong at the Windows/driver level too
   and that's worth knowing before going further.
3. Open Wireshark. In the interface list you should see one or more
   `USBPcapN` entries (one per USB root hub). If you're not sure which one,
   start one, and check Windows Device Manager for which USB Root Hub the
   device is under, or just capture on all of them (traffic volume is tiny
   for this kind of device, that's fine).
4. **Start the capture before you touch anything in the app.**
5. Perform this exact sequence, **one action at a time, with a ~3 second
   pause between each**, and keep a running note (even just scribbled) of
   the order you did them in — we'll correlate that against capture
   timestamps afterward. Suggested sequence (safe, all read-visible,
   reversible, nothing destructive):
   1. Just let it sit connected for ~5 seconds, untouched (baseline/
      keepalive traffic, if any).
   2. Toggle **Mute** on `In A` on, then off.
   3. Change **Gain** on `In A` by a distinct, easy-to-spot amount (e.g.
      drag to exactly -10.0 dB), note the exact value shown.
   4. Open `Out 1`'s **XOver**, change the **HPF Type** to something
      distinctive (e.g. `48db-LR`), and the **HPF freq** to `80.0`.
   5. In the same dialog's EQ table, change band 1's **Gain** to `-3.0` and
      **Q** to `2.0`.
   6. Open **Limiters**, change `Out 1` threshold to a different value
      (e.g. `-10dB`).
   7. Stop there — skip preset save/store and device-ID/address changes
      for this first pass; those write to the unit's non-volatile storage
      and we don't need them yet to get the core protocol.
6. **Stop the capture**, save it as `.pcapng` (File > Save As), and bring
   that file back (same USB stick / cloud storage route).

## What NOT to click, for now

Per the earlier safety discussion: avoid the preset Store buttons, the
"save/load 20-group to PC" buttons, the device name/ID "Change" button, and
anything resembling a firmware update — not because any of these are known
to be dangerous, but because we don't yet know what's behind them at the
byte level, and there's no known factory-reset path for this hardware if
something unexpected happens. Everything in the sequence above is a normal,
visible, reversible parameter change exactly like a user would make day to
day.

## After the capture

Bring the `.pcapng` file back (to this machine, or wherever you'll continue
from) along with your note of the action order. The analysis step — finding
which bytes correspond to which action, working out framing/checksums — is
pure offline work on the capture file, no device needed.
