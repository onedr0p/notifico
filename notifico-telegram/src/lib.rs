use crate::context::{Message, TG_BODY};
use crate::step::STEPS;
use async_trait::async_trait;
use contact::TelegramContact;
use notifico_core::credentials::{get_typed_credential, Credentials, TypedCredential};
use notifico_core::engine::plugin::{EnginePlugin, StepOutput};
use notifico_core::engine::PipelineContext;
use notifico_core::error::EngineError;
use notifico_core::pipeline::SerializedStep;
use notifico_core::templater::{RenderResponse, Templater};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::sync::Arc;
use step::Step;
use teloxide::prelude::Requester;
use teloxide::Bot;

const CHANNEL_NAME: &'static str = "telegram";

mod contact;
mod context;
mod step;

#[derive(Debug, Serialize, Deserialize)]
pub struct TelegramBotCredentials {
    token: String,
}

impl TypedCredential for TelegramBotCredentials {
    const CREDENTIAL_TYPE: &'static str = "telegram_token";
}

pub struct TelegramPlugin {
    templater: Arc<dyn Templater>,
    credentials: Arc<dyn Credentials>,
}

impl TelegramPlugin {
    pub fn new(templater: Arc<dyn Templater>, credentials: Arc<dyn Credentials>) -> Self {
        Self {
            templater,
            credentials,
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
            Step::LoadTemplate { template_id } => {
                if context.recipient.is_none() {
                    return Err(EngineError::RecipientNotSet);
                };

                let rendered_template = self
                    .templater
                    .render(CHANNEL_NAME, template_id, context.event_context.0.clone())
                    .await?;
                let rendered_template: TelegramContent = rendered_template.try_into().unwrap();

                context.plugin_contexts.insert(
                    TG_BODY.into(),
                    serde_json::to_value(rendered_template.body).unwrap(),
                );
            }
            Step::Send { credential } => {
                let Some(recipient) = context.recipient.clone() else {
                    return Err(EngineError::RecipientNotSet);
                };

                let message: Message =
                    serde_json::from_value(context.plugin_contexts.clone().into()).unwrap();

                let contact: TelegramContact = recipient.get_primary_contact()?;

                // Send
                let credential: TelegramBotCredentials = get_typed_credential(
                    self.credentials.as_ref(),
                    context.project_id,
                    &credential,
                )?;

                let bot = Bot::new(credential.token);
                bot.send_message(contact.into_recipient(), message.body)
                    .await
                    .unwrap();
            }
        }

        Ok(StepOutput::None)
    }

    fn steps(&self) -> Vec<Cow<'static, str>> {
        STEPS.iter().map(|&s| s.into()).collect()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TelegramContent {
    pub body: String,
}

impl TryFrom<RenderResponse> for TelegramContent {
    type Error = ();

    fn try_from(value: RenderResponse) -> Result<Self, Self::Error> {
        serde_json::from_value(Value::from_iter(value.0)).map_err(|_| ())
    }
}
