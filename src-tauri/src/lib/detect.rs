use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn detect_separator(file_path: &str) -> Option<char> {
  let file = File::open(file_path).expect("Failed to open file");
  let mut reader = BufReader::new(file);

  let mut line = String::new();
  let mut separators_count = HashMap::new();
  let mut max_count = 0;
  let mut separator = None;

  // read first line
  if let Ok(bytes_read) = reader.read_line(&mut line) {
    if bytes_read > 0 {
      // count all possible occurrences of segmentation symbols
      for c in [';', ',', '\t', '|'] {
        let count = line.matches(c).count();
        if count > max_count {
          max_count = count;
          separator = Some(c);
        }
        separators_count.insert(c, count);
      }
    }
  }

  separator
}
