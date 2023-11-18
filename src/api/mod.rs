use crate::storage::{Document, DocumentIdentifier, UserStorage};
use axum::{
    body::Body,
    extract::Path,
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use tokio::io::ErrorKind;
use tracing::warn;

pub fn router() -> Router<(), Body> {
    Router::new()
        .route("/document", get(entries))
        .route("/document/:identifier", get(read))
        .route("/document/:identifier", put(write))
}

async fn entries(storage: UserStorage) -> Result<Json<Vec<Document>>, StatusCode> {
    let documents = storage.entries().await.map_err(|e| {
        warn!("Failed to list documents: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(documents))
}

async fn read(
    Path(identifier): Path<DocumentIdentifier>,
    storage: UserStorage,
) -> Result<String, StatusCode> {
    let document = storage
        .read(identifier, false)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => StatusCode::NOT_FOUND,
            _ => {
                warn!("Failed to read document: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(document.contents)
}

async fn write(
    Path(identifier): Path<DocumentIdentifier>,
    storage: UserStorage,
    contents: String,
) -> StatusCode {
    match storage
        .write(Document {
            identifier,
            contents,
        })
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(err) => {
            warn!("Failed to write document: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
