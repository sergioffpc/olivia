# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-11-03

First release.

### Added

- `olivia`: headless game loop with a timer that reports the frame rate ([#48], [#49]).
- `olivia`: restart the game when its source changes ([#95]).
- `olivia-math`: math types for 2D games ([#44]).
- `olivia-window`: run the game loop in a window ([#50]).
- `olivia-filesystem`: `Filesystem` handle over the game's source and save area ([#51]), and listing the files of a directory ([#79]).
- `olivia-graphics`: clear the screen ([#52]), draw sprites through a camera ([#53]), depth, batching, filters and blending ([#54]), named depths ([#56]), text ([#57]), and recovery from surface and device loss ([#58]).
- `olivia-input`: query the keyboard and mouse in `update` ([#59]).
- `olivia-physics`: simulate a game-owned `World` ([#60]), query it and draw its diagnostics ([#70]).
- `olivia-audio`: play sounds and cues ([#72]), and play them from positions in space ([#73]).
- `olivia`: own the ECS world, its physics and its drawing ([#80]).
- `olivia-animation`: motion capture for 2D characters ([#92]).
- Mocap capture page that records animations from the webcam ([#100]).
- Astronaut Cat example game, with shooting stars across the background ([#96]).
- Native development on macOS ([#99]).

### Fixed

- `olivia-window`: show the window title on Wayland ([#55]).

[Unreleased]: https://github.com/sergioffpc/olivia/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/sergioffpc/olivia/releases/tag/v1.0.0
[#44]: https://github.com/sergioffpc/olivia/pull/44
[#48]: https://github.com/sergioffpc/olivia/pull/48
[#49]: https://github.com/sergioffpc/olivia/pull/49
[#50]: https://github.com/sergioffpc/olivia/pull/50
[#51]: https://github.com/sergioffpc/olivia/pull/51
[#52]: https://github.com/sergioffpc/olivia/pull/52
[#53]: https://github.com/sergioffpc/olivia/pull/53
[#54]: https://github.com/sergioffpc/olivia/pull/54
[#55]: https://github.com/sergioffpc/olivia/pull/55
[#56]: https://github.com/sergioffpc/olivia/pull/56
[#57]: https://github.com/sergioffpc/olivia/pull/57
[#58]: https://github.com/sergioffpc/olivia/pull/58
[#59]: https://github.com/sergioffpc/olivia/pull/59
[#60]: https://github.com/sergioffpc/olivia/pull/60
[#70]: https://github.com/sergioffpc/olivia/pull/70
[#72]: https://github.com/sergioffpc/olivia/pull/72
[#73]: https://github.com/sergioffpc/olivia/pull/73
[#79]: https://github.com/sergioffpc/olivia/pull/79
[#80]: https://github.com/sergioffpc/olivia/pull/80
[#92]: https://github.com/sergioffpc/olivia/pull/92
[#95]: https://github.com/sergioffpc/olivia/pull/95
[#96]: https://github.com/sergioffpc/olivia/pull/96
[#99]: https://github.com/sergioffpc/olivia/pull/99
[#100]: https://github.com/sergioffpc/olivia/pull/100
