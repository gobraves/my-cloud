//use crate::db_schema::file_history::FileHistory;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{FromRow, Row};
use uuid::Uuid;

//use super::file_history;

#[derive(Debug, FromRow)]
pub struct Files {
    pub id: i64,
    pub uid: Uuid,
    pub filename: String,
    pub parent_dir_id: i64,
    pub is_deleted: bool,
    pub size: i64,
    pub is_dir: bool,
    pub version: i64,
    //created_at: DateTime<Utc>,
    //updated_at: DateTime<Utc>,
}

impl Files {
    pub fn  new(id: i64, uid: Uuid, filename: String, parent_dir_id: i64, size: i64, is_dir: bool) -> Self {
        Files {
            id,
            uid,
            filename,
            parent_dir_id,
            size,
            is_dir,
            is_deleted: false,
            version: 1,
        }
    }

    pub fn from_row(row: &PgRow) -> Files {
        Files {
            id: row.get("id"),
            uid: row.get("uid"),
            filename: row.get("filename"),
            parent_dir_id: row.get("parent_dir_id"),
            size: row.get("size"),
            is_dir: row.get("is_dir"),
            is_deleted: row.get("is_delete"),
            version: row.get("version"),
        }
    }

    // check if file exists
    pub async fn get_by_parent_dir_id_and_uid_and_filename(
        parent_dir_id: i64,
        uid: Uuid,
        filename: &str,
        pool: &PgPool,
    ) -> Result<Files, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM files WHERE parent_dir_id = $1 and uid = $2 and filename = $3 and is_delete = false")
            .bind(parent_dir_id)
            .bind(uid)
            .bind(filename)
            .fetch_one(pool)
            .await?;
        Ok(Files::from_row(&row))
    }

    // get file with file history by uid, id
    pub async fn get_by_uid_and_id(
        uid: Uuid,
        id: i64,
        pool: &PgPool,
    ) -> Result<PgRow, sqlx::Error> {
        let row = sqlx::query("SELECT a.id, a.filename, a.size, a.is_dir, b.slices, b.slices_hash FROM file as a join file_history as b on a.id = b.id and a.version = b.version WHERE a.uid = $1 and a.id = $2 and is_delete = false")
            .bind(uid)
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(row)
    }

    pub async fn get_by_parent_dir_id_and_uid(
        parent_dir_id: i64,
        uid: Uuid,
        pool: &PgPool,
    ) -> Result<Vec<Files>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT * FROM files WHERE parent_dir_id = $1 and uid = $2 and is_delete = false",
        )
        .bind(parent_dir_id)
        .bind(uid)
        .fetch_all(pool)
        .await?;

        let mut files = Vec::new();
        for row in rows {
            files.push(Files::from_row(&row));
        }

        Ok(files)
    }

    // check file or dir if owned by user
    pub async fn check_owner(uid: Uuid, id: i64, pool: &PgPool) -> Result<Option<Files>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM files WHERE uid = $1 and id = $2 and is_delete = false")
            .bind(uid)
            .bind(id)
            .fetch_optional(pool)
            .await?;
        match row {
            Some(row) => {
                Ok(Some(Files::from_row(&row)))

            },
            None => Ok(None),
        }

    }

    pub async fn insert_dir(
        id: i64,
        uid: Uuid,
        dir_name: &str,
        parent_dir_id: i64,
        pool: &PgPool,
    ) -> Result<Files, sqlx::Error> {
        let row = sqlx::query("INSERT INTO files (id, uid, filename, parent_dir_id, is_dir) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(id)
            .bind(uid)
            .bind(dir_name)
            .bind(parent_dir_id)
            .bind(true)
            .fetch_one(pool)
            .await?;

        Ok(Files::from_row(&row))
    }

    pub async fn insert_file(
        &self,
        slice: Vec<String>,
        slices_hash: Vec<String>,
        pool: &PgPool,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        let row = sqlx::query("INSERT INTO files (id, uid, filename, parent_dir_id, size, is_dir) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *")
            .bind(self.id)
            .bind(self.uid)
            .bind(&self.filename)
            .bind(self.parent_dir_id)
            .bind(self.size)
            .bind(self.is_dir)
            .fetch_one(&mut tx)
            .await?;

        sqlx::query("INSERT INTO file_history (id, file_version, slices, slices_hash) VALUES ($1, $2, $3, $4)")
            .bind(self.id)
            .bind(self.version)
            .bind(slice)
            .bind(slices_hash)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn update_file_info(&self, filename: &str, pool: &PgPool) -> Result<Files, sqlx::Error> {
        let mut tx = pool.begin().await?;
        sqlx::query("select * from files where id = $1 for update")
            .bind(self.id)
            .fetch_one(&mut tx)
            .await?;

        let row = sqlx::query("UPDATE files SET filename = $1 WHERE id = $4 RETURNING *")
            .bind(filename)
            .bind(self.id)
            .fetch_one(&mut tx)
            .await?;

        tx.commit().await?;

        Ok(Files::from_row(&row))
    }

    // update file version when file content is updated
    pub async fn update_file_version(&self, slice: Vec<String>, slice_hash: Vec<String>, pool: &PgPool) -> Result<Files, sqlx::Error> {
        let mut tx = pool.begin().await?;

        sqlx::query("select * from files where id = $1 for update")
            .bind(self.id)
            .fetch_one(&mut tx)
            .await?;

        let row = sqlx::query("UPDATE files SET version = version + 1 WHERE id = $1 and version = $2 and uid = $3 RETURNING *")
            .bind(self.id)
            .bind(self.version)
            .bind(self.uid)
            .fetch_one(&mut tx)
            .await?;
        
        sqlx::query("INSERT INTO file_history (id, file_version, slices, slices_hash) VALUES ($1, $2, $3, $4)")
            .bind(self.id)
            .bind(self.version)
            .bind(slice)
            .bind(slice_hash)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;

        Ok(Files::from_row(&row))
    }

    pub async fn delete(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let _ = sqlx::query(
            "UPDATE files SET is_delete = true WHERE id = $1 and uid = $2 RETURNING *",
        )
        .bind(self.id)
        .bind(self.uid)
        .fetch_one(pool)
        .await?;

        Ok(())
    }
}
