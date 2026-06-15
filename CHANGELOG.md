
## v0.11.0 - 2026-06-15







### :rocket: New features

- **(ble)** Added BLE capabilities to `comchan`






### :bug: Bug fixes

- **(stream)** Resolve Cubic diagnostics for BLE lifecycle and UI stability










### :recycle: Refactoring

- **(ble)** Use typed events for channel communication










### :hammer: Build

- **(dist-workspace)** Add the BLE feature for compilation



## v0.10.1 - 2026-06-14







### :rocket: New features

- **(3d)** 3D obj file reading for `ratatui-wireframe`

- **(switch_modes)** Switch between modes plotter -> monitor, vice-versa

- **(switch_modes)** Switch between modes plotter -> monitor, vice-versa

- **(rtt)** Add `rtt`/`defmt` log viewing

- **(rtt)** Add `rtt`/`defmt` log viewing

- **(dark_mode)** Added Dark Mode for SVG exports

- **(dark_mode)** Added Dark Mode for SVG exports

- **(smooth)** Smoother 3D telemetry viewer.

- **(smooth)** Smoother 3D telemetry viewer.

- **(feature)** Added feature flags to terminal canvas

- **(rw_no_std)** Add no_std support for `ratatui-wireframe`

- **(octahedron_plus_custom_file)** Add octahedron model

- **(file)** Add wrfm file based braille models

- **(tetrahedron)** Add tetrahedron braille model

- **(custom_obj_file)** Ability to use a custom obj file

- **(zephyr_logs)** Supports plotting from Zephyr Logs

- **(3D)** Cube is proper (kinda) with a gnomon

- **(plotter)** Add hardware-accelerated 3D rendering with graceful fallback

- **(plotter)** Integrate live IMU telemetry and modularize 3D engine

- **(plotter)** Add zero-dependency 3D wireframe engine and tabbed UI

- **(hex-pretty)** Add pretty print mode for hex dump

- **(hex mode)** Basic hex mode

- **(session_replay)** Add Session Replay feature

- **(csv)** Add CSV file streaming

- **(recovery)** Add recovery after re-connection

- **(nushell)** Add completions for nushell

- **(completions)** Add completions

- **(simulate)** Add simulate option to simulate without hardware

- **(plot-title)** Add custom plot-title option

- **(export)** Export plot to SVG

- **(control code)** Add control codes (CTRL+L) to clear screen

- **(timestamps)** Timestamps changed from EPOCH to local time

- **(zephyr)** Zephyr Shell support






### :bug: Bug fixes

- **(no_std)** Auto detect std and no_std via feature flags

- **(edge)** Fixed some edge cases in the bounds checking

- **(monitor)** Allow mode-switch and quit during reconnect delays

- **(error)** Better error propagation in case re-connect fails

- **(error)** Better error propagation and re-connection

- **(rtt)** Resolve reconnect failures and eager config validation

- **(cubic+reconnection)** Fixed stuff as per cubic and fixed reconnection

- **(error)** Added proper error message for drain loop

- **(trig)** Trigonometry issues

- **(default)** Removed `ratatui` default features

- **(features)** Add features

- **(zephyr)** Logged data was not plotting

- **(edge)** Fixed some edge cases

- **(zephyr)** Logged data was not plotting

- **(model)** Cache parsed wrfm models to prevent per-frame parsing overhead

- **(tetrahedron)** Tetrahedron placement

- **(cargo)** Removed `ratty` from default features

- **(error)** Fixes silent error in obj file finding

- **(debug)** Debug logging type added

- **(better parsing)** Better parsing for logs

- **(plotter)** Correct Ratty terminal rendering mode detection

- **(ghosting)** Cube did not clear after changing tabs

- **(hex)** Resolve stale buffer, simulate logic, and config types

- **(graceful-exit)** Add a graceful exit during disconnection

- **(export limit)** Add export limit to prevent RAM overusage

- **(crash)** Fix crash due to less number of data points

- **(zephyr)** Add seperate mode for Zephyr shell

- **(clippy)** `libudev` error

- **(bug)** Fixed copy issue for variable `self.x`






### :zap: Performance

- **(limit)** Limited the drain to prevent overloading of thread

- **(render)** Stop cloning mesh data inside the 60fps loop

- **(&str)** Change &String to &str for idiomatic rust

- **(Optimization)** Use Cow to reduce string cloning






### :recycle: Refactoring

- **(monitor)** Extract reconnect polling loop into macro

- **(start)** Refactor start






### :art: Styling

- **(parser)** Better coding standards

- **(clippy)** Fixed clippy warning about EOF






### :hammer: Build

- **(typos)** Fixed some typos

- **(version)** Add version for `ratatui-wireframe`

- **(pre-release)** Pre-release with custom plot title

- **(bump)** Version bump from 0.3.2 to 0.3.3

- **(bump)** Bumped up version [skip ci]



### :tada: New Contributors
- @github-actions[bot] made their first contribution
- @Adez017 made their first contribution
- @ made their first contribution
- @Nimit746 made their first contribution
## v0.2.5 - 2026-02-06







### :rocket: New features

- **(replaced emojis)** Replaced emojis with Nerd Font Characters






















### :hammer: Build

- **(lock file update)** Lock file update

- **(bump ratatui)** Bump Ratatui version



## v0.1.0 - 2025-06-29




























### :tada: New Contributors
- @Vaishnav-Sabari-Girish made their first contribution
