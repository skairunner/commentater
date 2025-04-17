use crate::db::pgacquire::PgAcquire;
use std::env;

pub async fn add_test_data<'a, A: PgAcquire<'a>>(conn: A) {
    let mut conn = conn.acquire().await.unwrap();
    // Insert a WA user
    sqlx::query!(
        "INSERT INTO wa_user(id, worldanvil_id, name) OVERRIDING SYSTEM VALUE VALUES(5, '225bd01d-124c-4aa2-885b-0fc4bdf41bd8', 'nnie');",
    )
        .execute(&mut *conn)
        .await
        .unwrap();
    // Insert a commentator user
    sqlx::query!(
        "INSERT INTO commentater_user(id, worldanvil_id, api_key) OVERRIDING SYSTEM VALUE VALUES (5, 5, $1);",
        Some(env::var("TEST_USER_KEY").unwrap())
    )
    .execute(&mut *conn)
    .await
    .unwrap();
    sqlx::query!("INSERT INTO user_queue(user_id) VALUES (5);")
        .execute(&mut *conn)
        .await
        .unwrap();
    // Insert a world
    sqlx::query!("INSERT INTO world(id, user_id, worldanvil_id, name) OVERRIDING SYSTEM VALUE VALUES (5, 5, 'e69d6a36-2d22-4bf2-80f9-456a9b0d909e', 'Solaris');")
        .execute(&mut *conn)
        .await
        .unwrap();
    // Insert a few articles
    sqlx::query!(
        "
        INSERT INTO article(user_id, world_id, title, url)
        SELECT 5, 5, *
        FROM UNNEST($1::text[], $2::text[]);
        ",
        &[
            "A Day in the Caloris Basin".to_string(),
            "About Solaris".to_string()
        ],
        &[
            "https://www.worldanvil.com/w/solaris-nnie/a/a-day-in-the-caloris-basin-law"
                .to_string(),
            "https://www.worldanvil.com/w/solaris-nnie/a/about-solaris-article".to_string()
        ]
    )
    .execute(&mut *conn)
    .await
    .unwrap();
}
