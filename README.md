<img src="docs/SISR.svg" align="right" width="128"/>
<br />

[![Build Status](https://github.com/alia5/SISR/actions/workflows/snapshots.yml/badge.svg)](https://github.com/alia5/SISR/actions/workflows/snapshots.yml)
[![License: GPL-3.0](https://img.shields.io/github/license/alia5/SISR)](https://github.com/alia5/SISR/blob/main/LICENSE.txt)
[![Release](https://img.shields.io/github/v/release/alia5/SISR?include_prereleases&sort=semver)](https://github.com/alia5/SISR/releases)
[![Issues](https://img.shields.io/github/issues/alia5/SISR)](https://github.com/alia5/SISR/issues)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/alia5/SISR/pulls)
[![Downloads](https://img.shields.io/github/downloads/alia5/SISR/total?logo=github)](https://github.com/alia5/SISR/releases)
[![Discord](https://img.shields.io/discord/368823110817808384?logo=discord&logoColor=white&label=Discord&color=%23535fe5
)](https://discord.gg/hs34MtcHJY)


# SISR ✂️

**S**team **I**nput **S**ystem **R**edirector

SISR (pronounced "scissor") redirects Steam Input configurations to the system level (localhost or network).  

SISR takes controllers it receives from Steam (via Steam Input)
and forwards them as emulated but more compatible controllers (indistinguishable from real hardware) to the OS.

It can be used to circumvent issues with games and applications that
do not support Steam Input or otherwise pose challenges, like (but not limited to):

- Games with aggressive anti-cheat systems
- Emulators
- Windows Store games/apps
- Games with broken Steam Input support

All while still providing you the full feature set of SteamInput!

SISR can also be used to "tunnel"/forward Steam Input configurations over the network to other machines, including Keyboard/Mouse.  
This makes it possible to use devices like a Steam Deck as a dedicated controller without the need to stream the entire game.

The emulated controllers (and Keyboard/Mouse) are indistinguishable from real hardware and show up at system level.  
SISR achieves this by utilizing [VIIPER](https://github.com/Alia5/VIIPER) (requires **USBIP**).  
Unlike its predecessor [GlosSI](https://github.com/Alia5/GlosSI), it does not use the unmaintained [ViGEm](https://github.com/ViGEm/ViGEmBus) driver.

> [!WARNING]
**Highly experimental work in progress.** Everything is subject to change and may or may not work.  
Expect bugs, crashes, and missing features.

> [!IMPORTANT]
You **do** get the full functionality of SteamInput, _without having to launch your games from Steam_  
For this Steam must still be running in the background, though.   
**Alternatively** SISR also has a **no-Steam mode**, but it isn't really the intended use-case.

> [!CAUTION]
If you are a Media outlet / Youtuber or whateverelse with *significant reach*, and intend cover this software (aside from just mentioning it), **consider talking to me first**  
You are not required to, but I'd greatly appreciate it.  <br />  
  -The software is an active WIP, not ready for wide usage, with a bigger update in the next few days.  
 -I want to avoid people with significant reach stating false information.    
 -If something is unclear, I'm happy to help and/or improve my documentation  

## ✨🛣️ Features / Roadmap

- Full SteamInput featureset while emulating compatible controllers (indistinguishable from real hardware)
  - Xbox360 _or_
  - DualShock 4
- **Non Steam Mode*
-  Xbox 360 controller emulation
- Multi-platform support (Windows, Linux)
   Multiple operation modes
    - Standalone background service (To be improved)
    - Steam overlay window mode
-  PS4 controller emulation
- Networked operation across computers
  - Use devices like a SteamDeck as dedicated controller without streaming the whole game/display
- ~~🚧 Xbox One controller emulation~~
- ~~🚧 Generic controller emulation~~
- 🚧 Gyro Passthrough
- 🚧 Bundling multiple devices into a single controller
- 🚧 Automatic HidHide integration

## Documentation / FAQ / Help

Read the [documentation](https://alia5.github.io/SISR/)!

## 📝 Contributing

PRs welcome!
See [GitHub Issues](https://github.com/Alia5/SISR/issues) for open tasks.

Do note that I never written Rust before this project, and I discovered that I dislike Rust very much.  
And it shows, the code is a hot mess, but I do not care for a 90% useless toy project.  

## 📄 License

```license
SISR - Steam Input System Redirector

Copyright (C) 2025-2026 Peter Repukat

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
```




