extern crate wapc_guest as guest;
use guest::prelude::*;

extern crate chimera_kube_policy_sdk as chimera;
use chimera::request::ValidationRequest;

extern crate url;
extern crate regex;

use anyhow::anyhow;
use k8s_openapi::api::core::v1 as apicore;
use serde::{Deserialize, Serialize};
use serde_json::Result;

mod settings;
use settings::Settings;

#[no_mangle]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;



    chimera::accept_request()
}
