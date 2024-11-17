use mr_prober::Prober;
use rand::distributions::DistString;
use sqlx::Row;

#[sqlx::test]
async fn in_memory_sqlx_test(db: sqlx::PgPool) {
    // ARRANGE
    sqlx::query!("CREATE SEQUENCE test_seq")
        .execute(&db)
        .await
        .unwrap();

    let mut prober = Prober::in_memory(|_sentinel: Option<i64>| async {
        sqlx::query("SELECT nextval('test_seq') AS next")
            .fetch_one(&db)
            .await
            .map(|row| row.get(0))
    });

    // ACT
    for _ in 0..10 {
        prober.probe().await.unwrap();
    }

    // ASSERT
    assert_eq!(prober.current().await.unwrap(), Some(10));
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
            .map(|row| row.get(0))
    };

    let test_id = rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 10);
    let file_path = format!("/tmp/mrprober-test-{test_id}");
    let mut prober = Prober::from_file(&file_path, processor).await.unwrap();

    // ACT
    for _ in 0..10 {
        prober.probe().await.unwrap();
    }

    // ASSERT
    assert_eq!(prober.current().await.unwrap(), Some(10));
}
