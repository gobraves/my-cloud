//use crate::db_schema::file_history::FileHistory;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{FromRow, Row};
use uuid::Uuid;
use serde;

//use super::file_history;

#[derive(Debug, FromRow, serde::Serialize, serde::Deserialize)]
pub struct Workspaces {
    pub id: Uuid,
    pub uid: Uuid,
    pub name: String,
    pub sync: bool,
    //created_at: DateTime<Utc>,
    //updated_at: DateTime<Utc>,
}

impl Workspaces {
    fn from_row(row: &PgRow) -> Workspaces {
        Workspaces {
            id: row.get("id"),
            name: row.get("name"),
            uid: row.get("uid"),
            sync: row.get("sync"),
        }
    }

    pub fn new(id: Uuid, name: String, uid: Uuid, sync: bool) -> Self {
        Workspaces {
            id,
            name,
            uid,
            sync,
        }
    }

    pub async fn get(id: Uuid, uid: Uuid, pool: &PgPool) -> Result<Workspaces, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM workspaces WHERE id = $1 and uid = $2")
            .bind(id)
            .bind(uid)
            .fetch_one(pool)
            .await?;

        Ok(Workspaces::from_row(&row))
    }

    pub async fn insert(&self, pool: &PgPool) -> Result<Workspaces, sqlx::Error> {
        let row = sqlx::query(
            "INSERT INTO workspaces (id, name, uid, sync) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(self.id)
        .bind(&self.name)
        .bind(self.uid)
        .bind(self.sync)
        .fetch_one(pool)
        .await?;

        Ok(Workspaces::from_row(&row))
    }

    pub async fn update(&self, pool: &PgPool) -> Result<Workspaces, sqlx::Error> {
        let row = sqlx::query(
            "UPDATE workspaces SET name = $1 WHERE id = $2 RETURNING *",
        )
        .bind(&self.name)
        .bind(self.id)
        .fetch_one(pool)
        .await?;

        Ok(Workspaces::from_row(&row))
    }

    pub async fn delete(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM workspaces WHERE id = $1",
        )
        .bind(self.id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_workspace_list(uid: Uuid, pool: &PgPool) -> Result<Vec<Workspaces>, sqlx::Error> {
        let rows = sqlx::query("SELECT * FROM workspaces WHERE uid = $1")
            .bind(uid)
            .fetch_all(pool)
            .await?;

        let mut workspaces = Vec::new();
        for row in rows {
            workspaces.push(Workspaces::from_row(&row));
        }
        Ok(workspaces)
    }
}