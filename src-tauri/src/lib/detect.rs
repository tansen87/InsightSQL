use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn detect_separator(path: &str, skip_rows: usize) -> Option<char> {
  let file = File::open(path).expect("Failed to open file");
  let reader = BufReader::new(file);

  let mut lines_iter = reader.lines();

  // Skip the first `skip_rows` lines
  for _ in 0..skip_rows {
    if let Some(Ok(_)) = lines_iter.next() {
      // Line skipped
    } else {
      // If there are not enough lines to skip, return None or handle as you see fit
      return None;
    }
  }

  let mut line = String::new();
  let mut separators_count = HashMap::new();
  let mut max_count = 0;
  let mut separator = None;

  // read next line after skipping
  if let Some(Ok(next_line)) = lines_iter.next() {
    line.push_str(&next_line);

    // count all possible occurrences of segmentation symbols
    for c in [';', ',', '\t', '|', '^'] {
      let count = line.matches(c).count();
      if count > max_count {
        max_count = count;
        separator = Some(c);
      }
      separators_count.insert(c, count);
    }
  }

  separator
}
