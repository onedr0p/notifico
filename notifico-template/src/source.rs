use crate::error::TemplaterError;
use crate::{entity, PreRenderedTemplate, TemplateSelector};
use async_trait::async_trait;
use notifico_core::http::admin::{ListQueryParams, PaginatedResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct TemplateItem {
    #[serde(default = "Uuid::nil")]
    pub id: Uuid,
    pub project_id: Uuid,
    pub channel: String,
    pub name: String,
    pub template: PreRenderedTemplate,
}

impl From<entity::template::Model> for TemplateItem {
    fn from(value: entity::template::Model) -> Self {
        Self {
            id: value.id,
            project_id: value.project_id,
            template: PreRenderedTemplate::from(value.clone()),
            channel: value.channel,
            name: value.name,
        }
    }
}

#[async_trait]
pub trait TemplateSource: Send + Sync + 'static {
    async fn get_template(
        &self,
        project_id: Uuid,
        channel: &str,
        template: TemplateSelector,
    ) -> Result<PreRenderedTemplate, TemplaterError>;

    async fn get_template_by_id(&self, id: Uuid) -> Result<TemplateItem, TemplaterError>;

    async fn list_templates(
        &self,
        channel: &str,
        params: ListQueryParams,
    ) -> Result<PaginatedResult<TemplateItem>, TemplaterError>;

    async fn create_template(&self, item: TemplateItem) -> Result<TemplateItem, TemplaterError>;

    async fn update_template(&self, item: TemplateItem) -> Result<TemplateItem, TemplaterError>;

    async fn delete_template(&self, id: Uuid) -> Result<(), TemplaterError>;
}
