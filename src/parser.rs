use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginItem {
  pub name: String,
  pub path: Option<PathBuf>,
}

#[derive(Default)]
struct ItemState {
  name: Option<String>,
  path: Option<PathBuf>,
}

pub fn parse_login_items(input: &str) -> Vec<LoginItem> {
  let mut items = Vec::new();
  let mut state = ItemState::default();

  for line in input.lines().map(str::trim_start) {
    if is_item_start(line) {
      if let Some(item) = state.finish() {
        items.push(item);
      }
      state = ItemState::default();
      continue;
    }

    if let Some(value) = parse_field(line, "Name:") {
      state.name = Some(clean_name(value));
    }

    if let Some(value) = parse_field(line, "URL:") {
      state.path = parse_url(value);
    }
  }

  if let Some(item) = state.finish() {
    items.push(item);
  }

  items
}

impl ItemState {
  fn finish(&mut self) -> Option<LoginItem> {
    if self.name.is_none() && self.path.is_none() {
      return None;
    }

    Some(LoginItem {
      name: self.name.take().unwrap_or_else(|| "未命名".to_string()),
      path: self.path.take(),
    })
  }
}

fn is_item_start(line: &str) -> bool {
  let trimmed = line.trim();
  trimmed.starts_with('#') && trimmed.ends_with(':')
}

fn parse_field<'a>(line: &'a str, label: &str) -> Option<&'a str> {
  line.strip_prefix(label).map(str::trim)
}

fn clean_name(value: &str) -> String {
  match value.trim() {
    "" | "(null)" => "未命名".to_string(),
    other => other.to_string(),
  }
}

fn parse_url(value: &str) -> Option<PathBuf> {
  let value = value.trim();
  if value.is_empty() || value == "(null)" {
    return None;
  }

  let raw_path = value.strip_prefix("file://").unwrap_or(value);
  let raw_path = raw_path.strip_prefix("localhost/").unwrap_or(raw_path);
  let decoded = percent_decode(raw_path);

  Some(PathBuf::from(decoded))
}

fn percent_decode(input: &str) -> String {
  let mut bytes = Vec::with_capacity(input.len());
  let mut iter = input.as_bytes().iter().copied();

  while let Some(byte) = iter.next() {
    if byte == b'%' {
      let hi = iter.next();
      let lo = iter.next();
      if let (Some(hi), Some(lo)) = (hi, lo) {
        if let (Some(hi), Some(lo)) = (hex_value(hi), hex_value(lo)) {
          bytes.push(hi * 16 + lo);
          continue;
        }
        bytes.push(b'%');
        bytes.push(hi);
        bytes.push(lo);
        continue;
      }
    }
    bytes.push(byte);
  }

  String::from_utf8_lossy(&bytes).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
  match byte {
    b'0'..=b'9' => Some(byte - b'0'),
    b'a'..=b'f' => Some(byte - b'a' + 10),
    b'A'..=b'F' => Some(byte - b'A' + 10),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::parse_login_items;
  use std::path::Path;

  const SAMPLE: &str = include_str!("../test/test.txt");

  #[test]
  fn parses_known_items() {
    let items = parse_login_items(SAMPLE);

    assert!(items.iter().any(|item| {
      item.name == "Battery Toolkit"
        && item
          .path
          .as_deref()
          .is_some_and(|path| path == Path::new("/Applications/Battery Toolkit.app/"))
    }));

    assert!(items.iter().any(|item| {
      item.name == "iStat Menus Helper"
        && item.path.as_deref().is_some_and(|path| {
          path == Path::new("/Users/swizzer/Library/LaunchAgents/com.bjango.istatmenus.agent.plist")
        })
    }));

    assert!(items.iter().any(|item| item.name == "天气"));
  }
}
