use crate::registry::state::LOADING;

pub(super) struct LoadingGuard;

impl Drop for LoadingGuard {
    fn drop(&mut self) {
        LOADING.set(false);
    }
}
