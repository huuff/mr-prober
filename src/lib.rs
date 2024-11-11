use std::{future::Future, marker::PhantomData};

pub trait Poller {
    type Item;
    type Sentinel;

    fn next(&self) -> impl Future<Output = Option<Self::Item>>;

    fn commit(&mut self, sentinel: Self::Sentinel) -> impl Future<Output = ()>;
}

pub trait PollerRetriever<Item, Sentinel> {
    fn next(&self, sentinel: Option<&Sentinel>) -> impl Future<Output = Option<Item>>;
}

impl<Item, Sentinel, F, Fut> PollerRetriever<Item, Sentinel> for F
where
    F: Fn(Option<&Sentinel>) -> Fut,
    Fut: Future<Output = Option<Item>>,
{
    fn next(&self, sentinel: Option<&Sentinel>) -> impl Future<Output = Option<Item>> {
        (self)(sentinel)
    }
}

pub struct InMemoryPoller<Item, Sentinel, Retriever> {
    last_sentinel: Option<Sentinel>,
    retriever: Retriever,
    _item: PhantomData<Item>,
}

impl<Item, Sentinel, Retriever> InMemoryPoller<Item, Sentinel, Retriever> {
    pub fn new(retriever: Retriever) -> Self {
        Self {
            last_sentinel: None,
            retriever,
            _item: PhantomData,
        }
    }
}

impl<Item, Sentinel, Retriever> Poller for InMemoryPoller<Item, Sentinel, Retriever>
where
    Retriever: PollerRetriever<Item, Sentinel>,
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
