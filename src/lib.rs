//! SPDX-License-Identifier: MIT OR Apache-2.0
#![doc = include_str!("../README.md")]

#[cfg(feature = "derive")]
pub use bevy_actify_derive::InputAction;

use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin, PreUpdate},
    ecs::{
        event::{EventReader, EventWriter},
        schedule::{IntoSystemConfigs, SystemSet},
        system::{Local, Res, ResMut, SystemParam},
    },
};

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

/// Plugin that adds the input action `A` to an
/// app.
///
/// This will register the resources and systems
/// required for an input action to fully function.
///
/// ### Usage
/// You can contribute to the input action via
/// [`InputActionDrain`] before [`InputActionSystem`] in
/// [`PreUpdate`](bevy::app::PreUpdate) and read its state
/// via either [`InputActionState`] or [`InputActionReader`]
/// after [`InputActionSystem`].
///
/// ### Example
/// ```rust
/// # use bevy::{prelude::*, input::InputSystem};
/// # use bevy_actify::*;
///
/// #[derive(InputAction, PartialEq, Clone)]
/// struct Jump;
///
/// # fn main() {
/// App::new()
///    .add_plugins(
///        (
///            DefaultPlugins,
///            InputActionPlugin::<Jump>::new(),
///        ),
///    )
///    .add_systems(
///        PreUpdate,
///        // this system reads keyboard input and
///        // writes it into the drain.
///        keyboard_jump
///            .after(InputSystem)
///            .before(InputActionSystem),
///    )
///    .run();
/// # }
///
/// fn keyboard_jump(keyboard: Res<ButtonInput<KeyCode>>, mut action: InputActionDrain<Jump>) {
///     if keyboard.pressed(KeyCode::Space) {
///         action.pour(Jump);
///     }
/// }
/// ```
pub struct InputActionPlugin<A: InputAction> {
    _marker: PhantomData<A>,
}

/// Provides read-only access to the current state of an
/// input action.
///
/// An action may be in either:
/// - **`Active`** status: The action is currently active
///   and has an associated value.
/// - **`Idle`** status: The action is not active.
///
/// ### Usage
/// Use this type in systems that need to read the state
/// of an input action.
///
/// ### Notes
/// - The state is updated every frame based on the values
///   poured into the [`InputActionDrain`].
/// - To check if the action is active without cloning the
///   value, use [`InputActionState::is_active`].
#[derive(SystemParam, Debug)]
pub struct InputActionState<'w, A: InputAction> {
    inner: Res<'w, internal::InputActionState<A>>,
}

/// Provides write access to the input action drain.
///
/// The drain collects values from multiple systems to
/// resolve the final state of the input action.
///
/// ### Usage
/// Use this type in systems that provide input action state,
/// see [`InputActionDrain::pour`].
///
/// ### Behavior
/// - The drain only retains the **most recent state** poured
///   into it.
/// - Any previously poured value is overwritten by the new value.
/// - The drain is cleared every frame after its value is read
///   to update the [`InputActionState`].
///
/// ### Notes
/// - If multiple systems pour state into the drain, only the
///   **last value** poured will be used to update the [`InputActionState`].
#[derive(SystemParam, Debug)]
pub struct InputActionDrain<'w, A: InputAction> {
    inner: ResMut<'w, internal::InputActionDrain<A>>,
}

