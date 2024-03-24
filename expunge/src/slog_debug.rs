use std::cell::RefCell;
use std::sync::OnceLock;

thread_local! {
    static TL_SCOPES: RefCell<Vec<bool>> = RefCell::new(Vec::new())
}

static DISABLED: OnceLock<bool> = OnceLock::new();

/// A type guard for disabling expunging within slog. Other calls to expunge will be unaffected.
pub struct DisabledGuard;

impl DisabledGuard {
    /// A thread local type guard for disabling expunging within slog:
    ///   true = disabled
    ///
    /// When dropped, it will be reset to the parent value.
    ///
    /// Expunge is enabled by default.
    pub fn new(disabled: bool) -> Self {
        TL_SCOPES.with(|s| {
            s.borrow_mut().push(disabled);
        });

        DisabledGuard
    }
}

impl Drop for DisabledGuard {
    fn drop(&mut self) {
        TL_SCOPES.with(|s| {
            s.borrow_mut()
                .pop()
                .expect("TL_SCOPES should contain a logger");
        })
    }
}

pub fn disable_globally() -> Result<(), bool> {
    DISABLED.set(true)
}

pub fn is_disabled() -> bool {
    DISABLED.get().copied().unwrap_or_default()
        || TL_SCOPES.with(|s| {
            let s = s.borrow();
            match s.last() {
                Some(disabled) => *disabled,
                None => false,
            }
        })
}
