//! SPDX-License-Identifier: MIT OR Apache-2.0
//!
//! A flexible and ergonomic input action system for Bevy.
//!
//! This library provides a structured way to handle input actions in Bevy,
//! allowing you to define abstract input actions that can be mapped from
//! various input sources (keyboard, mouse, gamepad, etc.) and consumed
//! by game systems in a consistent way.
//!
//! # Features
//!
//! - **Input Action Abstraction**: Define abstract input actions separate
//!   from concrete inputs
//! - **State Management**: Track whether actions are active/idle and their
//!   current values
//! - **Event-like System**: React to action starts, updates, and stops
//! - **Multi-source Input**: Combine inputs from multiple sources into a
//!   single action state
//! - **Conditional Systems**: Feature-gated helpers for common input conditions
//!
//! # Core Concepts
//!
//! - **`InputAction`**: A trait representing an abstract input action
//! - **`InputActionPlugin`**: Registers an action and its systems
//! - **`InputActionState`**: Read the current state of an action
//! - **`InputActionDrain`**: Write action state from input systems
//! - **`InputActionReader`**: React to changes in action state
//!
//! # Basic Usage
//!
//! 1. Define your input action type (must implement `InputAction`)
//! 2. Add the `InputActionPlugin` for your type
//! 3. Write input systems that pour values into the `InputActionDrain`
//! 4. Read the action state using `InputActionState` (or `InputActionReader`
//!    for status updates)
//!
//! # System Ordering
//!
//! - Systems that write to `InputActionDrain` should run **before**
//!   `InputActionSystem`
//! - Systems that read from `InputActionState` typically run in
//!   `Update` (after `PreUpdate`)
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "conditions")]
pub mod conditions;

#[cfg(feature = "derive")]
pub use bevy_actify_derive::InputAction;

#[cfg(feature = "conditions")]
pub use conditions::{
    input_action_active, input_action_started, input_action_stopped, input_action_updated,
};

use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin, PreUpdate, SubApp},
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
    /// The input action has started at this frame.
    Started(&'e A),

    /// The input action had been active, but changed
    /// the value.
    Updated(&'e A),

    /// The input action has stopped at this frame.
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

/// Marker trait for all input actions.
pub trait InputAction: Send + Sync + Clone + PartialEq + 'static {}

/// Extension trait for [`App`] and [`SubApp`].
pub trait InputActionAppExt {
    /// Adds the input action to the app.
    ///
    /// This will register the resources and systems
    /// required for an input action to fully function
    /// within an app.
    fn add_input_action<A: InputAction>(&mut self);
}

impl InputActionAppExt for SubApp {
    fn add_input_action<A: InputAction>(&mut self) {
        self.init_resource::<internal::InputActionState<A>>();
        self.init_resource::<internal::InputActionDrain<A>>();

        self.add_event::<internal::InputActionUpdated<A>>();

        self.add_systems(
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

impl InputActionAppExt for App {
    fn add_input_action<A: InputAction>(&mut self) {
        self.main_mut().add_input_action::<A>();
    }
}

impl<A: InputAction> InputActionState<'_, A> {
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

impl<A: InputAction> InputActionDrain<'_, A> {
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

impl<A: InputAction> InputActionReader<'_, '_, A> {
    /// see [`EventReader::read`](bevy::ecs::event::EventReader::read).
    pub fn read(&mut self) -> impl ExactSizeIterator<Item = InputActionStatus<A>> {
        self.inner.read().map(|event| match event {
            internal::InputActionUpdated::Started(state) => InputActionStatus::Started(state),
            internal::InputActionUpdated::Updated(state) => InputActionStatus::Updated(state),
            internal::InputActionUpdated::Stopped => InputActionStatus::Stopped,
        })
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