/// Represents the status of an input action as read
/// from an [`InputActionReader`].
///
/// This enum describes the lifecycle of an input action,
/// indicating whether it has just started, been updated,
/// or stopped. It is typically used to react to changes
/// in input state in a structured way.
///
/// ### Variants
/// - **`Started(A)`**: The input action has transitioned
///   from `Idle` to `Active`. This variant contains the
///   current state of the input action (`A`).
/// - **`Updated(A)`**: The input action was already `Active`,
///   but its state has changed. This variant contains the
///   updated state of the input action (`A`).
/// - **`Stopped`**: The input action has transitioned from
///   `Active` to `Idle`. This variant does not contain
///   additional data, as the action is no longer active.
#[derive(Debug)]
pub enum InputActionStatus<'e, A: InputAction> {
    Started(&'e A),
    Updated(&'e A),
    Stopped,
}

/// Reader for input action status updates.
///
/// This system param provides an event-like way to react to
/// changes in input actions.
#[derive(SystemParam, Debug)]
pub struct InputActionReader<'w, 's, A: InputAction> {
    inner: EventReader<'w, 's, internal::InputActionUpdated<A>>,
}

pub trait InputAction: Send + Sync + Clone + PartialEq + 'static {}

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

        app.add_event::<internal::InputActionUpdated<A>>();

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

impl<'w, A: InputAction> InputActionState<'w, A> {
    /// Returns whether the input action is currently active.
    pub fn is_active(&self) -> bool {
        matches!(self.inner.as_ref(), internal::InputActionState::Active(_))
    }

    /// Returns the current state of the input action.
    ///
    /// The input action can be in one of two statuses:
    /// - **`Active`**: The action is currently active,
    ///   and this method returns `Some(value)`.
    /// - **`Idle`**: The action is not active, and this
    ///   method returns `None`.
    ///
    /// ### Notes
    /// - If the action is `Active`, this method **clones**
    ///   the associated value. If you only need to check whether
    ///   the action is active (without cloning the value), use
    ///   [`InputActionState::is_active`].
    /// - Cloning the value may have a performance cost, so
    ///   avoid calling this method repeatedly if you only need
    ///   to check the active status.
    pub fn state(&self) -> Option<A> {
        match self.inner.as_ref() {
            internal::InputActionState::Active(state) => Some(state.clone()),
            internal::InputActionState::Idle => None,
        }
    }
}

impl<'w, A: InputAction> InputActionDrain<'w, A> {
    /// Pours (writes) a state into the input action drain.
    ///
    /// This method is used to contribute a state to the input action
    /// system. The state represents the current value of the input
    /// action as provided by a specific source (e.g., keyboard,
    /// gamepad, or other input systems).
    ///
    /// ### Behavior
    /// - The drain only retains the **most recent state** that was
    ///   poured into it.
    /// - Any previously poured state is overwritten by the new state.
    /// - The drain is cleared every frame after its state is read
    ///   to update the `InputActionState`.
    ///
    /// ### Notes
    /// - This method is typically called by systems that provide
    ///   input action values (e.g., keyboard or gamepad input systems).
    /// - If multiple systems pour states into the drain, only
    ///   the **last state** poured will be used to update the
    ///   `InputActionState`.
    pub fn pour(&mut self, state: A) {
        self.inner.replace(state);
    }
}

impl<'e, A: InputAction> From<&'e internal::InputActionUpdated<A>> for InputActionStatus<'e, A> {
    fn from(value: &'e internal::InputActionUpdated<A>) -> Self {
        match value {
            internal::InputActionUpdated::Started(state) => Self::Started(state),
            internal::InputActionUpdated::Updated(state) => Self::Updated(state),
            internal::InputActionUpdated::Stopped => Self::Stopped,
        }
    }
}

impl<'w, 's, A: InputAction> InputActionReader<'w, 's, A> {
    /// see [`EventReader::read`](bevy::ecs::event::EventReader::read).
    pub fn read(&mut self) -> impl ExactSizeIterator<Item = InputActionStatus<A>> {
        self.inner.read().map(|event| event.into())
    }

    /// see [`EventReader::is_empty`](bevy::ecs::event::EventReader::is_empty).
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// see [`EventReader::clear`](bevy::ecs::event::EventReader::clear).
    pub fn clear(&mut self) {
        self.inner.clear()
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
    mut event: EventWriter<internal::InputActionUpdated<A>>,
    state: Res<internal::InputActionState<A>>,
) {
    let state = match state.as_ref() {
        internal::InputActionState::Active(state) => Some(state),
        internal::InputActionState::Idle => None,
    };

    match (&*local, state) {
        (None, None) => {}
        (None, Some(value)) => {
            event.send(internal::InputActionUpdated::Started(value.clone()));
        }
        (Some(_), None) => {
            event.send(internal::InputActionUpdated::Stopped);
        }
        (Some(previous), Some(next)) => {
            if previous != next {
                event.send(internal::InputActionUpdated::Updated(next.clone()));
            }
        }
    };

    *local = state.cloned();
}

mod internal {
    use std::ops::{Deref, DerefMut};

    use bevy::ecs::{event::Event, system::Resource};

    use crate::InputAction;

    /// Represents the current state of an input action.
    ///
    /// The state can be either:
    /// - **`Active`**: The input action is currently active
    ///   and has an associated value.
    /// - **`Idle`**: The input action is not active.
    ///
    /// The state is updated every frame based on the values
    /// poured into the [`InputActionDrain`].
    #[derive(Resource, Debug)]
    pub enum InputActionState<A: InputAction> {
        Active(A),
        Idle,
    }

    /// Temporary storage for the current input action state.
    ///
    /// This resource is used by *producing systems* to write
    /// the current state of an input action. Only the most
    /// recently written value is retained, and older values
    /// are discarded.
    ///
    /// The value in the drain is used to update the [`InputActionState`]
    /// at the end of each frame.
    ///
    /// ### Behavior
    /// - If multiple systems write to the drain, only the
    ///   **last value** written will be used.
    /// - The drain is automatically cleared after its value
    ///   is read to update the [`InputActionState`].
    #[derive(Resource, Debug)]
    pub struct InputActionDrain<A: InputAction>(Option<A>);

    /// Input action update event.
    ///
    /// This event is written when state of the input action
    /// changes.
    #[derive(Event, Debug)]
    pub enum InputActionUpdated<A: InputAction> {
        Started(A),
        Updated(A),
        Stopped,
    }

    impl<A: InputAction> Default for InputActionState<A> {
        fn default() -> Self {
            Self::Idle
        }
    }

    impl<A: InputAction> Default for InputActionDrain<A> {
        fn default() -> Self {
            Self(None)
        }
    }

    impl<A: InputAction> Deref for InputActionDrain<A> {
        type Target = Option<A>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<A: InputAction> DerefMut for InputActionDrain<A> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
}
