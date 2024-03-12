mod signal;

pub use signal::{AstroRunSignal, Signal};

use crate::{Error, JobId, Result};
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct SignalManager {
  pub signals: Arc<Mutex<HashMap<JobId, AstroRunSignal>>>,
}

impl SignalManager {
  pub fn new() -> Self {
    SignalManager {
      signals: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub fn register_signal(&self, job_id: JobId, signal: AstroRunSignal) {
    self.signals.lock().insert(job_id, signal);
  }

  pub fn unregister_signal(&self, job_id: &JobId) {
    self.signals.lock().remove(&job_id);
  }

  pub fn get_signal(&self, job_id: &JobId) -> Option<AstroRunSignal> {
    self.signals.lock().get(job_id).cloned()
  }

  pub fn cancel_job(&self, job_id: &JobId) -> Result<()> {
    let signal = self
      .get_signal(job_id)
      .ok_or_else(|| Error::error(format!("Job {} not found", job_id.to_string())))?;

    signal.cancel()?;

    Ok(())
  }
}
