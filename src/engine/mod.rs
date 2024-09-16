use async_trait::async_trait;
use notifico_core::engine::{EngineError, EnginePlugin, PipelineContext};
use notifico_core::pipeline::{Pipeline, SerializedStep};
use notifico_core::recipient::Recipient;
use notifico_core::templater::TemplaterError;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::any::Any;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use tracing::instrument;

#[derive(Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub pipelines: Vec<Pipeline>,
}

pub struct Engine {
    plugins: HashMap<Cow<'static, str>, Arc<dyn EnginePlugin + Send + Sync>>,
}

impl Debug for Engine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let plugins = self.plugins.keys().cloned().collect::<Vec<_>>();
        f.write_fmt(format_args!("Engine {{ plugins: [{:?}] }}", plugins))
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            plugins: Default::default(),
        }
    }

    pub fn add_plugin(&mut self, plugin: impl EnginePlugin + 'static) {
        self.plugins.insert(plugin.step_type(), Arc::new(plugin));
    }

    #[instrument]
    pub(crate) async fn execute_step(
        &mut self,
        context: &mut PipelineContext,
        step: &SerializedStep,
    ) -> Result<(), EngineError> {
        let step_type = step.get_type();

        for (plugin_type, plugin) in self.plugins.iter() {
            if step_type.starts_with(&**plugin_type) {
                plugin.execute_step(context, step).await?;
                return Ok(());
            }
        }
        Err(EngineError::PluginNotFound(step.clone()))
    }
}