use loco_rs::prelude::*;

pub struct CleanTitles;
#[async_trait]
impl Task for CleanTitles {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "clean_titles".to_string(),
            detail: "Task generator".to_string(),
        }
    }
    async fn run(&self, _app_context: &AppContext, _vars: &task::Vars) -> Result<()> {
        println!("Task CleanTitles generated");
        Ok(())
    }
}
