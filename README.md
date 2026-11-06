# Open-source Orion 32 Control Panel

This is an open-source rewrite of the Orion 32 control panel, written in Rust with egui!  The goal of this project is to offer a fast, robust, and modern control panel for those who love their Orion 32.  It's written to be cross-platform, and Linux is supported, too!

The Orion 32 was released in 2012, and is still an excellent audio interface.  It has 32 line-level, balanced analog inputs and outputs on Tascam DB-25 connectors, operates at 192 kHz at 24-bit, and has great audio quality.  It also supports 64-channel MADI ins/outs, two 8-channel ADAT ins, two 8-channel ADAT outs, and S/PDIF in/out, all in a tiny 1U 19" rack.  It also has a very stable clock with its drift compensated by a literal oven, and supports a 10M external clock sync.

Not only is it great hardware, but from a software perspective, it is also excellent.  The Orion 32's audio is USB Audio Class compliant, so it works nearly everywhere, and its control panel is configured over standard USB HID.  On Linux, everything is supported directly by the kernel without any binary drivers.  The only missing piece to making this interface fully cross-platform was its control panel, which was only offered as proprietary software.  The open-source control panel in this repository fills that gap and lets you operate your interface on whatever system you choose!

## Screenshots

Here are some screenshots of this control panel at work!

<table>
  <tr>
    <td><img width="100%" alt="Routing tab" src="https://github.com/user-attachments/assets/5526b043-6bf6-4e29-a090-f1235f2cea03" /></td>
    <td><img width="100%" alt="Mixer tab" src="https://github.com/user-attachments/assets/4bf66610-86c6-4bce-8197-009a125553ef" /></td>
    <td><img width="100%" alt="AD phases tab" src="https://github.com/user-attachments/assets/51e8c434-3c13-4878-b939-53adfc9d6c58" /></td>
  </tr>
  <tr>
    <td><img width="100%" alt="Settings modal" src="https://github.com/user-attachments/assets/8df4fb4b-7a80-4e9e-a698-5a63c745be57" /></td>
    <td><img width="100%" alt="About modal" src="https://github.com/user-attachments/assets/3cc52910-76ad-481b-bb55-84ba0a1eda58" /></td>
    <td><img width="100%" alt="Devices list" src="https://github.com/user-attachments/assets/7607977b-f0d4-441f-b943-ae0e06f301ca" /></td>
  </tr>
</table>

## Downloading and installing

