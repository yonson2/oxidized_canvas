use std::path::Path;

use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    boot::{create_app, BootResult, StartMode},
    controller::AppRoutes,
    // db::{self, truncate_table},
    environment::Environment,
    task::Tasks,
    // AppWorker,
    worker::Processor,
    Result,
};
use migration::Migrator;
use sea_orm::DatabaseConnection;

use crate::{controllers, initializers, tasks};

pub struct App;
#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(mode: StartMode, environment: &Environment) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment).await
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![Box::new(
            initializers::view_engine::ViewEngineInitializer,
        )])
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .add_route(controllers::arts::index())
            .add_route(controllers::arts::routes())
    }

    fn connect_workers<'a>(_p: &'a mut Processor, _ctx: &'a AppContext) {
        // p.register(DownloadWorker::build(ctx));
    }

    fn register_tasks(tasks: &mut Tasks) {
        tasks.register(tasks::clean_titles::CleanTitles);
        tasks.register(tasks::import_arts::ImportArts);
        tasks.register(tasks::create_art::CreateArt);
        tasks.register(tasks::seed::SeedData);
    }

    async fn truncate(_db: &DatabaseConnection) -> Result<()> {
        // truncate_table(db, users::Entity).await?;
        Ok(())
    }

    async fn seed(_db: &DatabaseConnection, _base: &Path) -> Result<()> {
        // db::seed::<users::ActiveModel>(db, &base.join("users.yaml").display().to_string()).await?;
        Ok(())
    }
}
