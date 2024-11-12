use crate::SentinelStorage;

#[derive(Default)]
pub struct MemorySentinelStorage<Sentinel> {
    sentinel: Option<Sentinel>,
}

impl<Sentinel: Clone> SentinelStorage<Sentinel> for MemorySentinelStorage<Sentinel> {
    async fn current(&self) -> Option<Sentinel> {
        self.sentinel.clone()
    }

    async fn commit(&mut self, sentinel: Sentinel) {
        self.sentinel.replace(sentinel);
    }
}
