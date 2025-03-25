pub mod plugin;

pub use plugin::{InputActionPlugin, InputActionSystem};

use bevy::{
    ecs::system::{Res, ResMut, SystemParam},
    prelude::EventReader,
};

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

pub(crate) mod internal {
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
