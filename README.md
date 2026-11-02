# Open source Orion 32 Control Panel

This is an open source rewrite of the Orion 32 control panel, written in Rust with egui!  The goal of this project is to offer a fast, robust, and modern control panel for those that love their Orion 32.  It's written to be cross-platform, and Linux is supported, too!

The Orion 32 was released in 2012, and is still an excellent audio interface.  It has 32 line-level, balanced analog inputs and outputs on Tascam DB-25 connectors, operates at 192 KHz at 24 bit, and has excellent audio quality.  It also supports 64-channel MADI ins/outs, 2 8-channel ADAT ins, 2 8-channel ADAT outs, and S/PDIF in/out, all in a tiny 1U 19" rack.  It also has a very stable clock with its drift compensated by a literal oven, and supports a 10M external clock sync.

Not only is it great hardware, but from a software perspective, it is also excellent.  The Orion 32 audio is USB standards-compliant, so it will work nearly everywhere.  The control panel is also configured over HID, which is also standards-compliant.  On Linux, everything is supported directly from the kernel without any binary drivers.  The only missing piece to making this interface fully cross-platform is its control panel, which was only offered as proprietary software.  The open source control panel in this repository fills this gap, and lets you operate your interface on whatever system you choose!

## Screenshots

Here are some screenshots of this control panel at work!

<table>
  <tr>
    <td><img width="100%" alt="Image" src="https://github.com/user-attachments/assets/d41c8168-8701-4b53-b804-716afc0a03a5" /></td>
    <td><img width="100%" alt="Image" src="https://github.com/user-attachments/assets/3e99f1da-30e9-480c-8581-aa8e6a43682f" /></td>
    <td><img width="100%" alt="Image" src="https://github.com/user-attachments/assets/8dde972d-6ef0-4287-a8a0-801d980db778" /></td>
  </tr>
  <tr>
    <td><img width="100%" alt="Image" src="https://github.com/user-attachments/assets/bb87fdb0-6508-477b-8407-546f15ea08c5" /></td>
    <td><img width="100%" alt="Image" src="https://github.com/user-attachments/assets/560753d0-c966-47a1-b9bd-e1d0059405fc" /></td>
    <td><img width="100%" alt="Image" src="https://github.com/user-attachments/assets/9644685b-3137-48d2-954d-1ce52b4eefb4" /></td>
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
- Does not care if your device is activated.  The original control panel refuses to work with devices until they register for an account and fill out a mandatory form, which includes invasive questions, including who they bought it from and how much they paid for it.
- Actually _works_.  The original launcher malfunctions with a message to check your internet connection.  The control panel data is no longer hosted, and requires manual intervention to acquire this data to be functional.
- Does not hassle you with interactive administrator permissions to start a Windows service every time it starts.
- Connects directly to your interface.  The original control panel connects to a network server that must always be running.
- Long-standing routing and AD phase bugs have been fixed.  The original control panel still has bugs where active routes are sometimes erroneously presented as muted, and AD phases are sometimes erroneously displayed as the wrong state.
- Knobs and sliders update smoothly.  The original control panel only updated occasionally when moving, which made audibly chunky changes.
- Interface can still be programmed while in standby.  The power button simply disables the display and mutes the audio, and does not turn the interface off.  The original control panel pretends that it is off by disabling the control panel, where this version permits programming in standby.

## What works

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
