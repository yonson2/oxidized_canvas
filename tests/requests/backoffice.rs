use std::net::SocketAddr;

use axum_test::{TestServer, TestServerConfig};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use loco_rs::testing;
use oxidized_canvas::{
    app::App,
    models::{
        arts::{self, ArtParams},
        mixarts::{self, MixArtParams},
        mixes::{self, MixParams},
    },
    services::backoffice_auth,
};
use sea_orm::EntityTrait;
use serde::Serialize;
use serial_test::serial;

#[derive(Serialize)]
struct LoginBody<'a> {
    password: &'a str,
}

#[derive(Serialize)]
struct EditArtBody<'a> {
    title: &'a str,
    prompt: &'a str,
    model: &'a str,
}

#[tokio::test]
#[serial]
async fn redirects_unauthenticated_backoffice_requests_to_login() {
    let (_ctx, server) = boot_server().await;

    let response = server.get("/backoffice").await;
    let body = response.text();

    assert_eq!(response.status_code(), 303, "{body}");
    response.assert_header("location", "/backoffice/login");
}

#[tokio::test]
#[serial]
async fn can_render_backoffice_login_page_directly() {
    let (_ctx, server) = boot_server().await;

    let response = server.get("/backoffice/login").await;
    let body = response.text();

    assert_eq!(response.status_code(), 200, "{body}");
    assert!(body.contains("Unlock Backoffice"), "{body}");
}

#[tokio::test]
#[serial]
async fn can_log_in_and_edit_art_metadata() {
    let (ctx, server) = boot_server().await;
    let art = arts::Model::create(
        &ctx.db,
        &ArtParams {
            image: STANDARD.encode("fake-image-bytes"),
            prompt: "Original prompt".to_string(),
            title: "Original title".to_string(),
            model: Some("seed-model".to_string()),
        },
    )
    .await
    .unwrap();

    let login = server
        .post("/backoffice/login")
        .form(&LoginBody {
            password: "change_me",
        })
        .await;

    assert_eq!(login.status_code(), 303);
    login.assert_header("location", "/backoffice");

    let details = server.get(&format!("/backoffice/arts/{}", art.id)).await;
    let body = details.text();
    assert_eq!(details.status_code(), 200, "{body}");
    assert!(body.contains("Original title"), "{body}");

    let save = server
        .post(&format!("/backoffice/arts/{}", art.id))
        .form(&EditArtBody {
            title: "Retitled in backoffice",
            prompt: "Updated prompt from the backoffice edit form",
            model: "manual-override",
        })
        .await;

    assert_eq!(save.status_code(), 303);
    save.assert_header("location", format!("/backoffice/arts/{}", art.id));

    let updated = arts::Entity::find_by_id(art.id)
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.title, "Retitled in backoffice");
    assert_eq!(
        updated.prompt,
        "Updated prompt from the backoffice edit form"
    );
    assert_eq!(updated.model.as_deref(), Some("manual-override"));
}

#[tokio::test]
#[serial]
async fn can_delete_art_from_backoffice() {
    let (ctx, mut server) = boot_server().await;
    let art = arts::Model::create(
        &ctx.db,
        &ArtParams {
            image: STANDARD.encode("fake-image-bytes"),
            prompt: "Prompt to delete".to_string(),
            title: "Delete me".to_string(),
            model: Some("seed-model".to_string()),
        },
    )
    .await
    .unwrap();

    server.add_cookie(backoffice_auth::session_cookie(&ctx).unwrap());

    let delete = server
        .post(&format!("/backoffice/arts/{}/delete", art.id))
        .await;
    let body = delete.text();

    assert!(matches!(delete.status_code().as_u16(), 200 | 303), "{body}");
    if delete.status_code().as_u16() == 303 {
        delete.assert_header("location", "/backoffice/arts");
    } else {
        assert!(body.contains("Artwork library"), "{body}");
    }

    let deleted = arts::Entity::find_by_id(art.id).one(&ctx.db).await.unwrap();
    assert!(deleted.is_none());
}

#[tokio::test]
#[serial]
async fn can_view_and_delete_mix_from_backoffice() {
    let (ctx, mut server) = boot_server().await;
    let art = arts::Model::create(
        &ctx.db,
        &ArtParams {
            image: STANDARD.encode("fake-image-bytes"),
            prompt: "Prompt for source art".to_string(),
            title: "Source art".to_string(),
            model: Some("seed-model".to_string()),
        },
    )
    .await
    .unwrap();
    let mix = mixes::Model::create(
        &ctx.db,
        &MixParams {
            image: STANDARD.encode("fake-mix-image-bytes"),
            prompt: "Mix prompt".to_string(),
            title: "Mix title".to_string(),
            model: "mix-model".to_string(),
        },
    )
    .await
    .unwrap();
    mixarts::Model::create(
        &ctx.db,
        &MixArtParams {
            mix_id: mix.id,
            art_ids: vec![art.id],
        },
    )
    .await
    .unwrap();

    server.add_cookie(backoffice_auth::session_cookie(&ctx).unwrap());

    let details = server.get(&format!("/backoffice/mixes/{}", mix.id)).await;
    let body = details.text();
    assert_eq!(details.status_code(), 200, "{body}");
    assert!(body.contains("Mix title"), "{body}");
    assert!(body.contains("Art #"), "{body}");

    let delete = server
        .post(&format!("/backoffice/mixes/{}/delete", mix.id))
        .await;
    let body = delete.text();

    assert!(matches!(delete.status_code().as_u16(), 200 | 303), "{body}");
    if delete.status_code().as_u16() == 303 {
        delete.assert_header("location", "/backoffice/mixes");
    } else {
        assert!(body.contains("Mix archive"), "{body}");
    }

    let deleted = mixes::Entity::find_by_id(mix.id)
        .one(&ctx.db)
        .await
        .unwrap();
    assert!(deleted.is_none());
}

async fn boot_server() -> (loco_rs::app::AppContext, TestServer) {
    let boot = testing::boot_test::<App>().await.unwrap();
    let config = TestServerConfig {
        default_content_type: Some("application/json".to_string()),
        save_cookies: true,
        ..Default::default()
    };
    let server = TestServer::new_with_config(
        boot.router
            .unwrap()
            .into_make_service_with_connect_info::<SocketAddr>(),
        config,
    )
    .unwrap();

    (boot.app_context, server)
}
