# Changelog

## 0.12.0 (unreleased)

### Changed

* Render using Vulkan. ([#131], [#136])

### Fixed

* Game mode selection UI events no longer enter an event channel loop. ([#134])
* Removed dependency cycles in crates. ([#135])

[#131]: https://gitlab.com/azriel91/autexousious/issues/131
[#134]: https://gitlab.com/azriel91/autexousious/issues/134
[#135]: https://gitlab.com/azriel91/autexousious/issues/135
[#136]: https://gitlab.com/azriel91/autexousious/issues/136

## 0.11.0 (2019-05-24)

### Added

* Objects have a 3 tick delay upon `Hit` interactions. ([#103])
* `hit_limit` determines the number of objects that the hitter can hit. ([#103])
* `repeat_delay` specifies number of ticks that must pass before an object can be `hit` by the same hitter. ([#103])
* Control input `transitions` can be specified per frame. ([#96])
* Added `--frame_rate` argument to control application frame rate. ([#117])
* Added `stand_attack_1` sequence. ([#117])
* Added `jump_attack` sequence. ([#117])
* Added `dash_attack` sequence. ([#117])
* Added `dazed` sequence. ([#118])
* `StunPoints` status attribute on `Characters`. ([#118])
* Characters switch to different sequences depending on their accumulated `StunPoints`. ([#118])
* Players can now return to game mode selection from the character selection UI. ([#110])
* Players can now return to character mode selection from the map selection UI. ([#120])
* Play a sound when a character hits another. ([#124])
* Play sounds in game mode selection UI. ([#127])
* Play sounds in character selection UI. ([#127])
* Play sounds in map selection UI. ([#127])

### Changed

* `Attack` and `Jump` no longer have hardcoded *hold* transitions. ([#96])
* ***Breaking:*** `hp_damage` is specified as part of `hit` interaction kind. ([#103])
* ***Breaking:*** `sp_damage` is specified as part of `hit` interaction kind. ([#103])
* ***Breaking:*** `physical` interaction kind is renamed to `hit`. ([#103])
* ***Breaking:*** `Interaction` kind determines fields applicable to the interaction. ([#103])
* ***Breaking:*** `stand_attack` is renamed to `stand_attack_0`. ([#117])
* Default `hit_repeat` delay is increased from 3 ticks to to 10 ticks. ([#117])
* Improved clarity of character selection UI. ([#110])
* Improved clarity of map selection UI. ([#120])
* Use control input for game mode selection UI. ([#122])
* `jump` and `dash` sequence velocities and gravity have been approximately halved. ([#128])
* Freeze character position when `FrameFreezeClock` is ongoing. ([#128])

### Fixed

* `Walk` sequence stays on first frame when an action button is held. ([#114])
* Character sprites now correctly render at their entity position. ([#129])

[#96]: https://gitlab.com/azriel91/autexousious/issues/96
[#103]: https://gitlab.com/azriel91/autexousious/issues/103
[#110]: https://gitlab.com/azriel91/autexousious/issues/110
[#114]: https://gitlab.com/azriel91/autexousious/issues/114
[#117]: https://gitlab.com/azriel91/autexousious/issues/117
[#118]: https://gitlab.com/azriel91/autexousious/issues/118
[#120]: https://gitlab.com/azriel91/autexousious/issues/120
[#122]: https://gitlab.com/azriel91/autexousious/issues/122
[#124]: https://gitlab.com/azriel91/autexousious/issues/124
[#127]: https://gitlab.com/azriel91/autexousious/issues/127
[#128]: https://gitlab.com/azriel91/autexousious/issues/128
[#129]: https://gitlab.com/azriel91/autexousious/issues/129

## 0.10.0 (2019-03-29)

### Added

* Added `dodge` sequence. ([#102])
* Added `dash_forward` and `dash_back` sequences. ([#102])
* Added basic health bars to characters. ([#104])
* `#[numeric_newtype]` proc macro attribute to derive numeric traits. ([#98])
* `stdio_command state_barrier <state_id>` blocks stdin until the given state is running. ([#105])

### Changed

* Use logic clock based system to update object components each tick. ([#92])
* Use logic clock based system to update map components each tick. ([#99])
* ***Breaking:*** In objects and maps, sprites are now specified using `sprite = { sheet = 0, index = 0 }`. ([#92])
* Stdin command input now needs a comment line to indicate "no input this frame". ([#105])

### Fixed

* Map layers are now positioned correctly. ([#99])
* Game does not go past loading when asset loading has error. ([#101])

### Removed

* No longer use `amethyst_animation` to update object and map components. ([#92], [#99])

[#92]: https://gitlab.com/azriel91/autexousious/issues/92
[#98]: https://gitlab.com/azriel91/autexousious/issues/98
[#99]: https://gitlab.com/azriel91/autexousious/issues/99
[#101]: https://gitlab.com/azriel91/autexousious/issues/101
[#102]: https://gitlab.com/azriel91/autexousious/issues/102
[#104]: https://gitlab.com/azriel91/autexousious/issues/104
[#105]: https://gitlab.com/azriel91/autexousious/issues/105

## 0.9.0 (2019-02-15)

### Added

* Control input via stdin. ([#93])
* First cut of hot reloading. ([#94])

### Changed

* Characters and objects are now instantiated using prefabs. ([#56])
* `KeyboardEscapeIntercept` is now tested properly without simulating input. ([#15])
* Load game objects and maps asynchronously. ([#94])

[#15]: https://gitlab.com/azriel91/autexousious/issues/15
[#93]: https://gitlab.com/azriel91/autexousious/issues/93
[#94]: https://gitlab.com/azriel91/autexousious/issues/94

## 0.8.0 (2019-01-04)

### Changed

* Object animations are updated in a dedicated system. ([#81])
* Components are now more granular. ([#83])
* Components are augmented onto existing entities instead of added when built. ([#56])

[#56]: https://gitlab.com/azriel91/autexousious/issues/56
[#81]: https://gitlab.com/azriel91/autexousious/issues/81
[#83]: https://gitlab.com/azriel91/autexousious/issues/83

## 0.7.0 (2018-11-23)

### Added

* Naive collision detection and effects. ([#69])
* Flinch and foward falling sequences. ([#70])
* Game win condition when there is one player remaining. ([#71])
* Mirrored collision detection. ([#82])

[#69]: https://gitlab.com/azriel91/autexousious/issues/69
[#70]: https://gitlab.com/azriel91/autexousious/issues/70
[#71]: https://gitlab.com/azriel91/autexousious/issues/71
[#82]: https://gitlab.com/azriel91/autexousious/issues/82

## 0.6.0 (2018-10-12)

### Added

* Map selection UI that allows users to select maps. ([#66])
* Character selection controllable via stdin. ([#67])
* Map selection controllable via stdin. ([#67])
* Game mode selection controllable via stdin. ([#67])
* Configuration types for hit (`interactions`) and hurt (`body`) boxes. ([#68])

### Changed

* Use `AssetSlug` (*namespace/name*) to reference assets. ([#57])
* Updated Amethyst to make use of sprite batching and custom events. ([#63])

### Fixed

* Allow maps without layers. ([#72])

[#57]: https://gitlab.com/azriel91/autexousious/issues/57
[#63]: https://gitlab.com/azriel91/autexousious/issues/63
[#66]: https://gitlab.com/azriel91/autexousious/issues/66
[#67]: https://gitlab.com/azriel91/autexousious/issues/67
[#68]: https://gitlab.com/azriel91/autexousious/issues/68
[#72]: https://gitlab.com/azriel91/autexousious/issues/72

## 0.5.0 (2018-08-31)

### Added

* Implemented maps with bounds and layers. ([#47], [#48])
* Updated sprites configuration to take in `u32` instead of `f32` for sprite dimensions. ([#47])
* Prevent characters from moving outside map margins. ([#48])
* Use `typename` to derive `System` names. ([#52])
* Implemented rudimentary character selection menu. ([#51])

### Changed

* Sprite offsets are optional when declaring sprite sheets. ([#47])
* Sprites are rendered using a dedicated sprite pass. ([#55])
* `stop_run` sequence has been renamed to `run_stop`. ([#49])
* Use state specific dispatcher for `GamePlayState`. ([#50])

### Fixed

* Map layers with different sprite dimensions are now rendered with the right layout data. ([#55])
* Characters switch to the `jump_descend` sequence when airborne during `walk`, `run`, and `run_stop`. ([#49])

[#47]: https://gitlab.com/azriel91/autexousious/issues/47
[#48]: https://gitlab.com/azriel91/autexousious/issues/48
[#49]: https://gitlab.com/azriel91/autexousious/issues/49
[#50]: https://gitlab.com/azriel91/autexousious/issues/50
[#51]: https://gitlab.com/azriel91/autexousious/issues/51
[#52]: https://gitlab.com/azriel91/autexousious/issues/52
[#55]: https://gitlab.com/azriel91/autexousious/issues/55

## 0.4.0 (2018-07-20)

### Added

* Implemented jump.
* Implemented double tap to run.
* Mirror sprites when entity is facing left.
* Created test harness to make it easier to test logic.

## 0.3.0 (2018-06-08)

### Added

* Read controller configuration.
* Update character sequence based on input.

## 0.2.0 (2018-04-26)

### Added

* Loads object sprites and animations.
* Creates an entity per object.

## 0.1.0 (2018-03-24)

### Added

* Loads fonts and displays a simple menu.
* Published on [itch.io](https://azriel91.itch.io/will)
