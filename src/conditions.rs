use crate::{InputAction, InputActionState};

/// Returns `true` if the input action [`A`] is active.
pub fn input_action_active<A: InputAction>(action: InputActionState<A>) -> bool {
    action.is_active()
}
