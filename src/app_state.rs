use std::{ops::Deref, sync::Arc};
use sqlx::PgPool;
use crate::tl::schema_manager::SchemaManager;

pub struct AppState {
    inner: Arc<InnerAppState>,
}

impl AppState {
    pub(crate) fn new(db: Arc<PgPool>, schema_manager: SchemaManager) -> Self {
        Self { inner: Arc::new(InnerAppState { db, schema_manager: Box::new(schema_manager) }) }
    }
}

pub struct InnerAppState {
    pub db: Arc<PgPool>,
    pub schema_manager: Box<SchemaManager>,
}

impl Deref for AppState {
    type Target = InnerAppState;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner)
        }
    }
}