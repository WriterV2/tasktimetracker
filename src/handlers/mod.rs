use axum::routing::get;
use axum::Router;
use sqlx::SqlitePool;
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;

mod booking_handlers;
mod tag_handlers;
mod tagassignment_handlers;

#[derive(Clone)]
struct ApiContext {
    pool: SqlitePool,
}

pub async fn router(pool: SqlitePool) -> Router {
    Router::new()
        .route(
            "/api/bookings",
            get(booking_handlers::get_bookings)
                .post(booking_handlers::post_booking)
                .patch(booking_handlers::patch_booking)
                .delete(booking_handlers::delete_booking),
        )
        .route(
            "/api/tags",
            get(tag_handlers::get_tags)
                .post(tag_handlers::post_tag)
                .patch(tag_handlers::patch_tag)
                .delete(tag_handlers::delete_tag),
        )
        .route(
            "/api/assignedtag",
            get(tagassignment_handlers::get_tagassignments)
                .post(tagassignment_handlers::post_tagassignments)
                .delete(tagassignment_handlers::delete_tagassignment),
        )
        .layer(ServiceBuilder::new().layer(AddExtensionLayer::new(ApiContext { pool })))
}
