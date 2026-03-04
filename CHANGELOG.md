# Changelog

All notable changes to this project will be documented in this file.

## [0.1.3] - 2026-03-04

### 🚀 Features

- *(cli)* Add cli command to change tag

### 🐛 Bug Fixes

- *(ci)* Fail lint on warning

## [0.1.2] - 2026-02-08

### 🚀 Features

- *(cli)* Rename --environment to --env
- *(tui)* Add first tui version
- *(tui)* Implement sidebar
- *(tui)* Add empty main panel
- *(tui)* Switch panel on click
- *(tui)* Update theming
- *(tui)* Add list of environments
- *(tui)* Activate projects/environments
- *(tui)* Show environment values in table
- *(tui)* Show command list and selected command
- *(tui)* Make sidebar lists clickable
- *(sidebar)* Simplify navigation with auto-activate and space toggle
- *(tui)* Syntax highlight command
- *(tui)* Colorize environment values
- *(tui)* Make commands clickable
- *(tui)* Add copy button
- *(tui)* Show tags
- *(pacs)* Add --ui flag which launches TUI

### 🐛 Bug Fixes

- *(cli)* Exclude other projects from tag completion
- *(tui)* Reset selected command when switching project
- *(tui)* Fix linter warnings
- *(tui)* Fix format issue

### 📚 Documentation

- *(demo)* Update demo tape and gif
- *(readme)* Add --ui flag to launch TUI

### 🔧 Refactor

- *(core)* Simplify get active project code
- *(tui)* Move keybindings setup out of render loop
- *(core)* Add get_active_project methods
- *(core,cli)* Add list_environments helper
- *(core,cli)* Rename activate_environment to set_active_environment
- *(tui)* Split out run function into lib

### ⚙️ Miscellaneous Tasks

- *(tui)* Use crates.io dependencies with local overrides

### Build

- *(ci)* Add pacs-tui to CI

## [0.1.1] - 2026-01-24

### 🚀 Features

- *(cli)* Mute command color
- *(cli)* Show info if no commands are found

### 🐛 Bug Fixes

- *(cli)* Expand env if listed with cmd name
- *(cli)* Remove support for global commands
- *(ci)* Exit clippy on warnings

### 📚 Documentation

- *(readme)* Add homebrew installation
- *(readme)* Add a "why pacs" section
- *(readme)* Update example output

### ⚙️ Miscellaneous Tasks

- *(readme)* Keep gif external

## [0.1.0] - 2026-01-24

### 🚀 Features

- *(all)* Add context model and command expansion
- *(cli)* Add autocomplete for contexts
- *(cli)* Rename activate -> switch, deactivate -> clear
- *(cli)* Colorize "env ls" outpout
- *(cli)* Auto switch to new project/env

### 🐛 Bug Fixes

- *(core)* Allow copy and run commands to fall back to global scope

### 📚 Documentation

- *(readme)* Add badges to README
- *(readme)* Extend usage section
- *(demo)* Add demo tape and gif
- *(demo)* Update demo tape

### 🔧 Refactor

- *(core)* Simplify pacs core api
- *(all)* Rename context to env

### ⚙️ Miscellaneous Tasks

- *(readme)* Update usage section
- *(readme)* Update context and placeholder section
- *(lint)* Fix clippy warnings
- *(lint)* Fix clippy warnings

## [0.1.0-beta.2] - 2026-01-24

### 🚀 Features

- *(cli)* List command names only with -n
- *(core)* Sort commands when listing
- *(core)* Sort commands when saving
- *(cli)* Do not inset commands for easier copy paste
- *(cli)* Colorize commands and make them bold

### ⚙️ Miscellaneous Tasks

- Add readme to crate metadata

## [0.1.0-beta.1] - 2026-01-22

### 🚀 Features

- Hello world pacs


