use mr_prober::{mem::MemorySentinelStorage, Prober};
use sqlx::Row;

#[sqlx::test]
async fn in_memory_sqlx_test(db: sqlx::PgPool) {
    // ARRANGE
    sqlx::query!("CREATE SEQUENCE test_seq")
        .execute(&db)
        .await
        .unwrap();

    let storage = MemorySentinelStorage::default();
    let processor = |_sentinel: Option<i64>| async {
        sqlx::query("SELECT nextval('test_seq') AS next")
            .fetch_one(&db)
            .await
            .unwrap()
            .get(0)
    };

    let mut prober = Prober::new(storage, processor);

    // ACT
    for _ in 0..10 {
        prober.probe().await
    }

    // ASSERT
    assert_eq!(prober.current().await, Some(10));
}

#[sqlx::test]
async fn file_sqlx_tst(db: sqlx::PgPool) {
    // ARRANGE
    sqlx::query!("CREATE SEQUENCE test_seq")
        .execute(&db)
        .await
        .unwrap();

    let processor = |_sentinel: Option<i64>| async {
        sqlx::query("SELECT nextval('test_seq') AS next")
            .fetch_one(&db)
            .await
            .unwrap()
            .get(0)
    };

    let mut prober = Prober::from_file("/tmp/testorage.txt", processor).await;

    // ACT
    for _ in 0..10 {
        prober.probe().await
    }

    // ASSERT
    assert_eq!(prober.current().await, Some(10));
}
