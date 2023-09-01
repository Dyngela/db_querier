use sqlx::postgres::PgPoolOptions;
use std::env;
use dotenvy::dotenv;
use sqlx::{Pool, Postgres};


#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    dotenv().expect(".env file not found");
    let pool: Pool<Postgres> = PgPoolOptions::new()
        .max_connections(5)
        .min_connections(5)
        .connect(&env::var("DATABASE_URL")?).await?;
    let res = sqlx::query!(
        r#"
            SELECT * FROM information_schema.tables where table_schema = 'public'
        "#
    ).fetch_all(&pool).await?;

    for r in res {
        println!("{:?}", r)
    }

    Ok(())
}
