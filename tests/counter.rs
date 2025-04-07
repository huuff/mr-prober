use std::sync::{Arc, Mutex};

use mr_prober::{proc::Processor, Prober};
use rand::distributions::DistString;

#[tokio::test]
async fn in_memory() {
    // ARRANGE
    let counter = Arc::new(Mutex::new(Counter::default()));

    let mut prober = Prober::in_memory(CounterProcessor::new(Arc::clone(&counter)));

    // ACT
    for _ in 0..10 {
        prober.probe().await.expect_ok();
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
    let mut prober = Prober::from_file(&file_path, CounterProcessor::new(Arc::clone(&counter)))
        .await
        .unwrap();

    // ACT
    for _ in 0..10 {
        prober.probe().await.expect_ok();
    }

    // ASSERT
    assert_eq!(
        counter.lock().unwrap().interactions,
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
    );
}

#[tokio::test]
async fn auto_prober() {
    // ARRANGE
    let counter = Arc::new(Mutex::new(Counter::default()));

    let prober = Prober::in_memory(CounterProcessor::new(Arc::clone(&counter)));

    // ACT
    prober.into_auto(Default::default()).spawn().await.unwrap();

    // ASSERT
    assert_eq!(
        counter.lock().unwrap().interactions,
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
    );
}

#[derive(Default)]
struct Counter {
    interactions: Vec<u64>,
}

impl Counter {
    /// Pushes `val` into the interactions and returns the number of interactions so far
    fn interact(&mut self, val: u64) -> u64 {
        self.interactions.push(val);
        self.interactions.len().try_into().unwrap()
    }
}

pub struct CounterProcessor {
    counter: Arc<Mutex<Counter>>,
}

impl CounterProcessor {
    fn new(counter: Arc<Mutex<Counter>>) -> Self {
        Self { counter }
    }
}

#[async_trait::async_trait]
impl Processor for CounterProcessor {
    type Sentinel = u64;

    async fn next(
        &self,
        current: Option<u64>,
    ) -> Result<Option<u64>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        if current.is_some_and(|it| it >= 10) {
            return Ok(None);
        }

        let next = self.counter.lock().unwrap().interact(current.unwrap_or(0));

        Ok(Some(next))
    }
}
