thread_local! {
    pub(super) static LOADING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(crate) fn is_loading() -> bool {
    LOADING.get()
}
