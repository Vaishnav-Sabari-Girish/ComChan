
## Unreleased



### :rocket: New features

- **(dark_mode)** Added Dark Mode for SVG exports

- **(dark_mode)** Added Dark Mode for SVG exports

- **(smooth)** Smoother 3D telemetry viewer.

- **(smooth)** Smoother 3D telemetry viewer.

- **(feature)** Added feature flags to terminal canvas

- **(rw_no_std)** Add no_std support for `ratatui-wireframe`


### :bug: Bug fixes

- **(error)** Added proper error message for drain loop

- **(trig)** Trigonometry issues

- **(default)** Removed `ratatui` default features

- **(features)** Add features


### :zap: Performance

- **(limit)** Limited the drain to prevent overloading of thread


## v0.7.0 - 2026-05-29



### :rocket: New features

- **(octahedron_plus_custom_file)** Add octahedron model

- **(file)** Add wrfm file based braille models

- **(tetrahedron)** Add tetrahedron braille model

- **(custom_obj_file)** Ability to use a custom obj file

- **(zephyr_logs)** Supports plotting from Zephyr Logs


### :bug: Bug fixes

- **(zephyr)** Logged data was not plotting

- **(edge)** Fixed some edge cases

- **(zephyr)** Logged data was not plotting

- **(model)** Cache parsed wrfm models to prevent per-frame parsing overhead

- **(tetrahedron)** Tetrahedron placement

- **(cargo)** Removed `ratty` from default features

- **(error)** Fixes silent error in obj file finding

- **(debug)** Debug logging type added

- **(better parsing)** Better parsing for logs


### :zap: Performance

- **(render)** Stop cloning mesh data inside the 60fps loop


### :art: Styling

- **(parser)** Better coding standards


### :hammer: Build

- **(version)** Add version for `ratatui-wireframe`


## v0.6.0 - 2026-05-23



### :rocket: New features

- **(3D)** Cube is proper (kinda) with a gnomon

- **(plotter)** Add hardware-accelerated 3D rendering with graceful fallback

- **(plotter)** Integrate live IMU telemetry and modularize 3D engine

- **(plotter)** Add zero-dependency 3D wireframe engine and tabbed UI


### :bug: Bug fixes

- **(plotter)** Correct Ratty terminal rendering mode detection

- **(ghosting)** Cube did not clear after changing tabs


## v0.5.0-rc1 - 2026-05-17



### :rocket: New features

- **(hex-pretty)** Add pretty print mode for hex dump

- **(hex mode)** Basic hex mode


### :bug: Bug fixes

- **(hex)** Resolve stale buffer, simulate logic, and config types


## v0.4.0 - 2026-05-15



### :rocket: New features

- **(session_replay)** Add Session Replay feature


### :art: Styling

- **(clippy)** Fixed clippy warning about EOF


## v0.3.7-rc1 - 2026-05-12



### :rocket: New features

- **(csv)** Add CSV file streaming


## v0.3.6-rc4 - 2026-05-07



### :rocket: New features

- **(recovery)** Add recovery after re-connection


## v0.3.6-rc3 - 2026-05-07



### :bug: Bug fixes

- **(graceful-exit)** Add a graceful exit during disconnection


## v0.3.6-rc2 - 2026-05-07



### :rocket: New features

- **(nushell)** Add completions for nushell


## v0.3.6-rc1 - 2026-05-07



### :rocket: New features

- **(completions)** Add completions


## v0.3.5 - 2026-05-06



### :rocket: New features

- **(simulate)** Add simulate option to simulate without hardware


## v0.3.5-beta - 2026-04-30



### :rocket: New features

- **(plot-title)** Add custom plot-title option


### :zap: Performance

- **(&str)** Change &String to &str for idiomatic rust


### :hammer: Build

- **(pre-release)** Pre-release with custom plot title


## v0.3.4 - 2026-04-30



### :rocket: New features

- **(export)** Export plot to SVG


### :bug: Bug fixes

- **(export limit)** Add export limit to prevent RAM overusage

- **(crash)** Fix crash due to less number of data points

- **(zephyr)** Add seperate mode for Zephyr shell


## v0.3.3 - 2026-04-21



### :rocket: New features

- **(control code)** Add control codes (CTRL+L) to clear screen

- **(timestamps)** Timestamps changed from EPOCH to local time


### :hammer: Build

- **(bump)** Version bump from 0.3.2 to 0.3.3

- **(bump)** Bumped up version [skip ci]


## v0.3.2 - 2026-04-19



### :rocket: New features

- **(zephyr)** Zephyr Shell support


## v0.3.0 - 2026-04-09



### :bug: Bug fixes

- **(clippy)** `libudev` error

- **(bug)** Fixed copy issue for variable `self.x`


### :zap: Performance

- **(Optimization)** Use Cow to reduce string cloning


### :recycle: Refactoring

- **(start)** Refactor start


## v0.2.5 - 2026-02-06



### :rocket: New features

- **(replaced emojis)** Replaced emojis with Nerd Font Characters


### :hammer: Build

- **(lock file update)** Lock file update

- **(bump ratatui)** Bump Ratatui version


## v0.1.0 - 2025-06-29


<!-- ComChan -->
