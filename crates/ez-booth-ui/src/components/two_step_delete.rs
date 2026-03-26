use leptos::*;
/// Shared state manager for two-step deletion interactions (arm -> confirm -> reset).
#[derive(Clone)]
pub struct TwoStepDeleteController<T>
where
    T: PartialEq + Clone + 'static,
{
    armed: RwSignal<Option<T>>,
}

impl<T> TwoStepDeleteController<T>
where
    T: PartialEq + Clone + 'static,
{
    pub fn new() -> Self {
        Self {
            armed: create_rw_signal(None),
        }
    }

    pub fn arm(&self, target: T) {
        if self.armed.get() == Some(target.clone()) {
            return;
        }
        self.armed.set(Some(target));
    }

    pub fn is_armed(&self, target: &T) -> bool {
        self.armed.get().as_ref() == Some(target)
    }

    pub fn confirm_with<F>(&self, target: &T, mut action: F)
    where
        F: FnMut(),
    {
        if self.is_armed(target) {
            action();
            self.reset();
        }
    }

    pub fn reset(&self) {
        self.armed.set(None);
    }

    pub fn signal(&self) -> RwSignal<Option<T>> {
        self.armed
    }
}

pub fn use_two_step_delete<T>() -> TwoStepDeleteController<T>
where
    T: PartialEq + Clone + 'static,
{
    TwoStepDeleteController::new()
}
