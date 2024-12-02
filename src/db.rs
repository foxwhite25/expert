use crate::rule::Rule;
use anyhow::Result;
use sqlx::migrate::MigrateDatabase;
use sqlx::{Sqlite, SqlitePool};

#[derive(Clone, Debug)]
pub struct Database {
    conn: SqlitePool,
}

impl Database {
    pub async fn new(db_url: &str) -> Result<Self> {
        if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
            Sqlite::create_database(db_url).await?
        }
        let conn = SqlitePool::connect(db_url).await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                condition TEXT NOT NULL,
                output TEXT NOT NULL
            )",
        )
        .execute(&conn)
        .await?;

        Ok(Self { conn })
    }

    pub async fn save_rules(&self, rules: &[Rule]) -> Result<()> {
        for rule in rules {
            let condition = rule.condition.to_string();
            let output = rule.output.join(",");
            sqlx::query("INSERT INTO rules (condition, output) VALUES (?, ?)")
                .bind(condition)
                .bind(output)
                .execute(&self.conn)
                .await?;
        }
        Ok(())
    }

    pub async fn load_rules_raw(&self) -> Result<Vec<(i64, String, String)>> {
        let rows = sqlx::query!("SELECT id, condition, output FROM rules")
            .fetch_all(&self.conn)
            .await?;
        Ok(rows
            .into_iter()
            .map(|row| (row.id, row.condition, row.output))
            .collect())
    }

    pub async fn load_rules(&self) -> Result<Vec<Rule>> {
        self.load_rules_raw()
            .await?
            .into_iter()
            .map(Rule::try_from)
            .collect()
    }

    pub async fn reset(&self) -> Result<()> {
        sqlx::query("DROP TABLE IF EXISTS rules")
            .execute(&self.conn)
            .await?;

        sqlx::query(
            "CREATE TABLE rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                condition TEXT NOT NULL,
                output TEXT NOT NULL
            )",
        )
        .execute(&self.conn)
        .await?;

        Ok(())
    }
}
