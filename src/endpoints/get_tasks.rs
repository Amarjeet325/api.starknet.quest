use crate::models::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use futures::stream::StreamExt;
use mongodb::bson::{doc, from_document, Document};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTask {
    id: u32,
    quest_id: u32,
    name: String,
    href: String,
    cta: String,
    verify_endpoint: String,
    desc: String,
    completed: bool,
}

#[derive(Serialize)]
pub struct QueryError {
    error: String,
}

#[derive(Deserialize)]
pub struct GetTasksQuery {
    quest_id: u32,
    addr: String,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetTasksQuery>,
) -> impl IntoResponse {
    let pipeline = vec![
        doc! { "$match": { "quest_id": query.quest_id } },
        doc! {
            "$lookup": {
                "from": "completed_tasks",
                "let": { "task_id": "$id" },
                "pipeline": [
                    {
                        "$match": {
                            "$expr": { "$eq": [ "$task_id", "$$task_id" ] },
                            "address": query.addr,
                        },
                    },
                ],
                "as": "completed",
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "id": 1,
                "quest_id": 1,
                "name": 1,
                "href": 1,
                "cta": 1,
                "verify_endpoint": 1,
                "desc": 1,
                "completed": { "$gt": [ { "$size": "$completed" }, 0 ] },
            }
        },
    ];
    let tasks_collection = state.db.collection::<Document>("tasks");
    match tasks_collection.aggregate(pipeline, None).await {
        Ok(mut cursor) => {
            let mut tasks: Vec<UserTask> = Vec::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        if let Ok(task) = from_document::<UserTask>(document) {
                            tasks.push(task);
                        }
                    }
                    _ => continue,
                }
            }
            if tasks.is_empty() {
                let error = QueryError {
                    error: String::from("No tasks found for this quest_id"),
                };
                (StatusCode::OK, Json(error)).into_response()
            } else {
                tasks.sort_by(|a, b| a.id.cmp(&b.id));
                (StatusCode::OK, Json(tasks)).into_response()
            }
        }
        Err(_) => {
            let error = QueryError {
                error: String::from("Error querying tasks"),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}