Head to [the latest release](https://github.com/synthead/orion-32-control-panel/releases/latest) and download the binary for your operating system, then run it.  The control panel is a single, self-contained binary built in Rust, so no installation is necessary.  Keep the binary anywhere you'd like.

If you want to migrate your route labels from the original control panel, simply save an o32r file by clicking save from the original control panel, and load the o32r file from the new control panel.

### Windows users

Windows will show a "Windows protected your PC" warning when you first run the control panel, which makes it look like a suspicious program.  In reality, the binary is just unsigned, because code-signing certificates cost money.  If you downloaded it from a release on this repository, click "More info", then "Run anyway".  Windows may show the warning again for new releases.

### Linux users

On Linux, your user may not have read and write access to the Orion 32's HID node.  A udev rule may need to be added for the control panel to connect to your interface.  To fix this, add the following to `/etc/udev/rules.d/antelope-orion-32-hid.rules`:

```
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="23e5", ATTRS{idProduct}=="a040", GROUP="audio", MODE="0660"
```

This allows anyone in the `audio` group access to the Orion 32's HID node so the control panel will work for them.  You may need to add your user to the `audio` group, like so:

```
sudo gpasswd -a "$USER" audio
```

To apply the settings, restart your computer.

## Improvements

- Open source!
- Cross-platform!  There is finally a control panel for Linux!
- Starts instantly, and queries the interface way, _way_ faster.
- Respects your privacy.  The original control panel [phones home with your serial number](https://github.com/user-attachments/assets/2cc49ad6-c7d8-4646-8fa2-e62d66594c4e) every time it starts.  Yuck!
- Does not care if your device is activated.  The original control panel [denies access to the interface](https://github.com/user-attachments/assets/b928eaf0-4e0d-4691-869c-0e8265ca9101) until the user registers for an account and submits [a mandatory form where all fields are required](https://github.com/user-attachments/assets/b6dc4f71-4134-4916-82f6-76e114971a8d), which includes invasive questions like who they bought it from and how much they paid for it.
- Actually _works_.  The original launcher has shown [a message to check your internet connection](https://github.com/user-attachments/assets/40db4b51-ac39-406b-b5d4-642adc1fa5f9) for more than a year.  The control panel data appears to be no longer hosted, and requires manual intervention to obtain that data before it will work.
- Does not hassle you with [an interactive administrator permissions prompt](https://github.com/user-attachments/assets/66e7b258-fc90-4bbd-9cd0-d451bc62b0db) every time it starts.
- Connects directly to your interface.  The original control panel [connects to a network server](https://github.com/user-attachments/assets/2fea9af5-4119-4f73-9dc3-433ff719b634) that must always be running.
- Long-standing routing and AD phase bugs have been fixed.  The original control panel still has bugs where active routes are sometimes shown as muted, and AD phases are sometimes displayed in the wrong state.
- Hitting tab in a mixer pan or level field now jumps to the same field on the next channel, instead of alternating between pan and level as the original control panel did.
- VU meters have [peak indicators](https://github.com/user-attachments/assets/760af7db-0317-4a6a-b5e7-b743f8c64060)!  The appearance is now identical to the meters on the physical screen.
- Knobs and sliders update smoothly.  The original control panel only updated occasionally when moving, which made audibly chunky changes.
- Persists unique route labels per interface.  The original control panel kept the same labels across different interfaces.
- When more than one interface is connected, the last-used interface is selected when the control panel starts.
- The interface can still be [programmed while in standby](https://github.com/user-attachments/assets/9bf72619-b4a3-448c-ac8c-7a00ea3c37d0).  The power feature on the interface simply disables the display and mutes the audio, and does not turn the interface off.  The original control panel pretends that it is off by disabling the control panel, while this version permits programming in standby.

## Supported features

- Multiple interface selection.
- VU meters including log curve and peak indicators, and VU meter sources.
- Clock source and real-time clock frequency, including 10M atomic clock.
- Line out adjustment.
- Identical route programming behavior, including selecting and dragging routes, and editing route labels.
- Real-time MADI and ADAT channel count updates.
- Signal indicators.
- Loading and saving native o32r files.
- Persisting label names when closing and opening the control panel.
- Undo and redo support when editing routes.
- Selecting and saving presets.
- Standby support, including programming during standby.
- Full mixer support, with double-click pan resets and better tab movement.
- AD phases.
- All settings.
- All data in the about window.
- USB hot plugging is correctly handled.
- OS-appropriate configuration directory.

## Not implemented

- Any action that phones home.
- Windows service for networked control panel server.
- Client support to connect to networked control panel server.
- Administration tools to manage Windows service for control panel server.
- Animations when changing views.  This makes the UI immediate.
- Firmware updates.  New updates are not being released, so the interface can be updated once from Windows, if needed.
- "How to" button.  In the original control panel, this links to a YouTube playlist that has been removed.
- News feed.  Polling this feed does not respect your privacy, and the news feed is not online anymore.
- Windows-only ASIO options.  These have been moved to driver settings in the system tray, but are still visible as permanently disabled settings in the original control panel.
- Multiple windows.  The original control panel could detach the routing, mixer, and AD phase settings as separate windows, which is cumbersome and not very useful.
- Double-clicking a mixer level or the line out knob no longer resets it to a default value as the original control panel did. This prevents a sudden loud burst of audio from an accidental double-click.
