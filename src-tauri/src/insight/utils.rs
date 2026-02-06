use std::fs::File;
use std::future::Future;
use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow};
use memmap2::Mmap;
use tauri::{AppHandle, Emitter};

use crate::io::csv::options::CsvOptions;

pub const EXCEL_MAX_ROW: usize = 104_8575; // no headers
pub const RDR_BUFFER_SIZE: usize = 1 * 1024 * 1024;
pub const WTR_BUFFER_SIZE: usize = 1 * 1024 * 1024;
pub const DEFAULT_BATCH_SIZE: usize = 50_000;

#[inline]
pub fn num_cpus() -> usize {
  num_cpus::get()
}

pub const fn chunk_size(nitems: usize, njobs: usize) -> usize {
  if nitems < njobs {
    nitems
  } else {
    nitems / njobs
  }
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

pub fn njobs(jobs: Option<usize>) -> usize {
  let max_cpus = num_cpus();
  jobs.map_or(
    max_cpus,
    |j| {
      if j == 0 || j > max_cpus { max_cpus } else { j }
    },
  )
}

#[inline]
pub fn batch_size<P: AsRef<Path> + Send + Sync>(opts: &CsvOptions<P>, njobs: usize) -> usize {
  let num_rows = match opts.indexed() {
    Ok(Some(idx)) => idx.count() as usize,
    _ => {
      return DEFAULT_BATCH_SIZE;
    }
  };

  if num_rows.is_multiple_of(njobs) {
    num_rows / njobs
  } else {
    (num_rows / njobs) + 1
  }
}

pub fn parse_usize(s: &str, name: &str) -> Result<usize, String> {
  s.parse::<usize>()
    .map_err(|e| format!("parse '{name}' error: {e}"))
}

pub struct MmapOffsets {
  mmap: Mmap,
  num_offsets: usize, // 不包含最后的 total_len
}

impl MmapOffsets {
  pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
    let file = File::open(&path)?;
    let metadata = file.metadata()?;
    let file_len = metadata.len();

    if file_len < 8 || file_len % 8 != 0 {
      return Err(anyhow::anyhow!("Invalid index file size"));
    }

    let mmap = unsafe { Mmap::map(&file)? };

    // 最后 8 字节是 total_len
    let total_len_bytes = &mmap[file_len as usize - 8..];
    let total_records = u64::from_be_bytes(total_len_bytes.try_into().unwrap());

    let num_offsets = total_records as usize;
    let expected_file_size = (num_offsets + 1) * 8;

    if file_len != expected_file_size as u64 {
      return Err(anyhow::anyhow!(
        "Index file size mismatch: got {}, expected {}",
        file_len,
        expected_file_size
      ));
    }

    Ok(Self { mmap, num_offsets })
  }

  #[inline]
  pub fn get(&self, i: usize) -> u64 {
    if i >= self.num_offsets {
      panic!(
        "Offset index {} out of bounds (num_offsets = {})",
        i, self.num_offsets
      );
    }
    let start = i * 8;
    u64::from_be_bytes(self.mmap[start..start + 8].try_into().unwrap())
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.num_offsets
  }
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
