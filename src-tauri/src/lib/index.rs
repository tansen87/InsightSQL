use std::io;
use std::ops;

use anyhow::Result;
use csv;
use csv_index::RandomAccessSimple;

/// Indexed composes a CSV reader with a simple random access index.
pub struct Indexed<R, I> {
  csv_rdr: csv::Reader<R>,
  idx: RandomAccessSimple<I>,
}

impl<R, I> ops::Deref for Indexed<R, I> {
  type Target = csv::Reader<R>;
  fn deref(&self) -> &csv::Reader<R> {
    &self.csv_rdr
  }
}

impl<R, I> ops::DerefMut for Indexed<R, I> {
  fn deref_mut(&mut self) -> &mut csv::Reader<R> {
    &mut self.csv_rdr
  }
}

impl<R: io::Read + io::Seek, I: io::Read + io::Seek> Indexed<R, I> {
  /// Opens an index.
  pub fn open(csv_rdr: csv::Reader<R>, idx_rdr: I) -> Result<Indexed<R, I>> {
    Ok(Indexed {
      csv_rdr: csv_rdr,
      idx: RandomAccessSimple::open(idx_rdr)?,
    })
  }

  /// Return the number of records in this index.
  pub fn count(&self) -> u64 {
    self.idx.len()
  }

  /// Seek to the starting position of record `i`.
  pub fn seek(&mut self, mut i: u64) -> Result<()> {
    if i >= self.count() {
      let msg = format!(
        "invalid record index {} (there are {} records)",
        i,
        self.count()
      );
      return Err(io::Error::new(io::ErrorKind::Other, msg).into());
    }
    if self.csv_rdr.has_headers() {
      i += 1;
    }
    let pos = self.idx.get(i)?;
    self.csv_rdr.seek(pos)?;
    Ok(())
  }
}
