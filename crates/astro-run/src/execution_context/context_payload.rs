use serde::{Deserialize, Serialize};
use std::any::Any;

#[typetag::serde]
pub trait ContextPayloadExt: Any + Send + Sync {
  fn as_any(&self) -> &dyn Any;
}

#[derive(Serialize, Deserialize)]
pub struct ContextPayload(Box<dyn ContextPayloadExt>);

impl ContextPayload {
  pub fn new<P>(payload: P) -> Self
  where
    P: ContextPayloadExt,
  {
    ContextPayload(Box::new(payload))
  }

  pub fn payload<P>(&self) -> Option<&P>
  where
    P: ContextPayloadExt + 'static,
  {
    self.0.as_ref().as_any().downcast_ref::<P>()
  }
}

impl std::fmt::Debug for ContextPayload {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // Implement a empty payload for debug
    f.debug_struct("ContextPayload").finish()
  }
}

impl Clone for ContextPayload {
  fn clone(&self) -> Self {
    let payload_string = serde_json::to_string(&self.0).expect("Failed to serialize payload");

    let payload: Box<dyn ContextPayloadExt> =
      serde_json::from_str(&payload_string).expect("Failed to deserialize payload");

    ContextPayload(payload)
  }
}
