use std::future::Future;
use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow};
use tauri::{AppHandle, Emitter};

#[inline]
pub fn num_cpus() -> usize {
  num_cpus::get()
}

pub fn num_of_chunks(nitems: usize, chunk_size: usize) -> usize {
  if chunk_size == 0 {
    return nitems;
  }
  let mut n = nitems / chunk_size;
  if nitems % chunk_size != 0 {
    n += 1;
  }
  n
}

pub trait EventEmitter {
  fn emit_total_rows(&self, count: usize) -> impl Future<Output = Result<()>> + Send;
  fn emit_update_rows(&self, count: usize) -> impl Future<Output = Result<()>> + Send;
  fn emit_total_msg(&self, msg: &str) -> impl Future<Output = Result<()>> + Send;
  fn emit_update_msg(&self, msg: &str) -> impl Future<Output = Result<()>> + Send;
  fn emit_info(&self, info: &str) -> impl Future<Output = Result<()>> + Send;
  fn emit_err(&self, err: &str) -> impl Future<Output = Result<()>> + Send;
  fn emit_success(&self, success: &str) -> impl Future<Output = Result<()>> + Send;
}

impl EventEmitter for AppHandle {
  async fn emit_total_rows(&self, count: usize) -> Result<()> {
    self
      .emit("total-rows", count)
      .map_err(|e| anyhow!("emit total rows failed: {e}"))
  }

  async fn emit_update_rows(&self, count: usize) -> Result<()> {
    self
      .emit("update-rows", count)
      .map_err(|e| anyhow!("emit update rows failed: {e}"))
  }

  async fn emit_total_msg(&self, msg: &str) -> Result<()> {
    self
      .emit("total-msg", msg)
      .map_err(|e| anyhow!("emit total msg failed: {e}"))
  }

  async fn emit_update_msg(&self, msg: &str) -> Result<()> {
    self
      .emit("update-msg", msg)
      .map_err(|e| anyhow!("emit update msg failed: {e}"))
  }

  async fn emit_info(&self, info: &str) -> Result<()> {
    self
      .emit("info", info)
      .map_err(|e| anyhow!("emit info failed: {e}"))
  }

  async fn emit_err(&self, err: &str) -> Result<()> {
    self
      .emit("err", err)
      .map_err(|e| anyhow!("emit err failed: {e}"))
  }

  async fn emit_success(&self, success: &str) -> Result<()> {
    self
      .emit("success", success)
      .map_err(|e| anyhow!("emit success failed: {e}"))
  }
}

#[derive(Default)]
pub struct MockEmitter {
  pub total_rows: Arc<Mutex<Vec<usize>>>,
  pub update_rows: Arc<Mutex<Vec<usize>>>,
  pub total_msg: Arc<Mutex<String>>,
  pub update_msg: Arc<Mutex<String>>,
  pub info: Arc<Mutex<String>>,
  pub err: Arc<Mutex<String>>,
  pub success: Arc<Mutex<String>>,
}

impl EventEmitter for MockEmitter {
  async fn emit_total_rows(&self, count: usize) -> Result<()> {
    self
      .total_rows
      .lock()
      .map_err(|poison| anyhow!("total rows lock poisoned: {poison}"))?
      .push(count);
    Ok(())
  }

  async fn emit_update_rows(&self, count: usize) -> Result<()> {
    self
      .update_rows
      .lock()
      .map_err(|poison| anyhow!("update rows lock poisoned: {poison}"))?
      .push(count);
    Ok(())
  }

  async fn emit_total_msg(&self, msg: &str) -> Result<()> {
    self
      .total_msg
      .lock()
      .map_err(|poison| anyhow!("total msg lock poisoned: {poison}"))?
      .push_str(msg);
    Ok(())
  }

  async fn emit_update_msg(&self, msg: &str) -> Result<()> {
    self
      .update_msg
      .lock()
      .map_err(|poison| anyhow!("update msg lock poisoned: {poison}"))?
      .push_str(msg);
    Ok(())
  }

  async fn emit_info(&self, info: &str) -> Result<()> {
    self
      .info
      .lock()
      .map_err(|poison| anyhow!("info lock poisoned: {poison}"))?
      .push_str(info);
    Ok(())
  }

  async fn emit_err(&self, err: &str) -> Result<()> {
    self
      .err
      .lock()
      .map_err(|poison| anyhow!("err lock poisoned: {poison}"))?
      .push_str(err);
    Ok(())
  }

  async fn emit_success(&self, success: &str) -> Result<()> {
    self
      .success
      .lock()
      .map_err(|poison| anyhow!("success lock poisoned: {poison}"))?
      .push_str(success);
    Ok(())
  }
}
