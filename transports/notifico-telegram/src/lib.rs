use crate::step::STEPS;
use async_trait::async_trait;
use contact::TelegramContact;
use notifico_core::recorder::Recorder;
use notifico_core::step::SerializedStep;
use notifico_core::{
    credentials::{CredentialStorage, TypedCredential},
    engine::PipelineContext,
    engine::{EnginePlugin, StepOutput},
    error::EngineError,
    templater::RenderedTemplate,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::sync::Arc;
use step::Step;
use teloxide::prelude::Requester;
use teloxide::Bot;

mod contact;
mod step;

#[derive(Debug, Serialize, Deserialize)]
struct TelegramBotCredentials {
    token: String,
}

impl TypedCredential for TelegramBotCredentials {
    const CREDENTIAL_TYPE: &'static str = "telegram_bot";
}

pub struct TelegramPlugin {
    credentials: Arc<dyn CredentialStorage>,
    recorder: Arc<dyn Recorder>,
}

impl TelegramPlugin {
    pub fn new(credentials: Arc<dyn CredentialStorage>, recorder: Arc<dyn Recorder>) -> Self {
        Self {
            credentials,
            recorder,
        }
    }
}

#[async_trait]
impl EnginePlugin for TelegramPlugin {
    async fn execute_step(
        &self,
        context: &mut PipelineContext,
        step: &SerializedStep,
    ) -> Result<StepOutput, EngineError> {
        let step: Step = step.clone().convert_step()?;

        match step {
            Step::Send { credential } => {
                let credential: TelegramBotCredentials = self
                    .credentials
                    .get_typed_credential(context.project_id, &credential)
                    .await?;
                let bot = Bot::new(credential.token);
                let contact: TelegramContact = context.get_contact()?;

                for message in context.messages.iter().cloned() {
                    let content: TelegramContent = message.content.try_into().unwrap();

                    // Send
                    let result = bot
                        .send_message(contact.clone().into_recipient(), content.body)
                        .await;

                    match result {
                        Ok(_) => self.recorder.record_message_sent(
                            context.event_id,
                            context.notification_id,
                            message.id,
                        ),
                        Err(e) => self.recorder.record_message_failed(
                            context.event_id,
                            context.notification_id,
                            message.id,
                            &e.to_string(),
                        ),
                    }
                }
            }
        }

        Ok(StepOutput::Continue)
    }

    fn steps(&self) -> Vec<Cow<'static, str>> {
        STEPS.iter().map(|&s| s.into()).collect()
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct TelegramContent {
    pub body: String,
}

impl TryFrom<RenderedTemplate> for TelegramContent {
    type Error = ();

    fn try_from(value: RenderedTemplate) -> Result<Self, Self::Error> {
        serde_json::from_value(Value::from_iter(value.0)).map_err(|_| ())
    }
}
