mod parser;
mod rui;

use crate::parser::{LoginItem, parse_login_items};
use std::io;
use std::process::Command;

fn main() -> color_eyre::Result<()> {
  color_eyre::install()?;
  ensure_sudo()?;
  let items = load_items()?;
  ratatui::run(|terminal| rui::run_app(terminal, items, delete_item))?;
  Ok(())
}
// TODO: 提示用户输入密码而不是直接调用 sudo -v
fn ensure_sudo() -> io::Result<()> {
  let status = Command::new("sudo").arg("-v").status()?;
  if status.success() {
    Ok(())
  } else {
    Err(io::Error::new(io::ErrorKind::Other, "sudo 失败"))
  }
}

fn load_items() -> io::Result<Vec<LoginItem>> {
  let output = Command::new("sfltool").arg("dumpbtm").output()?;
  if !output.status.success() {
    return Err(io::Error::new(
      io::ErrorKind::Other,
      format!("sfltool dumpbtm 失败: {}", output.status),
    ));
  }

  Ok(parse_login_items(&String::from_utf8_lossy(&output.stdout)))
}

fn delete_item(item: &LoginItem) -> io::Result<()> {
  let Some(path) = item.path.as_ref() else {
    return Err(io::Error::new(io::ErrorKind::Other, "无可删除路径"));
  };

  if !path.is_absolute() {
    return Err(io::Error::new(
      io::ErrorKind::Other,
      "路径不是绝对路径，请手动查找删除",
    ));
  }

  let metadata = std::fs::metadata(path)?;
  if metadata.is_dir() {
    std::fs::remove_dir_all(path)
  } else {
    std::fs::remove_file(path)
  }
}
