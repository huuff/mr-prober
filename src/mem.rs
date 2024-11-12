use crate::{Prober, ProberRetriever};

pub struct MemoryBackedProber< Sentinel, Retriever> {
    last_sentinel: Option<Sentinel>,
    retriever: Retriever,
}

impl<Sentinel, Retriever> MemoryBackedProber<Sentinel, Retriever> {
    pub fn new(retriever: Retriever) -> Self {
        Self {
            last_sentinel: None,
            retriever,
        }
    }
}

impl<Item, Sentinel, Retriever> Prober<Item, Sentinel> for MemoryBackedProber<Sentinel, Retriever>
where
    Retriever: ProberRetriever<Item, Sentinel>,
{
    async fn next(&self) -> Option<Item> {
        self.retriever.next(self.last_sentinel.as_ref()).await
    }

    async fn commit(&mut self, sentinel: Sentinel) {
        self.last_sentinel.replace(sentinel);
    }
}
