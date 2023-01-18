
#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
  Home,
  Status,
  Alerts,
  Commands,
  Dashboard,
  Logs
}


impl From<MenuItem> for usize {
  fn from(input: MenuItem) -> usize {
    match input {
      MenuItem::Home      => 0,
      MenuItem::Status    => 1,
      MenuItem::Alerts    => 2,
      MenuItem::Commands  => 3,
      MenuItem::Dashboard => 4,
      MenuItem::Logs      => 5
    }
  }
}

