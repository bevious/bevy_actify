<p align="center">
  <img src="https://github.com/bevious/bevy_actify/blob/main/logo.png?raw=true" width="250" />
</p>
<p align="center">
  An input action plugin for <a href="https://bevyengine.org/"><strong>Bevy</strong></a><br />
</p>
<hr />
<p align="center">
  <a href="https://crates.io/crates/bevy_actify">
    <img alt="crates.io" src="https://img.shields.io/crates/v/bevy_actify" />
  </a>
  <img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue" />
  <img alt="Bevy version: 0.16" src="https://img.shields.io/badge/Bevy-0.16-pink" />
  <a href="https://github.com/bevious/bevy_actify/actions/workflows/cargo.yml">
    <img alt="Action: Test" src="https://github.com/bevious/bevy_actify/actions/workflows/cargo.yml/badge.svg" />
  </a>
</p>

A tiny abstraction layer for input handling in [Bevy](https://bevyengine.org/) that decouples
input sources from game logic through clean action-based interfaces.

## Problem

Raw input handling leads to:

- Tight coupling between devices and game logic
- Code duplication across input methods
- Messy state management

## How to use

### Basics

#### InputActionDrain

`InputActionDrain` is a system parameter, that is the *vassel* into which the systems *pour* input action state.

For example a system would read keyboard input and *pour* the state into the appropriate
`InputActionDrain`.

#### InputActionState

`InputActionState` is a system parameter, that is the source of current input action state. The gameplay system
(i.e. movement, jump) would read the input action state from this.

#### InputActionReader

`InputActionReader` is a system parameter, that provides an event based interface, similar to `EventReader` (actually it
uses an event reader under the hood). The read values are of type `InputActionStatus`—an `InputActionStatus` can be one of:
- `Started` when the input action has just started to be active,
- `Updated` when the input action has already been active, but the value has changed,
- `Stopped` when the input action had been active, but now is not.

### Run conditions

This library provides several input action based system run conditions. Those are:
- `input_action_active`: The system will run each frame the input action is active
- `input_action_started`: Similar to `ButtonInput`'s `just_pressed`—will make your system run at the frame on which the action has started
- `input_action_updated`: The system will run if the state of an input action has changed (i.e. an axis changed direction)
- `input_action_stopped`: The system will run once the input action gets stopped (i.e. a button has been released)

### Example

```rust
use bevy::{prelude::*, input::InputSystem};
use bevy_actify::prelude::*;

// 1. Define your action
#[derive(InputAction, Clone, PartialEq)]
struct Jump(f32); // f32 for analog sensetivity

// 2. Map inputs to actions
fn keyboard_input(keyboard: Res<ButtonInput<KeyCode>>, mut action: InputActionDrain<Jump>) {
    if keyboard.pressed(KeyCode::Space) {
        action.pour(Jump(1f32));
    }
}

// 3. Use in game systems
fn character_jump(action: InputActionState<Jump>) {
    if let Some(state) = action.state() {
        let jump_power = state.0;
        // Apply force...
    }
}

// 4. Register the input action and systems
fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_input_action::<Jump>()
    .add_systems(PreUpdate, keyboard_input.after(InputSystem).before(InputActionSystem)) // properly order your systems to avoid 1 frame delay!
    .add_systems(Update, character_jump)
    .run();
}
```

## How to contribute

Fork repository, make changes, and send us a pull request.

We will review your changes and apply them to the main
branch shortly, provided they don't violate our quality standards.

## License

This project is dual-licensed under:

- [MIT License](LICENSE-MIT)
- [Apache 2.0 License](LICENSE-APACHE-2.0)

You may choose either license at your option.
