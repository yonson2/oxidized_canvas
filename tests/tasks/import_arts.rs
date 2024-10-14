use loco_rs::{task, testing};
use oxidized_canvas::app::App;

use loco_rs::boot::run_task;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_can_run_import_arts() {
    let boot = testing::boot_test::<App>().await.unwrap();

    assert!(run_task::<App>(
        &boot.app_context,
        Some(&"import_arts".to_string()),
        &task::Vars::default()
    )
    .await
    .is_ok());
}
