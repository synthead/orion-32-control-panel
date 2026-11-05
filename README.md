# Open source Orion 32 Control Panel

This is an open source rewrite of the Orion 32 control panel, written in Rust with egui!  The goal of this project is to offer a fast, robust, and modern control panel for those that love their Orion 32.  It's written to be cross-platform, and Linux is supported, too!

The Orion 32 was released in 2012, and is still an excellent audio interface.  It has 32 line-level, balanced analog inputs and outputs on Tascam DB-25 connectors, operates at 192 KHz at 24 bit, and has excellent audio quality.  It also supports 64-channel MADI ins/outs, 2 8-channel ADAT ins, 2 8-channel ADAT outs, and S/PDIF in/out, all in a tiny 1U 19" rack.  It also has a very stable clock with its drift compensated by a literal oven, and supports a 10M external clock sync.

Not only is it great hardware, but from a software perspective, it is also excellent.  The Orion 32 audio is USB standards-compliant, so it will work nearly everywhere.  The control panel is also configured over HID, which is also standards-compliant.  On Linux, everything is supported directly from the kernel without any binary drivers.  The only missing piece to making this interface fully cross-platform is its control panel, which was only offered as proprietary software.  The open source control panel in this repository fills this gap, and lets you operate your interface on whatever system you choose!

## Screenshots

Here are some screenshots of this control panel at work!

<table>
  <tr>
    <td><img width="100%" alt="Image" src="https://github.com/user-attachments/assets/5526b043-6bf6-4e29-a090-f1235f2cea03" /></td>
    <td><img width="100%" alt="Image" src="https://github.com/user-attachments/assets/4bf66610-86c6-4bce-8197-009a125553ef" /></td>
    <td><img width="100%" alt="Image" src="https://github.com/user-attachments/assets/51e8c434-3c13-4878-b939-53adfc9d6c58" /></td>
  </tr>
  <tr>
    <td><img width="100%" alt="Image" src="https://github.com/user-attachments/assets/8df4fb4b-7a80-4e9e-a698-5a63c745be57" /></td>
    <td><img width="100%" alt="Image" src="https://github.com/user-attachments/assets/3cc52910-76ad-481b-bb55-84ba0a1eda58" /></td>
    <td><img width="100%" alt="Image" src="https://github.com/user-attachments/assets/7607977b-f0d4-441f-b943-ae0e06f301ca" /></td>
  </tr>
</table>

## Downloading and installing

Head to [the latest release](https://github.com/synthead/orion-32-control-panel/releases/latest) and download the binary for your operating system, then run it.  The control panel is a single, self-contained binary built by Rust, so no installation is necessary.

Note: On Windows, a modal will appear about how Windows has "protected" you from running the control panel without saying why, which makes it appear to be a suspicious program.  In reality, the binary is unsigned, and signing it with a Microsoft-issued CA to make the warning go away costs money.  If you downloaded it from a release on this repository, you can simply ignore this warning once, and it shouldn't bother you again.

## Improvements

- Open source!
- Cross-platform!  There is finally a control panel for Linux!
- Starts instantly, and queries the interface way, _way_ faster.
- Respects your privacy.  The original control panel phones home with your serial number every time it starts.  Yuck.
- Does not care if your device is activated.  The original control panel denies access to the interface until the user registers for an account and submits a mandatory form where all fields are required, which includes invasive questions like who they bought it from and how much they paid for it.
- Actually _works_.  The original launcher malfunctions with a message to check your internet connection.  The control panel data is no longer hosted, and requires manual intervention to acquire this data to be functional.
- Does not hassle you with interactive administrator permissions to start a Windows service every time it starts.
- Connects directly to your interface.  The original control panel connects to a network server that must always be running.
- Long-standing routing and AD phase bugs have been fixed.  The original control panel still has bugs where active routes are sometimes erroneously presented as muted, and AD phases are sometimes erroneously displayed as the wrong state.
- VU meters have peak indicators!  The appearance is now idential to the meters on the physical screen.
- Knobs and sliders update smoothly.  The original control panel only updated occasionally when moving, which made audibly chunky changes.
- Persists unique route labels per interface.  The original control panel kept the same labels across different interfaces.
- When more than one interface is connected, the last-used interface is selected when the control panel starts.
- Interface can still be programmed while in standby.  The power button simply disables the display and mutes the audio, and does not turn the interface off.  The original control panel pretends that it is off by disabling the control panel, where this version permits programming in standby.

## What works

- Multiple interface selection.
- VU meters, and VU meter sources.
- Clock source and frequency.
- Line out adjustment.
- Identical route programming behavior, including selecting and dragging routes, and editing route labels.
- Signal indicators.
- Loading and saving native o32r files.
- Persisting label names when closing and opening the control panel.
- Undo and redo support when editing routes.
- Selecting and saving presets.
- Standby support.
- Full mixer support.
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
- Firmware updates.  New updates are not being released, so it can be updated once in Windows, if needed.
- "How to" button.  This links to a YouTube playlist that has been removed on the original control panel.
- News feed.  Polling this feed does not respect your privacy, and the news feed is not online anymore.
- Windows-only ASIO options.  These have been moved to driver settings in the system tray, but are still visible as permanently-disabled settings in the original control panel.
- Multiple window support.  The original control panel could detach the routing, mixer, and AD phase settings as separate windows, which is cumbersome and not very useful.
