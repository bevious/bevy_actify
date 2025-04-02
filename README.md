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
  <img alt="Bevy version: 0.15" src="https://img.shields.io/badge/Bevy-0.15-pink" />
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
