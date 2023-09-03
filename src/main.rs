use sqlx::postgres::PgPoolOptions;
use std::env;
use dotenvy::dotenv;
use sqlx::{Pool, Postgres};

#[derive(Debug)]
struct Table {
    name: String,
    fields: Vec<Field>
}

#[derive(Debug)]
struct Field {
    name: String,
    field_type: String,
    is_primary_key: bool,
    is_foreign_key: bool,
    reference_table: String
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    dotenv().expect(".env file not found");
    let pool: Pool<Postgres> = PgPoolOptions::new()
        .min_connections(5)
        .max_connections(5)
        .connect(&env::var("DATABASE_URL")?).await?;

    let mut tables = find_all_tables(&pool).await;

    for mut table in &mut tables {
        table.fields = find_table_attributes(&pool, table.name.clone()).await
    }
    
    find_primary_and_foreign_key(&pool, &mut tables).await;
    // println!("{:#?}", tables);


    Ok(())
}

async fn find_primary_and_foreign_key(pool: &sqlx::PgPool, tables: &mut Vec<Table>) {
    let res = sqlx::query!(
        r#"
            SELECT conrelid::regclass::varchar AS table_name,
                   pg_get_constraintdef(oid)
            FROM   pg_constraint
            WHERE  contype = 'f' or contype = 'p'
              AND    connamespace = 'public'::regnamespace
            ORDER  BY conrelid::regclass::text, contype DESC
        "#
    ).fetch_all(pool).await.expect("Fail to fetch foreign and primary key");



    for r in res {
        for t in &mut *tables {
            if t.name == r.table_name.clone() .expect("No value for table while getting PK and FK"){
                match r.pg_get_constraintdef {
                    Some(v) => {
                        if v.contains("PRIMARY KEY") {
                            let name_parse: &str = &v[13..v.len() -1];
                            for mut field in &mut t.fields {
                                if field.name == name_parse.to_string() {
                                    field.is_primary_key = true;
                                }
                            }
                        } else if v.contains("FOREIGN KEY") {
                            let foreign_key_name: &str = &v[13..')'];
                            println!("{}", foreign_key_name)
                        }
                    },
                    None => ()
                }
                break;
            }
        }
    }
}

fn parse_foreign_key(text: String, t: &Table) {
    // let table_referenced: &str
}

async fn find_all_tables(pool: &sqlx::PgPool) -> Vec<Table> {
    let res = sqlx::query!(
        r#"
            SELECT table_name FROM information_schema.tables where table_schema = 'public'
        "#
    ).fetch_all(pool).await.expect("Fail to fetch table list");
    let mut tables: Vec<Table> = vec![];
    for r in res {
        let mut table: Table = Table { name: "".to_string(), fields: vec![] };
        match r.table_name {
            Some(v) => table.name = v,
            None => ()
        }
        tables.push(table);
    }
    return tables
}

async fn find_table_attributes(pool: &sqlx::PgPool, table: String) -> Vec<Field> {
    let res = sqlx::query!(
        r#"
                SELECT column_name, data_type
                FROM information_schema.columns
                WHERE table_schema = 'public' AND table_name = $1
        "#, table
    ).fetch_all(pool).await.expect("Fail to fetch table list");
    let mut fields: Vec<Field> = vec![];
    for r in res {
        fields.push(Field{
            name: r.column_name.expect("No name in that table"),
            field_type: r.data_type.expect("No datatype found in that table"),
            is_primary_key: false,
            is_foreign_key: false,
            reference_table: "".to_string(),
        })
    }

    return fields;
}

// struct Search {
//     id: i64,
//     username: Option<String>,
//     min_age: Option<i8>,
//     max_age: Option<i8>,
// }
//
// fn search_query(search: Search) -> String {
//     let mut query = sqlx::QueryBuilder::new("SELECT * from users where id = ");
//     query.push_bind(search.id);
//
//     if let Some(username) = search.username {
//         query.push(" AND username = ");
//         query.push_bind(username);
//     }
//
//     if let Some(min_age) = search.min_age {
//         query.push(" AND age > ");
//         query.push_bind(min_age);
//     }
//
//     if let Some(max_age) = search.max_age {
//         query.push(" AND age < ");
//         query.push_bind(max_age);
//     }
//
//     query.build().sql().into()
// }

// fn main() {
//     dbg!(search_query(Search {
//         id: 12,
//         username: None,
//         min_age: None,
//         max_age: None,
//     })); // "SELECT * from users where id = $1"
//     dbg!(search_query(Search {
//         id: 12,
//         username: Some("Bob".into()),
//         min_age: None,
//         max_age: None,
//     })); // "SELECT * from users where id = $1 AND username = $2"
//     dbg!(search_query(Search {
//         id: 12,
//         username: Some("Bob".into()),
//         min_age: Some(10),
//         max_age: Some(70),
//     })); // "SELECT * from users where id = $1 AND username = $2 AND age > $3 AND age < $4"
// }