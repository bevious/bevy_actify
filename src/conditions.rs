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
    let has_started = action
        .read()
        .any(|status| matches!(status, InputActionStatus::Started(_)));
    action.clear();
    has_started
}

/// Returns `true` if the input action [`A`] has just updated.
pub fn input_action_updated<A: InputAction>(mut action: InputActionReader<A>) -> bool {
    let has_updated = action
        .read()
        .any(|status| matches!(status, InputActionStatus::Updated(_)));
    action.clear();
    has_updated
}

/// Returns `true` if the input action [`A`] has just stopped.
pub fn input_action_stopped<A: InputAction>(mut action: InputActionReader<A>) -> bool {
    let has_stopped = action
        .read()
        .any(|status| matches!(status, InputActionStatus::Stopped));
    action.clear();
    has_stopped
}
