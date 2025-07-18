use std::{collections::HashMap, iter, slice};

use anyhow::{Result, anyhow};
use csv::ByteRecord;

pub struct Selection {
  indices: Vec<usize>,
}

type ByteString = Vec<u8>;
type _GetField = for<'c> fn(&mut &'c ByteRecord, &usize) -> Option<&'c [u8]>;

impl Selection {
  pub fn from_headers(headers: &ByteRecord, field_names: &[&str]) -> Result<Self> {
    let header_map: HashMap<_, _> = headers
      .iter()
      .enumerate()
      .map(|(idx, name)| (String::from_utf8_lossy(name).into_owned(), idx))
      .collect();
    let mut indices = Vec::new();
    for &field_name in field_names {
      match header_map.get(field_name) {
        Some(&index) => indices.push(index),
        None => return Err(anyhow!("Field '{}' not found in headers.", field_name).into()),
      }
    }

    Ok(Selection { indices })
  }

  pub fn get_row_key(&self, row: &ByteRecord) -> Vec<ByteString> {
    self
      .indices
      .iter()
      .filter_map(|&idx| row.get(idx).map(ByteString::from))
      .collect()
  }

  pub fn get_indices(&self) -> &Vec<usize> {
    &self.indices
  }

  pub fn first_indices(&self) -> Result<usize> {
    self
      .indices
      .get(0)
      .copied()
      .ok_or(anyhow!("The indices vector is empty."))
  }

  #[inline]
  /// Returns an iterator that yields selected fields from a CSV record.
  ///
  /// This method takes a CSV record and returns an iterator that yields only the fields
  /// specified by this Selection. The fields are returned in the order they were selected.
  ///
  /// # Arguments
  ///
  /// * `row` - The CSV record to select fields from
  ///
  /// # Returns
  ///
  /// An iterator that yields references to the selected fields as byte slices
  pub fn select<'a, 'b>(
    &'a self,
    row: &'b ByteRecord,
  ) -> iter::Scan<slice::Iter<'a, usize>, &'b ByteRecord, _GetField> {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn get_field<'c>(row: &mut &'c ByteRecord, idx: &usize) -> Option<&'c [u8]> {
      Some(&row[*idx])
    }

    let get_field: _GetField = get_field;
    self.get_indices().into_iter().scan(row, get_field)
  }
}
