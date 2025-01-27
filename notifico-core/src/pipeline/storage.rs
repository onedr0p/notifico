use crate::error::EngineError;
use crate::http::admin::{ListQueryParams, PaginatedResult};
use crate::pipeline::{Event, Pipeline};
use async_trait::async_trait;
use serde::Serialize;
use std::error::Error;
use uuid::Uuid;

#[derive(Clone, Serialize)]
pub struct PipelineResult {
    pub pipeline: Pipeline,
    pub event_ids: Vec<Uuid>,
}

#[async_trait]
pub trait PipelineStorage: Send + Sync {
    async fn get_pipelines_for_event(
        &self,
        project: Uuid,
        event_name: &str,
    ) -> Result<Vec<Pipeline>, EngineError>;
    async fn list_pipelines(
        &self,
        params: ListQueryParams,
    ) -> Result<PaginatedResult<PipelineResult>, EngineError>;
    async fn get_pipeline_by_id(&self, id: Uuid) -> Result<Option<PipelineResult>, EngineError>;
    async fn create_pipeline(&self, pipeline: Pipeline) -> Result<Pipeline, EngineError>;
    async fn update_pipeline(&self, pipeline: Pipeline) -> Result<(), EngineError>;
    async fn assign_events_to_pipeline(
        &self,
        pipeline_id: Uuid,
        event_id: Vec<Uuid>,
    ) -> Result<(), EngineError>;
    async fn delete_pipeline(&self, id: Uuid) -> Result<(), EngineError>;

    async fn list_events(
        &self,
        params: ListQueryParams,
    ) -> Result<PaginatedResult<Event>, EngineError>;

    async fn get_event_by_id(&self, id: Uuid) -> Result<Option<Event>, Box<dyn Error>>;
    async fn create_event(&self, project_id: Uuid, name: &str) -> Result<Event, Box<dyn Error>>;
    async fn update_event(&self, id: Uuid, name: &str) -> Result<Event, Box<dyn Error>>;
    async fn delete_event(&self, id: Uuid) -> Result<(), Box<dyn Error>>;
}
