<p align="center">
  <img src="https://github.com/iizudev/bevy_actify/blob/main/logo.png?raw=true" width="250" />
</p>
<p align="center">
  An input action plugin for <a href="https://bevyengine.org/"><strong>Bevy</strong></a>
</p>
<hr />

This plugin provides a unified way to handle input actions, allowing
developers to decouple game logic from specific input sources like keyboards,
gamepads, or touchscreens. Instead of hardcoding input details, you define
abstract input actions (e.g., "Jump", "Attack") and map them to any input
source.

## How to use
First things first you need to add this plugin as a dependency to your project by running:
```bash
cargo add bevy_actify
```

or by manually adding it to your `Cargo.toml`'s `dependencies` section:
```toml
# refer to https://crates.io/crates/bevy_actify for the latest version
bevy_actify = { version = "*" }
```

### Usage
```rust
use bevy::{input::InputSystem, prelude::*};
use bevy_actify::*;

#[derive(InputAction, Clone, PartialEq, Debug)]
struct MyAction;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, InputActionPlugin::<MyAction>::new()))
        .add_systems(
            PreUpdate,
            keyboard_to_my_action
                .after(InputSystem)
                .before(InputActionSystem),
        )
        .add_systems(Update, print_my_action)
        .run();
}

fn keyboard_to_my_action(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut action: InputActionDrain<MyAction>,
) {
    if keyboard.pressed(KeyCode::KeyF) {
        action.pour(MyAction);
    }
}

fn print_my_action(mut action: InputActionReader<MyAction>) {
    action.read().for_each(|a| println!("action: {:#?}", a));
}
```

## How to contribute
Fork repository, make changes, send us a pull request. We will review your
changes and apply them to the master branch shortly, provided they don't
violate our quality standards.
