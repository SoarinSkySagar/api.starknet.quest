use std::sync::Arc;

use crate::models::{QuestTaskDocument, VerifyNewQuery};
use crate::{
    models::{AppState},
    utils::{get_error, CompletedTasksTrait},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_auto_routes::route;
use futures::TryStreamExt;
use mongodb::bson::doc;
use serde_json::json;

#[route(
    get,
    "/quests/verify_twitter_rw",
    crate::endpoints::quests::verify_twitter_rw
)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyNewQuery>,
) -> impl IntoResponse {
    let quest_id = query.quest_id;
    let task_id = query.task_id;
    let pipeline = vec![doc! {
        "$match": doc! {
            "quest_id": quest_id,
            "id":task_id,
            "task_type": "twitter_rw"
        }
    }];

    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");
    match tasks_collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            while let Some(_result) = cursor.try_next().await.unwrap() {
                match state.upsert_completed_task(query.addr, task_id).await {
                    Ok(_) => return (StatusCode::OK, Json(json!({"res": true}))).into_response(),
                    Err(e) => get_error(format!("{}", e)),
                };
            }
            get_error("Error querying task".to_string())
        }
        Err(e) => get_error(e.to_string()),
    }
}
