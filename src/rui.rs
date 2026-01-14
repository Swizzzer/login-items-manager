use crate::parser::LoginItem;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Row, Table, TableState};
use ratatui::{DefaultTerminal, Frame};
use std::io;
use std::time::{Duration, Instant};

pub fn run_app(
  terminal: &mut DefaultTerminal,
  items: Vec<LoginItem>,
  mut delete_item: impl FnMut(&LoginItem) -> io::Result<()>,
) -> io::Result<()> {
  let mut app = App::new(items);

  loop {
    terminal.draw(|frame| app.render(frame))?;

    if event::poll(Duration::from_millis(120))? {
      if let Event::Key(key) = event::read()? {
        if app.handle_key(key, &mut delete_item)? {
          break Ok(());
        }
      }
    }
  }
}

struct App {
  items: Vec<LoginItem>,
  state: TableState,
  status: Option<StatusMessage>,
  confirm: Option<ConfirmDialog>,
}

struct StatusMessage {
  text: String,
  created_at: Instant,
}

#[derive(Clone, Copy)]
struct ConfirmDialog {
  index: usize,
  selection: ConfirmChoice,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConfirmChoice {
  Confirm,
  Cancel,
}

impl App {
  fn new(items: Vec<LoginItem>) -> Self {
    let mut state = TableState::default();
    if !items.is_empty() {
      state.select(Some(0));
    }

    Self {
      items,
      state,
      status: None,
      confirm: None,
    }
  }

  fn handle_key(
    &mut self,
    key: KeyEvent,
    delete_item: &mut impl FnMut(&LoginItem) -> io::Result<()>,
  ) -> io::Result<bool> {
    if self.confirm.is_some() {
      return Ok(self.handle_confirm(key, delete_item));
    }

    match key.code {
      KeyCode::Char('q') => return Ok(true),
      KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
      KeyCode::Down | KeyCode::Char('j') => self.select_next(),
      KeyCode::Home | KeyCode::Char('g') => self.select_first(),
      KeyCode::End | KeyCode::Char('G') => self.select_last(),
      KeyCode::Char('d') => self.prompt_delete(),
      _ => {}
    }

    Ok(false)
  }

  fn handle_confirm(
    &mut self,
    key: KeyEvent,
    delete_item: &mut impl FnMut(&LoginItem) -> io::Result<()>,
  ) -> bool {
    match key.code {
      KeyCode::Enter => {
        if self.confirm_choice() == Some(ConfirmChoice::Confirm) {
          self.apply_delete(delete_item);
        } else {
          self.cancel_delete();
          return false;
        }
        self.confirm = None;
      }
      KeyCode::Char('y') => {
        self.apply_delete(delete_item);
        self.confirm = None;
      }
      KeyCode::Char('n') | KeyCode::Esc => self.cancel_delete(),
      KeyCode::Left | KeyCode::Up => self.set_confirm_choice(ConfirmChoice::Confirm),
      KeyCode::Right | KeyCode::Down => self.set_confirm_choice(ConfirmChoice::Cancel),
      KeyCode::Tab => self.toggle_confirm_choice(),
      _ => {}
    }

    false
  }

  fn render(&mut self, frame: &mut Frame) {
    self.prune_status();
    let layout = Layout::default()
      .direction(Direction::Vertical)
      .constraints([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(2),
      ])
      .split(frame.area());

    let header = Paragraph::new(Line::from(vec![
      Span::styled(
        "Login Items",
        Style::default()
          .fg(Color::LightCyan)
          .add_modifier(Modifier::BOLD),
      ),
      Span::styled("  ✦  ", Style::default().fg(Color::LightMagenta)),
      Span::styled("sfltool dumpbtm", Style::default().fg(Color::LightGreen)),
    ]))
    .block(Block::default().borders(Borders::ALL).title("启动项管理"));
    frame.render_widget(header, layout[0]);

    let rows = self.items.iter().map(|item| {
      let path = item
        .path
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "—".to_string());
      Row::new(vec![item.name.clone(), path])
    });

    let table = Table::new(
      rows,
      [Constraint::Percentage(35), Constraint::Percentage(65)],
    )
    .header(
      Row::new(vec!["名称", "路径"]).style(
        Style::default()
          .fg(Color::Yellow)
          .add_modifier(Modifier::BOLD),
      ),
    )
    .block(Block::default().borders(Borders::ALL).title("启动项列表"))
    .row_highlight_style(
      Style::default()
        .bg(Color::Rgb(255, 203, 107))
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("➤ ");

    frame.render_stateful_widget(table, layout[1], &mut self.state);

    let footer = Paragraph::new(self.status_line())
      .style(Style::default().fg(Color::Gray))
      .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, layout[2]);

    if let Some(confirm) = self.confirm {
      self.render_confirm(frame, confirm);
    }
  }

  fn status_line(&self) -> Line<'_> {
    if self.confirm.is_some() {
      let message = self
        .status
        .as_ref()
        .map(|status| status.text.as_str())
        .unwrap_or("确认删除选中项?");
      return Line::from(Span::styled(
        message,
        Style::default().fg(Color::LightYellow),
      ));
    }

    if let Some(status) = &self.status {
      return Line::from(vec![
        Span::styled(
          status.text.as_str(),
          Style::default().fg(Color::LightYellow),
        ),
      ]);
    }

