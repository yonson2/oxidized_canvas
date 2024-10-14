use chrono::{DateTime, Utc};
use loco_rs::prelude::*;
use sea_orm::{Database, DbBackend, FromQueryResult, Statement};
use serde::{Deserialize, Serialize};

use crate::models::arts::{self, ArtParams};

pub struct ImportArts;
#[async_trait]
impl Task for ImportArts {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "import_arts".to_string(),
            detail: "Task generator".to_string(),
        }
    }
    async fn run(&self, ctx: &AppContext, _vars: &task::Vars) -> Result<()> {
        let db: DatabaseConnection = Database::connect("").await?;
        let unique: Vec<Art> = Art::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT * FROM arts ORDER BY id ASC"#,
            [],
        ))
        .all(&db)
        .await?;

        for a in unique {
            arts::Model::create(
                &ctx.db,
                &ArtParams {
                    title: a.title.clone(),
                    image: a.image,
                    prompt: a.prompt,
                },
            )
            .await?;
            println!("Succesfully imported {} - {}", a.id, a.title);
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, FromQueryResult)]
struct Art {
    pub id: i64,
    pub prompt: String,
    pub title: String,
    pub image: String,
    pub created_at: DateTime<Utc>,
    pub uuid: Uuid,
}
