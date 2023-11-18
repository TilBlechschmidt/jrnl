use crate::{auth::AuthenticatedUser, ENV_STORAGE_LOCATION};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};
use tokio::{fs, io};

const STORAGE_EXTENSION: &'static str = "md";
const TRUNCATE_LEN: usize = 1024;

// Unix timestamp that (almost) uniquely identifies a document
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocumentIdentifier(u64);

#[derive(Serialize, Deserialize)]
pub struct Document {
    pub identifier: DocumentIdentifier,
    pub contents: String,
}

pub struct UserStorage {
    path: PathBuf,
}

impl UserStorage {
    pub fn new(user_id: impl AsRef<str>) -> Self {
        let root: PathBuf = env::var(ENV_STORAGE_LOCATION)
            .expect(&format!("env var {ENV_STORAGE_LOCATION} not set"))
            .into();

        Self {
            path: root.join(user_id.as_ref()),
        }
    }

    pub async fn read(
        &self,
        identifier: DocumentIdentifier,
        truncate: bool,
    ) -> io::Result<Document> {
        let mut contents = fs::read_to_string(self.doc_path(identifier)).await?;

        if truncate {
            contents.truncate(TRUNCATE_LEN);
        }

        Ok(Document {
            identifier,
            contents,
        })
    }

    pub async fn write(&self, document: Document) -> io::Result<()> {
        let doc_path = self.doc_path(document.identifier);

        fs::create_dir_all(&self.path).await?;
        fs::write(doc_path, document.contents).await
    }

    pub async fn entries(&self) -> io::Result<Vec<Document>> {
        fs::create_dir_all(&self.path).await?;

        let mut entries = fs::read_dir(&self.path).await?;
        let mut documents = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            if let Some((name, extension)) = entry
                .file_name()
                .to_str()
                .map(|n| n.rsplit_once('.'))
                .flatten()
            {
                if extension != STORAGE_EXTENSION {
                    continue;
                }

                if let Ok(identifier) = name.parse().map(DocumentIdentifier) {
                    let document = self.read(identifier, true).await?;
                    documents.push(document);
                }
            }
        }

        documents.sort_unstable_by_key(|d| d.identifier);
        documents.reverse();

        Ok(documents)
    }

    fn doc_path(&self, document: DocumentIdentifier) -> PathBuf {
        self.path
            .join(format!("{}.{STORAGE_EXTENSION}", document.0))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for UserStorage
where
    S: Send + Sync,
{
    type Rejection = <AuthenticatedUser as FromRequestParts<S>>::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;
        Ok(UserStorage::new(user.subject))
    }
}