    Line::from(vec![
      Span::styled(
        "↑/↓ 选择  d 删除  q 退出",
        Style::default().fg(Color::LightYellow),
      ),
      Span::raw("   "),
      Span::styled("Home/End 跳转到首项/末项", Style::default().fg(Color::Gray)),
    ])
  }

  fn render_confirm(&self, frame: &mut Frame, confirm: ConfirmDialog) {
    let area = centered_rect(60, 30, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
      .borders(Borders::ALL)
      .style(Style::default().bg(Color::Rgb(58, 35, 89)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let name = self
      .items
      .get(confirm.index)
      .map(|item| item.name.as_str())
      .unwrap_or("所选项目");

    let title = Line::from(vec![
      Span::styled("确认删除 ", Style::default().fg(Color::LightRed)),
      Span::styled(name, Style::default().add_modifier(Modifier::BOLD)),
      Span::raw(" ?"),
    ]);

    let layout = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Min(1), Constraint::Length(3)])
      .split(inner);

    let paragraph = Paragraph::new(title)
      .style(Style::default().fg(Color::White))
      .alignment(Alignment::Center);
    frame.render_widget(paragraph, layout[0]);

    let buttons = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
      .split(layout[1]);

    let ok_selected = confirm.selection == ConfirmChoice::Confirm;
    let ok_style = if ok_selected {
      Style::default()
        .fg(Color::LightGreen)
        .add_modifier(Modifier::BOLD)
    } else {
      Style::default().fg(Color::Gray)
    };
    let ok_border = if ok_selected {
      Style::default().fg(Color::LightGreen)
    } else {
      Style::default().fg(Color::Gray)
    };

    let cancel_selected = confirm.selection == ConfirmChoice::Cancel;
    let cancel_style = if cancel_selected {
      Style::default()
        .fg(Color::LightYellow)
        .add_modifier(Modifier::BOLD)
    } else {
      Style::default().fg(Color::Gray)
    };
    let cancel_border = if cancel_selected {
      Style::default().fg(Color::LightYellow)
    } else {
      Style::default().fg(Color::Gray)
    };

    let ok_button = Paragraph::new(Line::from(Span::styled("Y 确认", ok_style)))
      .alignment(Alignment::Center)
      .block(
        Block::default()
          .borders(Borders::ALL)
          .border_style(ok_border),
      );

    let cancel_button = Paragraph::new(Line::from(Span::styled("N 取消", cancel_style)))
      .alignment(Alignment::Center)
      .block(
        Block::default()
          .borders(Borders::ALL)
          .border_style(cancel_border),
      );

    frame.render_widget(ok_button, buttons[0]);
    frame.render_widget(cancel_button, buttons[1]);
  }

  fn prompt_delete(&mut self) {
    let Some(index) = self.state.selected() else {
      self.set_status("无可删除的启动项");
      return;
    };

    self.confirm = Some(ConfirmDialog {
      index,
      selection: ConfirmChoice::Confirm,
    });
    self.set_status("确认删除选中项?");
  }

  fn cancel_delete(&mut self) {
    self.confirm = None;
    self.set_status("已取消删除操作");
  }

  fn confirm_choice(&self) -> Option<ConfirmChoice> {
    self.confirm.map(|confirm| confirm.selection)
  }

  fn set_confirm_choice(&mut self, choice: ConfirmChoice) {
    if let Some(confirm) = self.confirm.as_mut() {
      confirm.selection = choice;
    }
  }

  fn toggle_confirm_choice(&mut self) {
    if let Some(confirm) = self.confirm.as_mut() {
      confirm.selection = match confirm.selection {
        ConfirmChoice::Confirm => ConfirmChoice::Cancel,
        ConfirmChoice::Cancel => ConfirmChoice::Confirm,
      };
    }
  }

  fn apply_delete(&mut self, delete_item: &mut impl FnMut(&LoginItem) -> io::Result<()>) {
    let Some(confirm) = self.confirm else {
      return;
    };

    let Some(item) = self.items.get(confirm.index).cloned() else {
      self.set_status("无法定位所选项");
      return;
    };

    if item.path.is_none() {
      self.set_status("该启动项无可删除路径");
      return;
    }

    match delete_item(&item) {
      Ok(()) => {
        self.items.remove(confirm.index);
        self.set_status("已删除启动项");
        self.fix_selection(confirm.index);
      }
      Err(error) => {
        self.set_status(format!("删除失败: {error}"));
      }
    }
  }

  fn set_status(&mut self, message: impl Into<String>) {
    self.status = Some(StatusMessage {
      text: message.into(),
      created_at: Instant::now(),
    });
  }

  fn prune_status(&mut self) {
    if self.confirm.is_some() {
      return;
    }

    let Some(status) = &self.status else {
      return;
    };

    if status.created_at.elapsed() >= Duration::from_secs(2) {
      self.status = None;
    }
  }

  fn fix_selection(&mut self, removed_index: usize) {
    if self.items.is_empty() {
      self.state.select(None);
      return;
    }

    let next = removed_index.min(self.items.len() - 1);
    self.state.select(Some(next));
  }

  fn select_previous(&mut self) {
    let Some(current) = self.state.selected() else {
      return;
    };

    let next = current.saturating_sub(1);
    self.state.select(Some(next));
  }

  fn select_next(&mut self) {
    let Some(current) = self.state.selected() else {
      return;
    };

    let next = (current + 1).min(self.items.len().saturating_sub(1));
    self.state.select(Some(next));
  }

  fn select_first(&mut self) {
    if !self.items.is_empty() {
      self.state.select(Some(0));
    }
  }

  fn select_last(&mut self) {
    if !self.items.is_empty() {
      self.state.select(Some(self.items.len() - 1));
    }
  }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
  let vertical = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Percentage((100 - percent_y) / 2),
      Constraint::Percentage(percent_y),
      Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

  Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Percentage((100 - percent_x) / 2),
      Constraint::Percentage(percent_x),
      Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}
