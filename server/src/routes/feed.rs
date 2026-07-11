use axum::extract::Query;
use axum::Json;
use serde::Deserialize;

use crate::algo::cursor::{self, Cursor};
use crate::error::AppError;
use crate::fixtures;
use crate::model::FeedResponse;

const PAGE_SIZE: usize = 24;

#[derive(Deserialize)]
pub struct FeedParams {
    #[allow(dead_code)] // used from M1 when real sources land
    pub actor: Option<String>,
    pub cursor: Option<String>,
}

pub async fn feed(Query(params): Query<FeedParams>) -> Result<Json<FeedResponse>, AppError> {
    let offset = params
        .cursor
        .as_deref()
        .and_then(cursor::decode)
        .map(|c| c.offset)
        .unwrap_or(0);

    let pool = fixtures::pool();
    let items: Vec<_> = pool.iter().skip(offset).take(PAGE_SIZE).cloned().collect();
    let next_offset = offset + items.len();
    let cursor = (next_offset < pool.len()).then(|| {
        cursor::encode(&Cursor { snapshot: "fixture".into(), seed: 0, offset: next_offset })
    });

    Ok(Json(FeedResponse { items, cursor }))
}
