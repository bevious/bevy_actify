use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin, PreUpdate},
    ecs::{
        event::EventWriter,
        schedule::{IntoSystemConfigs, SystemSet},
        system::{Local, Res, ResMut},
    },
};

use crate::{InputAction, InputActionUpdated, internal};

/// Label for systems that update input actions.
///
/// ### Usage
/// - Those systems that provide input action state (i.e.,
///   pour into [`InputActionDrain`]) should be configured
///   to run **before** this set.
/// - Those systems that read input action state should be
///   configured to run **after** this set.
///
/// ### Notes
/// Since all input action systems run in the `PreUpdate`
/// stage, the systems that read input action state almost
/// never have to be explicitly configured to run after this
/// set, because they are most likely to run in the `Update`
/// schedule, which already runs *after*.
#[derive(SystemSet, Hash, PartialEq, Eq, Clone, Debug)]
pub struct InputActionSystem;

/// Adds the input action to an app.
pub struct InputActionPlugin<A: InputAction> {
    _marker: PhantomData<A>,
}

impl<A: InputAction> InputActionPlugin<A> {
    /// Returns a new input action plugin.
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<A: InputAction> Default for InputActionPlugin<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: InputAction> Plugin for InputActionPlugin<A> {
    fn build(&self, app: &mut App) {
        app.init_resource::<internal::InputActionState<A>>();
        app.init_resource::<internal::InputActionDrain<A>>();

        app.add_event::<InputActionUpdated<A>>();

        app.add_systems(
            PreUpdate,
            (
                update_input_action_state::<A>,
                write_input_action_events::<A>,
            )
                .chain()
                .in_set(InputActionSystem),
        );
    }
}

/// Updates the [`InputActionState`] based on the value
/// in the [`InputActionDrain`].
///
/// This system reads the current value from the
/// [`InputActionDrain`] and updates the [`InputActionState`]
/// accordingly:
/// - If the drain contains a value, the state becomes
///   [`InputActionState::Active`].
/// - If the drain is empty, the state becomes
///   [`InputActionState::Idle`].
///
/// ### Behavior
/// - The drain is cleared after its value is read.
/// - This system should run **before** any systems that
///   depend on the [`InputActionState`].
fn update_input_action_state<A: InputAction>(
    mut drain: ResMut<internal::InputActionDrain<A>>,
    mut state: ResMut<internal::InputActionState<A>>,
) {
    *state = drain
        .take()
        .map_or(internal::InputActionState::Idle, |state| {
            internal::InputActionState::Active(state)
        });
}

/// Writes events based on changes to the [`InputActionState`].
///
/// This system tracks the previous state of the input action
/// and emits events when the state changes:
/// - **`Started`**: Emitted when the state transitions from
///   `Idle` to `Active`.
/// - **`Stopped`**: Emitted when the state transitions from
///   `Active` to `Idle`.
/// - **`Updated`**: Emitted when the state remains `Active`
///   but the value changes.
///
/// ### Behavior
/// - Uses a [`Local`] resource to track the previous state.
/// - Compares the previous state with the current state to
///   determine if an event should be emitted.
/// - This system should run **after** the [`InputActionState`]
///   is updated.
///
fn write_input_action_events<A: InputAction>(
    mut local: Local<Option<A>>,
    mut event: EventWriter<InputActionUpdated<A>>,
    state: Res<internal::InputActionState<A>>,
) {
    let state = match state.as_ref() {
        internal::InputActionState::Active(state) => Some(state),
        internal::InputActionState::Idle => None,
    };

    match (&*local, state) {
        (None, None) => {}
        (None, Some(value)) => {
            event.send(InputActionUpdated::Started(value.clone()));
        }
        (Some(_), None) => {
            event.send(InputActionUpdated::Stopped);
        }
        (Some(previous), Some(next)) => {
            if previous != next {
                event.send(InputActionUpdated::Updated(next.clone()));
            }
        }
    };

    *local = state.cloned();
}
