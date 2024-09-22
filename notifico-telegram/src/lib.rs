use async_trait::async_trait;
use contact::TelegramContact;
use notifico_core::credentials::Credentials;
use notifico_core::engine::{EnginePlugin, PipelineContext};
use notifico_core::error::EngineError;
use notifico_core::pipeline::SerializedStep;
use notifico_core::templater::{RenderResponse, Templater};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::sync::Arc;
use step::{CredentialSelector, TelegramStep};
use teloxide::prelude::Requester;
use teloxide::Bot;
use tracing::debug;
use uuid::Uuid;

mod contact;
mod step;

#[derive(Debug, Serialize, Deserialize)]
pub struct TelegramBotCredentials {
    token: String,
}

impl TryFrom<Value> for TelegramBotCredentials {
    type Error = EngineError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value).map_err(|_| EngineError::InvalidCredentialFormat)
    }
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

#[derive(Default, Serialize, Deserialize)]
struct TelegramContext {
    template_id: Option<Uuid>,
}

#[async_trait]
impl EnginePlugin for TelegramPlugin {
    async fn execute_step(
        &self,
        context: &mut PipelineContext,
        step: &SerializedStep,
    ) -> Result<(), EngineError> {
        let telegram_context = context
            .plugin_contexts
            .entry("telegram".into())
            .or_insert(Value::Object(Default::default()));

        debug!("Plugin context: {:?}", telegram_context);

        let mut plugin_context: TelegramContext =
            serde_json::from_value(telegram_context.clone()).unwrap();
        let telegram_step: TelegramStep = step.clone().try_into().unwrap();

        match telegram_step {
            TelegramStep::LoadTemplate { template_id } => {
                plugin_context.template_id = Some(template_id);
                context.plugin_contexts.insert(
                    "telegram".into(),
                    serde_json::to_value(plugin_context).unwrap(),
                );
            }
            TelegramStep::Send(cred_selector) => {
                let Some(template_id) = plugin_context.template_id else {
                    return Err(EngineError::TemplateNotSet);
                };
                let Some(recipient) = context.recipient.clone() else {
                    return Err(EngineError::RecipientNotSet);
                };

                let tgcred: TelegramBotCredentials = match cred_selector {
                    CredentialSelector::BotName { bot_name } => self
                        .credentials
                        .get_credential(context.project_id, "telegram_token", &bot_name)?
                        .try_into()?,
                };

                let bot = Bot::new(tgcred.token);

                let rendered_template = self
                    .templater
                    .render("telegram", template_id, context.event_context.0.clone())
                    .await?;

                let rendered_template: TelegramBody = rendered_template.try_into().unwrap();

                let contact: TelegramContact = recipient
                    .get_primary_contact("telegram")?
                    .clone()
                    .try_into()?;

                bot.send_message(contact.into_recipient(), rendered_template.body)
                    .await
                    .unwrap();
            }
        }

        Ok(())
    }

    fn step_namespace(&self) -> Cow<'static, str> {
        "telegram".into()
    }
}

#[derive(Deserialize, Clone)]
pub struct TelegramBody {
    pub body: String,
}

impl TryFrom<RenderResponse> for TelegramBody {
    type Error = ();

    fn try_from(value: RenderResponse) -> Result<Self, Self::Error> {
        serde_json::from_value(Value::from_iter(value.0)).map_err(|_| ())
    }
}
