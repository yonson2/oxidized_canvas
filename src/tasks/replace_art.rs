use loco_rs::prelude::*;

use crate::services::art_service;

pub struct ReplaceArt;
#[async_trait]
impl Task for ReplaceArt {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "replace_art".to_string(),
            detail: "Replaces an existing art with a new AI-generated one. Usage: cargo loco task replace_art id:123"
                .to_string(),
        }
    }
    async fn run(&self, ctx: &AppContext, vars: &task::Vars) -> Result<()> {
        println!("Running `replace_art` task");

        let art_id_str = vars.cli_arg("id")?;

        let art_id = art_id_str.parse::<i32>().map_err(|e| {
            loco_rs::errors::Error::string(&format!(
                "Invalid 'id': {e}. Must be an integer. Usage: cargo loco task replace_art id:123"
            ))
        })?;

        println!("Attempting to replace art with ID: {}", art_id);
        let updated_art = art_service::replace_art(ctx, art_id).await?;

        println!(
            "Successfully replaced art: {} - {}",
            updated_art.id, updated_art.title
        );
        Ok(())
    }
}
