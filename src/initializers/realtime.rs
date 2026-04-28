use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::{
    Result,
    app::{AppContext, Initializer},
};
use socketioxide::{
    SocketIo,
    extract::{Data, SocketRef},
};
use tracing::warn;

use crate::services::realtime;

pub struct RealtimeInitializer;

#[async_trait]
impl Initializer for RealtimeInitializer {
    fn name(&self) -> String {
        "realtime".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        let (layer, io) = SocketIo::builder().build_layer();

        io.ns("/", async |socket: SocketRef| {
            socket.on(
                "subscribe-mix",
                |socket: SocketRef, Data::<String>(request_id)| async move {
                    if let Some(request_id) = realtime::parse_subscription_id(&request_id) {
                        socket.join(realtime::mix_room(&request_id));
                        socket.emit("subscription-confirmed", &"mix").ok();
                    } else {
                        warn!(subscription = %request_id, "invalid mix progress subscription");
                        socket
                            .emit("subscription-error", &"Invalid mix subscription.")
                            .ok();
                    }
                },
            );

            socket.on(
                "subscribe-art-replace",
                |socket: SocketRef, Data::<String>(art_uuid)| async move {
                    if let Some(art_uuid) = realtime::parse_subscription_id(&art_uuid) {
                        socket.join(realtime::art_replace_room(&art_uuid));
                        socket.emit("subscription-confirmed", &"art-replace").ok();
                    } else {
                        warn!(subscription = %art_uuid, "invalid art replacement subscription");
                        socket
                            .emit(
                                "subscription-error",
                                &"Invalid art replacement subscription.",
                            )
                            .ok();
                    }
                },
            );
        });

        realtime::install(io);

        Ok(router.layer(layer))
    }
}
