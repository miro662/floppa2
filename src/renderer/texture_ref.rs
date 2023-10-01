use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TextureRef(usize);
pub(crate) struct TextureRefManager {
    next_id: AtomicUsize,
}

impl TextureRefManager {
    pub(crate) fn new() -> TextureRefManager {
        TextureRefManager {
            next_id: AtomicUsize::new(0),
        }
    }

    pub(crate) fn next(&self) -> TextureRef {
        let texture_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        TextureRef(texture_id)
    }
}
