use std::sync::{OnceLock, RwLock};

use serde::Serialize;
use socketioxide::SocketIo;
use tracing::warn;
use uuid::Uuid;

static SOCKET_IO: OnceLock<RwLock<Option<SocketIo>>> = OnceLock::new();

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressUpdate {
    pub stage: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_to: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    pub done: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub failed: bool,
}

impl ProgressUpdate {
    pub fn new(stage: &'static str, message: impl Into<String>) -> Self {
        Self {
            stage,
            message: message.into(),
            redirect_to: None,
            done: false,
            failed: false,
        }
    }

    pub fn done(stage: &'static str, message: impl Into<String>) -> Self {
        Self {
            done: true,
            ..Self::new(stage, message)
        }
    }

    pub fn failed(stage: &'static str, message: impl Into<String>) -> Self {
        Self {
            failed: true,
            ..Self::new(stage, message)
        }
    }

    #[must_use]
    pub fn with_redirect_to(mut self, redirect_to: impl Into<String>) -> Self {
        self.redirect_to = Some(redirect_to.into());
        self
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn store() -> &'static RwLock<Option<SocketIo>> {
    SOCKET_IO.get_or_init(|| RwLock::new(None))
}

fn socket_io() -> Option<SocketIo> {
    store().read().expect("socket io store poisoned").clone()
}

pub fn install(io: SocketIo) {
    *store().write().expect("socket io store poisoned") = Some(io);
}

#[must_use]
pub fn mix_room(request_id: &Uuid) -> String {
    format!("mix:{request_id}")
}

#[must_use]
pub fn art_replace_room(art_uuid: &Uuid) -> String {
    format!("art-replace:{art_uuid}")
}

#[must_use]
pub fn parse_subscription_id(value: &str) -> Option<Uuid> {
    Uuid::parse_str(value).ok()
}

pub async fn emit_mix_progress(request_id: &Uuid, update: &ProgressUpdate) {
    emit("mix-progress", mix_room(request_id), update).await;
}

pub async fn emit_art_replace_progress(art_uuid: &Uuid, update: &ProgressUpdate) {
    emit("art-replace-progress", art_replace_room(art_uuid), update).await;
}

async fn emit(event: &str, room: String, update: &ProgressUpdate) {
    let Some(io) = socket_io() else {
        return;
    };

    let room_name = room.clone();
    if let Err(err) = io.to(room).emit(event, update).await {
        warn!(room = %room_name, event, error = %err, "failed to emit realtime progress update");
    }
}
