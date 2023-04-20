use sqlx::postgres::{PgPool, PgRow};
use sqlx::{FromRow, Row};
use uuid::Uuid;


#[derive(Debug, FromRow)]
pub struct Users {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    //created_at: DateTime<Utc>,
    //updated_at: DateTime<Utc>,
}

impl Users {
    pub fn new(name: String, email: String, password_hash: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            name,
            email,
            password_hash,
        }
    }

    fn from_row(row: &PgRow) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
        }
    }

    pub async fn find_by_id(id: Uuid, pool: &PgPool) -> Result<Users, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;

        Ok(Users::from_row(&row))
    }

    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Users, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_one(pool)
            .await?;

        Ok(Users::from_row(&row))
    }

    pub async fn update_by_id(
        id: Uuid,
        name: Option<String>,
        email: Option<String>,
        password_hash: Option<String>,
        pool: &PgPool,
    ) -> Result<Users, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let user = sqlx::query("SELECT * FROM users WHERE id = $1 for update")
            .bind(id)
            .fetch_one(&mut tx)
            .await?;

        let row = sqlx::query(
            "UPDATE users SET name = $1, email = $2, password_hash = $3 WHERE id = $4 RETURNING *",
        )
        .bind(name.unwrap_or(user.get("name")))
        .bind(email.unwrap_or(user.get("email")))
        .bind(password_hash.unwrap_or(user.get("password_hash")))
        .bind(id)
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(Users::from_row(&row))
    }

    pub async fn insert(
        &self,
        pool: &PgPool,
    ) -> Result<Users, sqlx::Error> {
        let row = sqlx::query(
            "INSERT INTO users (id, name, password_hash, email) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(&self.id)
        .bind(&self.name)
        .bind(&self.password_hash)
        .bind(&self.email)
        .fetch_one(pool)
        .await?;

        Ok(Users::from_row(&row))
    }
}
