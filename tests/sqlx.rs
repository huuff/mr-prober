use mr_prober::{mem::InMemoryPoller, Poller};
use sqlx::Row;

#[sqlx::test]
async fn in_memory_sqlx_test(db: sqlx::PgPool) {
    // ARRANGE
    sqlx::query!("CREATE SEQUENCE test_seq")
        .execute(&db)
        .await
        .unwrap();

    let mut poller = InMemoryPoller::<i64, i64, _>::new(move |_last: Option<&i64>| {
        let db = db.clone();
        async move {
            let res = sqlx::query("SELECT nextval('test_seq') AS next")
                .fetch_one(&db)
                .await
                .unwrap();
            Some(res.get::<i64, &str>("next"))
        }
    });

    for i in 0..10 {
        assert_eq!(i + 1, poller.next().await.unwrap());
        poller.commit(i).await;
    }
}
