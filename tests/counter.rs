use std::{
    convert::Infallible,
    sync::{Arc, Mutex},
};

use mr_prober::{Prober as _, ProberImpl};
use rand::distributions::DistString;

#[derive(Default)]
struct Counter {
    pub interactions: Vec<u64>,
}

impl Counter {
    /// Pushes `val` into the interactions and returns the number of interactions so far
    pub fn interact(&mut self, val: u64) -> u64 {
        self.interactions.push(val);
        self.interactions.len().try_into().unwrap()
    }
}

#[tokio::test]
async fn in_memory() {
    // ARRANGE
    let counter = Arc::new(Mutex::new(Counter::default()));

    let mut prober = ProberImpl::in_memory(|sentinel: Option<u64>| {
        let counter = Arc::clone(&counter);
        async move {
            let next = counter.lock().unwrap().interact(sentinel.unwrap_or(0));

            Ok::<_, Infallible>(Some(next))
        }
    });

    // ACT
    for _ in 0..10 {
        prober.probe().await.unwrap();
    }

    // ASSERT
    assert_eq!(
        counter.lock().unwrap().interactions,
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
    );
}

#[tokio::test]
async fn in_file() {
    // ARRANGE
    let counter = Arc::new(Mutex::new(Counter::default()));

    let test_id = rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 10);
    let file_path = format!("/tmp/mrprober-test-{test_id}");
    let mut prober = ProberImpl::from_file(&file_path, |sentinel: Option<u64>| {
        let counter = Arc::clone(&counter);
        async move {
            let next = counter.lock().unwrap().interact(sentinel.unwrap_or(0));

            Ok::<_, Infallible>(Some(next))
        }
    })
    .await
    .unwrap();

    // ACT
    for _ in 0..10 {
        prober.probe().await.unwrap();
    }

    // ASSERT
    assert_eq!(
        counter.lock().unwrap().interactions,
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
    );
}
