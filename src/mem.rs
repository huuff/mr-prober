use std::marker::PhantomData;

use crate::{Prober, ProberRetriever};

pub struct MemoryBackedProber<Item, Sentinel, Retriever> {
    last_sentinel: Option<Sentinel>,
    retriever: Retriever,
    _item: PhantomData<Item>,
}

impl<Item, Sentinel, Retriever> MemoryBackedProber<Item, Sentinel, Retriever> {
    pub fn new(retriever: Retriever) -> Self {
        Self {
            last_sentinel: None,
            retriever,
            _item: PhantomData,
        }
    }
}

impl<Item, Sentinel, Retriever> Prober for MemoryBackedProber<Item, Sentinel, Retriever>
where
    Retriever: ProberRetriever<Item, Sentinel>,
{
    type Item = Item;
    type Sentinel = Sentinel;

    async fn next(&self) -> Option<Self::Item> {
        self.retriever.next(self.last_sentinel.as_ref()).await
    }

    async fn commit(&mut self, sentinel: Self::Sentinel) {
        self.last_sentinel.replace(sentinel);
    }
}
