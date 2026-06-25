# Emulator adapters

The Rust `EmulatorAdapter` trait defines identity, executable names, supported systems, installation detection, and launch command construction. Commands carry an executable path and an argument vector.

Implemented adapters:

| Adapter | Detection | Argument builder | Full real-emulator test |
| --- | --- | --- | --- |
| RetroArch | Yes + manual picker | `-L`, core, content | Fake-emulator process integration |
| PCSX2 | Yes | fullscreen + image | Synthetic unit test |
| Dolphin | Yes | batch + exec | Not run |
| PPSSPP | Yes | fullscreen + content | Not run |
| DuckStation | Yes | batch/fullscreen + content | Not run |
| RPCS3 | Yes | content path | Not run |
| Cemu | Yes | content path | Not run |

Version probing, managed installation manifests, firmware validation, per-profile save discovery, and diagnostic launch tests are next implementation slices.

RetroArch uses a separate core path and never modifies a user's global configuration. The managed data layout includes config, cores, core-info, shaders, saves, states, screenshots, system, playlists, and logs.
