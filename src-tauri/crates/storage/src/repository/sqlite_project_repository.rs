use std::path::Path;
use std::sync::MutexGuard;

use prompt_core::{AppError, CreateProjectInput, Project, ProjectRepository, UpdateProjectInput};
use rusqlite::{OptionalExtension, Row};
use uuid::Uuid;

use crate::sqlite::{Connection, Db};
use crate::util::{db_err, now_timestamp};

const PROJECT_COLUMNS: &str = "id, name, description, color, created_at, updated_at";

/// SQLite 实现的项目仓库（M2 组织层）。
pub struct SqliteProjectRepository {
    db: Db,
}

impl SqliteProjectRepository {
    pub fn open_in_memory() -> Result<Self, AppError> {
        Ok(Self::new(Db::open_in_memory()?))
    }

    pub fn open(path: &Path) -> Result<Self, AppError> {
        Ok(Self::new(Db::open(path)?))
    }

    pub fn new(db: Db) -> Self {
        Self { db }
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, AppError> {
        self.db.lock()
    }
}

impl ProjectRepository for SqliteProjectRepository {
    fn find_all(&self) -> Result<Vec<Project>, AppError> {
        let conn = self.lock()?;
        let sql = format!("SELECT {PROJECT_COLUMNS} FROM projects ORDER BY created_at ASC");
        let mut stmt = conn.inner().prepare(&sql).map_err(db_err)?;
        let rows = stmt.query_map([], row_to_project).map_err(db_err)?;
        rows.map(|r| r.map_err(db_err)).collect()
    }

    fn create(&self, input: &CreateProjectInput) -> Result<Project, AppError> {
        let now = now_timestamp();
        let project = Project {
            id: Uuid::new_v4().to_string(),
            name: input.name.clone(),
            description: input.description.clone(),
            color: input.color.clone(),
            created_at: now.clone(),
            updated_at: now,
        };

        let conn = self.lock()?;
        conn.inner()
            .execute(
                "INSERT INTO projects (id, name, description, color, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    project.id,
                    project.name,
                    project.description,
                    project.color,
                    project.created_at,
                    project.updated_at,
                ],
            )
            .map_err(db_err)?;

        Ok(project)
    }

    fn update(&self, id: &str, input: &UpdateProjectInput) -> Result<Project, AppError> {
        let conn = self.lock()?;

        let sql = format!("SELECT {PROJECT_COLUMNS} FROM projects WHERE id = ?1");
        let mut project = conn
            .inner()
            .query_row(&sql, [id], row_to_project)
            .optional()
            .map_err(db_err)?
            .ok_or_else(|| AppError::NotFound(id.to_string()))?;

        if let Some(v) = &input.name {
            project.name = v.clone();
        }
        if let Some(v) = &input.description {
            project.description = Some(v.clone());
        }
        if let Some(v) = &input.color {
            project.color = Some(v.clone());
        }
        project.updated_at = now_timestamp();

        conn.inner()
            .execute(
                "UPDATE projects SET name = ?2, description = ?3, color = ?4, updated_at = ?5 \
                 WHERE id = ?1",
                rusqlite::params![
                    project.id,
                    project.name,
                    project.description,
                    project.color,
                    project.updated_at,
                ],
            )
            .map_err(db_err)?;

        Ok(project)
    }

    fn delete(&self, id: &str) -> Result<(), AppError> {
        let mut conn = self.lock()?;
        let tx = conn.inner_mut().transaction().map_err(db_err)?;
        let affected = tx
            .execute("DELETE FROM projects WHERE id = ?1", [id])
            .map_err(db_err)?;
        if affected == 0 {
            return Err(AppError::NotFound(id.to_string()));
        }
        // 清理该项目的提示词关联，避免悬挂引用。
        tx.execute("DELETE FROM prompt_projects WHERE project_id = ?1", [id])
            .map_err(db_err)?;
        tx.commit().map_err(db_err)?;
        Ok(())
    }
}

fn row_to_project(row: &Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        color: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}
