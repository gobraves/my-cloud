use sqlx::postgres::{PgPool, PgRow};
use sqlx::{FromRow, Row};

#[derive(Debug, FromRow)]
pub struct FileHistories {
    id: i64,
    fid: i64,
    file_version: i64,
    slices: Vec<String>,
    slices_hash: Vec<String>,
    //created_at: DateTime<Utc>,
    //updated_at: DateTime<Utc>,
}

impl FileHistories {
    fn from_row(row: &PgRow) -> Self {
        Self {
            id: row.get("id"),
            fid: row.get("fid"),
            file_version: row.get("file_version"),
            slices: row.get("slices"),
            slices_hash: row.get("slices_hash"),
        }
    }

    pub async fn find_by_id(id: i64, pool: &PgPool) -> Result<FileHistories, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM file_histories WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;

        Ok(FileHistories::from_row(&row))
    }

    pub async fn find_by_fid(fid: i64, pool: &PgPool) -> Result<FileHistories, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM file_histories WHERE fid = $1")
            .bind(fid)
            .fetch_one(pool)
            .await?;

        Ok(FileHistories::from_row(&row))
    }

    pub async fn insert(
        fid: i64,
        file_version: i64,
        slices: Vec<String>,
        slices_hash: Vec<String>,
        pool: &PgPool,
    ) -> Result<FileHistories, sqlx::Error> {
        let row = sqlx::query(
            "INSERT INTO file_histories (fid, file_version, slices, slices_hash) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(fid)
        .bind(file_version)
        .bind(slices)
        .bind(slices_hash)
        .fetch_one(pool)
        .await?;

        Ok(FileHistories::from_row(&row))
    }
}


