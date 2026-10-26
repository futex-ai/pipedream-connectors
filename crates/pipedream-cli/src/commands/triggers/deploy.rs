//! The trigger deployment command: `triggers deploy`.

use std::sync::Arc;

use pipedream_connect::resources::triggers::TriggersApi;
use pipedream_connect_types::{
    ComponentKey, DeployTriggerRequest, ExternalUserId, WorkflowEndpoint,
};

use crate::argument::Argument;
use crate::commands::triggers::args::DeployTriggerArgs;
use crate::document::{Output, Secrets};
use crate::error::{Result, called, checked};
use crate::inputs::{InputSource, Inputs, read_props};
use crate::target::Target;

/// Validates the deployment and reads its props input last. Sends no request.
pub(super) fn prepare(
    args: DeployTriggerArgs,
    inputs: &dyn Inputs,
) -> Result<DeployTriggerRequest> {
    let mut request = DeployTriggerRequest {
        id: checked(ComponentKey::new(args.component), Argument::Component)?,
        version: args.version,
        external_user_id: checked(
            ExternalUserId::new(args.external_user_id),
            Argument::ExternalUserId,
        )?,
        configured_props: None,
        dynamic_props_id: args.dynamic_props_id,
        workflow_id: args
            .workflow_id
            .map(|id| checked(WorkflowEndpoint::new(id), Argument::WorkflowId))
            .transpose()?,
        webhook_url: args.webhook_url,
        emit_on_deploy: args.emit_on_deploy,
    };
    request.configured_props = args
        .props
        .map(|path| read_props(inputs, &InputSource::from(path), Argument::Props))
        .transpose()?;
    Ok(request)
}

/// Deploys the trigger once. Its signing keys print only with `--show-secrets`.
pub(crate) async fn run(
    request: DeployTriggerRequest,
    api: Arc<dyn TriggersApi + Send + Sync>,
    secrets: Secrets,
) -> Result<Output> {
    let response = called(api.deploy(request).await, Target::TriggerDeploy)?;
    Output::redacted(Target::TriggerDeploy, response, secrets)
}

#[cfg(test)]
#[path = "_tests_/deploy_tests.rs"]
mod deploy_tests;
