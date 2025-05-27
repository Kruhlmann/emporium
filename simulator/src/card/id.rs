use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalCardId(u64);

impl GlobalCardId {
    pub fn new() -> Self {
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn reset() {
        COUNTER.store(0, Ordering::SeqCst);
    }
}

impl Default for GlobalCardId {
    fn default() -> Self {
        GlobalCardId::new()
    }
}

impl std::fmt::Display for GlobalCardId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

impl std::fmt::Debug for GlobalCardId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GlobalCardId(0x{:x})", self.0)
    }
}
