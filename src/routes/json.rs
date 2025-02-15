use std::num::NonZeroU32;

use crate::db::write;
use crate::env::BASE_PATH;
use crate::errors::{Error, JsonErrorResponse};
use crate::id::Id;
use crate::AppState;
use axum::extract::State;
use axum::Json;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub text: String,
    pub extension: Option<String>,
    pub expires: Option<NonZeroU32>,
    pub burn_after_reading: Option<bool>,
    pub password: Option<String>,
    pub title: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct RedirectResponse {
    pub path: String,
}

impl From<Entry> for write::Entry {
    fn from(entry: Entry) -> Self {
        Self {
            text: entry.text,
            extension: entry.extension,
            expires: entry.expires,
            burn_after_reading: entry.burn_after_reading,
            uid: None,
            password: entry.password,
            title: entry.title,
        }
    }
}

pub async fn insert(
    state: State<AppState>,
    Json(entry): Json<Entry>,
) -> Result<Json<RedirectResponse>, JsonErrorResponse> {
    let id: Id = tokio::task::spawn_blocking(|| {
        let mut rng = rand::thread_rng();
        rng.gen::<u32>()
    })
    .await
    .map_err(Error::from)?
    .into();

    let mut entry: write::Entry = entry.into();

    if let Some(max_exp) = state.max_expiration {
        entry.expires = entry
            .expires
            .map_or_else(|| Some(max_exp), |value| Some(value.min(max_exp)));
    }

    let url = id.to_url_path(&entry);
    let path = BASE_PATH.join(&url);
    state.db.insert(id, entry).await?;

    Ok(Json::from(RedirectResponse { path }))
}
