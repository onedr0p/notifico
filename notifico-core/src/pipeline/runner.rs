use crate::engine::{Engine, EventContext, PipelineContext, StepOutput};
use crate::error::EngineError;
use crate::pipeline::storage::PipelineStorage;
use crate::pipeline::Pipeline;
use crate::recipient::Recipient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ProcessEventRequest {
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    #[serde(default = "Uuid::nil")]
    pub project_id: Uuid,
    pub event: String,
    pub recipient: Option<RecipientSelector>,
    pub context: EventContext,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case", untagged)]
pub enum RecipientSelector {
    Recipient(Recipient),
}

pub struct PipelineRunner {
    pipeline_storage: Arc<dyn PipelineStorage>,
    engine: Engine,
}

impl PipelineRunner {
    pub fn new(pipeline_storage: Arc<dyn PipelineStorage>, engine: Engine) -> Self {
        Self {
            pipeline_storage,
            engine,
        }
    }

    pub async fn process_eventrequest(&self, msg: ProcessEventRequest) {
        self.process_event(
            msg.id,
            msg.project_id,
            &msg.event,
            msg.context,
            msg.recipient,
        )
        .await
        .unwrap()
    }

    /// Processes an event by executing the associated pipelines.
    ///
    /// # Parameters
    ///
    /// * `project_id` - The unique identifier of the project associated with the event.
    /// * `trigger_event` - The name of the event that triggered the pipeline execution.
    /// * `event_context` - The contextual information related to the event.
    /// * `recipient_sel` - An optional selector for the recipient of the event.
    pub async fn process_event(
        &self,
        event_id: Uuid,
        project_id: Uuid,
        event_name: &str,
        event_context: EventContext,
        recipient_sel: Option<RecipientSelector>,
    ) -> Result<(), EngineError> {
        // Fetch the pipelines associated with the project and event
        let pipelines = self
            .pipeline_storage
            .get_pipelines_for_event(project_id, event_name)
            .await?;

        // Determine the recipient based on the recipient selector
        let recipient = match recipient_sel {
            None => None,
            Some(RecipientSelector::Recipient(recipient)) => Some(recipient),
        };

        // Execute each pipeline in a separate task in parallel
        let mut join_handles = JoinSet::new();
        for pipeline in pipelines {
            let engine = self.engine.clone();
            let recipient = recipient.clone();
            let event_context = event_context.clone();
            let event_name = event_name.to_string();

            let channel = pipeline.channel.clone();

            let contact = recipient
                .clone()
                .map(|r| r.get_primary_contact(&channel))
                .unwrap_or_default();

            join_handles.spawn(async move {
                let context = PipelineContext {
                    step_number: 0,

                    project_id,
                    recipient,
                    event_name,
                    event_context,
                    plugin_contexts: Default::default(),
                    messages: Default::default(),
                    channel,
                    contact,
                    notification_id: Uuid::now_v7(),
                    event_id,
                };

                // Execute each step in the pipeline
                Self::execute_pipeline(engine, pipeline, context).await;
            });
        }

        // Wait for all pipelines to complete
        join_handles.join_all().await;
        Ok(())
    }

    pub async fn execute_pipeline(
        engine: Engine,
        pipeline: Pipeline,
        mut context: PipelineContext,
    ) {
        for (step_number, step) in pipeline.steps.iter().enumerate() {
            if step_number < context.step_number {
                continue;
            }

            let result = engine.execute_step(&mut context, step).await;
            match result {
                Ok(StepOutput::Continue) => context.step_number += 1,
                Ok(StepOutput::Interrupt) => break,
                Err(err) => {
                    error!("Error executing step: {:?}", err);
                    break;
                }
            }
        }
    }
}
