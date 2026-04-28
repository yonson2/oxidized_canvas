use loco_rs::{task, testing::request::boot_test};
use oxidized_canvas::app::App;

use loco_rs::boot::run_task;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_can_run_clean_titles() {
    let boot = boot_test::<App>().await.unwrap();

    assert!(
        run_task::<App>(
            &boot.app_context,
            Some(&"clean_titles".to_string()),
            &task::Vars::default()
        )
        .await
        .is_ok()
    );
}
