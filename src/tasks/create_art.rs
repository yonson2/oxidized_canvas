use loco_rs::errors::Error;
use loco_rs::prelude::*;

use crate::services::art_service;

pub struct CreateArt;
#[async_trait]
impl Task for CreateArt {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "create_art".to_string(),
            detail: "Task generator".to_string(),
        }
    }
    async fn run(&self, ctx: &AppContext, _vars: &task::Vars) -> Result<()> {
        let art = art_service::create_art(ctx)
            .await
            .map_err(|e| Error::Message(format!("Unable to create art: {e}")))?;

        println!("Created art: {} - {}", art.id, art.title);
        Ok(())
    }
}
