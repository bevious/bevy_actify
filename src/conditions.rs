//! SPDX-License-Identifier: MIT OR Apache-2.0
//!
//! Condition helpers for checking input action states.
//!
//! These functions provide convenient ways to check the state of input actions
//! in Bevy systems, particularly useful for system conditions.

use crate::{InputAction, InputActionReader, InputActionState, InputActionStatus};

/// Returns `true` if the input action [`A`] is active.
pub fn input_action_active<A: InputAction>(action: InputActionState<A>) -> bool {
    action.is_active()
}

/// Returns `true` if the input action [`A`] has just started.
pub fn input_action_started<A: InputAction>(mut action: InputActionReader<A>) -> bool {
    action
        .read()
        .last()
        .map_or(false, |a| matches!(a, InputActionStatus::Started(_)))
}

/// Returns `true` if the input action [`A`] has just updated.
pub fn input_action_updated<A: InputAction>(mut action: InputActionReader<A>) -> bool {
    action
        .read()
        .last()
        .map_or(false, |a| matches!(a, InputActionStatus::Updated(_)))
}

/// Returns `true` if the input action [`A`] has just stopped.
pub fn input_action_stopped<A: InputAction>(mut action: InputActionReader<A>) -> bool {
    action
        .read()
        .last()
        .map_or(false, |a| matches!(a, InputActionStatus::Stopped))
}
