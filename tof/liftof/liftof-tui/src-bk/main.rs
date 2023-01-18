mod menu;
mod dashboard_data;

use menu::MenuItem;

use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use rand::{distributions::Alphanumeric, prelude::*};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
use tui::{
    symbols,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block,
        BorderType,
        Borders,
        Cell,
        List,
        ListItem,
        ListState,
        Paragraph,
        Row,
        Table,
        Tabs,
        Dataset, 
        Chart,
        Axis,
        GraphType,
        Sparkline,
        Widget
    },
    Terminal,
};

// my stuff 
use tui_logger::TuiLoggerWidget;
use tui_logger;
use zmq;
use crossbeam_channel::{Sender, 
                        Receiver,
                        unbounded};
use dashboard_data::DashboardData;

/// The 0MQ PUB port is defined as DATAPORT_START + readoutboard_id
const DATAPORT_START : u32 = 30000;

/// The 0MP REP port is defined as CMDPORT_START + readoutboard_id
const CMDPORT_START  : u32 = 40000;



const DB_PATH: &str = "./data/db.json";

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Serialize, Deserialize, Clone)]
struct Pet {
    id: usize,
    name: String,
    category: String,
    age: usize,
    created_at: DateTime<Utc>,
}

fn render_logs<'a>() -> TuiLoggerWidget<'a> {
    TuiLoggerWidget::default()
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Gray))
        .style_info(Style::default().fg(Color::Blue))
        .block(
            Block::default()
                .title("Logs")
                .border_style(Style::default().fg(Color::White).bg(Color::Black))
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White).bg(Color::Black))
}

fn render_dashboard<'a>(update : Receiver<DashboardData>) -> Chart<'a> {
  let datasets = vec![
        Dataset::default()
              .name("data1")
              .marker(symbols::Marker::Dot)
              .graph_type(GraphType::Scatter)
              .style(Style::default().fg(Color::Cyan))
              .data(&[(0.0, 5.0), (1.0, 6.0), (1.5, 6.434)]),
          Dataset::default()
              .name("data2")
              .marker(symbols::Marker::Braille)
              .graph_type(GraphType::Line)
              .style(Style::default().fg(Color::Magenta))
              .data(&[(4.0, 5.0), (5.0, 8.0), (7.66, 13.5)]),
      ];
  
  let chart = Chart::new(datasets)
    .block(Block::default().title("Chart"))
    .x_axis(Axis::default()
        .title(Span::styled("X Axis", Style::default().fg(Color::Red)))
        .style(Style::default().fg(Color::White))
        .bounds([0.0, 10.0])
        .labels(["0.0", "5.0", "10.0"].iter().cloned().map(Span::from).collect()))
    .y_axis(Axis::default()
        .title(Span::styled("Y Axis", Style::default().fg(Color::Red)))
        .style(Style::default().fg(Color::White))
        .bounds([0.0, 10.0])
        .labels(["0.0", "5.0", "10.0"].iter().cloned().map(Span::from).collect()))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );

  let dashboard = Sparkline::default()
    .block(Block::default().title("Sparkline").borders(Borders::ALL))
    .data(&[0, 2, 3, 4, 1, 4, 10])
    .max(5)
    .style(Style::default().fg(Color::Red).bg(Color::White))
    //.alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );
    //dashboard
    chart
}

fn render_home<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "pet-CLI",
            Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Press 'p' to access pets, 'a' to add random new pets and 'd' to delete the currently selected pet.")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );
    home
}

fn render_pets<'a>(pet_list_state: &ListState) -> (List<'a>, Table<'a>) {
    let pets = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Pets")
        .border_type(BorderType::Plain);

    let pet_list = read_db().expect("can fetch pet list");
    let items: Vec<_> = pet_list
        .iter()
        .map(|pet| {
            ListItem::new(Spans::from(vec![Span::styled(
                pet.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_pet = pet_list
        .get(
            pet_list_state
                .selected()
                .expect("there is always a selected pet"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(pets).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let pet_detail = Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(selected_pet.id.to_string())),
        Cell::from(Span::raw(selected_pet.name)),
        Cell::from(Span::raw(selected_pet.category)),
        Cell::from(Span::raw(selected_pet.age.to_string())),
        Cell::from(Span::raw(selected_pet.created_at.to_string())),
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "ID",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Name",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Category",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Age",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Created At",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Detail")
            .border_type(BorderType::Plain),
    )
    .widths(&[
        Constraint::Percentage(5),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(5),
        Constraint::Percentage(20),
    ]);

    (list, pet_detail)
}

fn read_db() -> Result<Vec<Pet>, Error> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Pet> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}

fn add_random_pet_to_db() -> Result<Vec<Pet>, Error> {
    let mut rng = rand::thread_rng();
    let db_content = fs::read_to_string(DB_PATH)?;
    let mut parsed: Vec<Pet> = serde_json::from_str(&db_content)?;
    let catsdogs = match rng.gen_range(0, 1) {
        0 => "cats",
        _ => "dogs",
    };

    let random_pet = Pet {
        id: rng.gen_range(0, 9999999),
        name: rng.sample_iter(Alphanumeric).take(10).collect(),
        category: catsdogs.to_owned(),
        age: rng.gen_range(1, 15),
        created_at: Utc::now(),
    };

    parsed.push(random_pet);
    fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
    Ok(parsed)
}

fn remove_pet_at_index(pet_list_state: &mut ListState) -> Result<(), Error> {
    if let Some(selected) = pet_list_state.selected() {
        let db_content = fs::read_to_string(DB_PATH)?;
        let mut parsed: Vec<Pet> = serde_json::from_str(&db_content)?;
        parsed.remove(selected);
        fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
        let amount_pets = read_db().expect("can fetch pet list").len();
        if selected > 0 {
            pet_list_state.select(Some(selected - 1));
        } else {
            pet_list_state.select(Some(0));
        }
    }
    Ok(())
}



fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Set max_log_level to Trace
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();

    // Set default level for unknown targets to Trace
    tui_logger::set_default_level(log::LevelFilter::Trace);

    // First setup, then enable the raw mode for the 
    // terminal. This is important, otherwise, we 
    // won't see the problems during setup.
    // setup zmq here (for now)
    let address_ip    = String::from("tcp://127.0.0.1");
    //let data_addres   = String::from("tcp://10.0.1.151");
    let cmd_port     = CMDPORT_START;//+ get_board_id().unwrap();
    let cmd_address : String = address_ip.clone() + ":" + &cmd_port.to_string();
  
    let data_port    = DATAPORT_START;// + get_board_id().unwrap();
    let data_address : String = address_ip + ":" + &data_port.to_string();
    
    let ctx = zmq::Context::new();
    
    let data_socket = ctx.socket(zmq::SUB).expect("Unable to create 0MQ REP socket!");
    //info!("Will set up 0MQ REP socket at address {cmd_address}");
    data_socket.connect(&data_address);
    //info!("0MQ REP socket ocnnected at {cmd_address}");
    let topic = b"";
    data_socket.set_subscribe(topic);
    let data_response = data_socket.recv_bytes(zmq::DONTWAIT);
    //let resp =  String::from_utf8(data_response).expect("Got garbage response from client. If we start like this, I panic right away...");
    //println!("Connected to RB! Response {resp}");
    //panic!("Youre done! {}", resp);
    let (dashbrd_update, dashbrd_get_update)    : (Sender<DashboardData>, Receiver<DashboardData>)       = unbounded();


    //let cmd_socket = ctx.socket(zmq::REQ).expect("Unable to create 0MQ REP socket!");
    ////info!("Will set up 0MQ REP socket at address {cmd_address}");
    //cmd_socket.connect(&cmd_address);
    ////info!("0MQ REP socket ocnnected at {cmd_address}");
    //// block until we get a client
    //let ping = String::from("[LIFTOF-TUI] - ping");
    //cmd_socket.send(ping.as_bytes(), 0);
    //let server_response = cmd_socket.recv_bytes(0).expect("Communication to client failed!");
    //let resp =  String::from_utf8(server_response).expect("Got garbage response from client. If we start like this, I panic right away...");
    ////println!("Connected to RB! Response {resp}");
    //panic!("Youre done! {}", resp);
    // RATE dataset
    //let datasets = Vec::<Dataset<'a>>::new();

    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_titles = vec!["Home", "Status", "Alerts", "Commands", "Dashboard", "Logs", "Quit"];
    let mut active_menu_item = MenuItem::Home;
    let mut pet_list_state = ListState::default();
    pet_list_state.select(Some(0));


    // main loop
    loop {
      terminal.draw(|rect| {
        let size = rect.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(2),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(size);

        let footer_logs = Paragraph::new("pet-CLI 2020 - all rights reserved")
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("Logs")
                    .border_type(BorderType::Plain),
            );

        let menu = menu_titles
            .iter()
            .map(|t| {
                let (first, rest) = t.split_at(1);
                Spans::from(vec![
                    Span::styled(
                        first,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED),
                    ),
                    Span::styled(rest, Style::default().fg(Color::White)),
                ])
            })
            .collect();

        let tabs = Tabs::new(menu)
            .select(active_menu_item.into())
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider(Span::raw("|"));

        rect.render_widget(tabs, chunks[0]);
        match active_menu_item {
            MenuItem::Alerts    => (),
            MenuItem::Dashboard => {
              let updater = dashbrd_get_update.clone();
              rect.render_widget(render_dashboard(updater), chunks[1]);
            },
            MenuItem::Commands  => (),
            MenuItem::Home      => rect.render_widget(render_home(), chunks[1]),
            MenuItem::Logs      => {
              let r_logs = render_logs();
              rect.render_widget(r_logs, chunks[1]);
            }
            MenuItem::Status => {
                let pets_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                    )
                    .split(chunks[1]);
                let (left, right) = render_pets(&pet_list_state);
                rect.render_stateful_widget(left, pets_chunks[0], &mut pet_list_state);
                rect.render_widget(right, pets_chunks[1]);
            }
        }
        rect.render_widget(render_logs(), chunks[2]);
      })?;

      // scan for keys here
      match rx.recv()? {
        Event::Input(event) => match event.code {
          KeyCode::Char('q') => {
            disable_raw_mode()?;
            terminal.show_cursor()?;
            break;
          }
          KeyCode::Char('h') => active_menu_item = MenuItem::Home,
          KeyCode::Char('s') => active_menu_item = MenuItem::Status,
          KeyCode::Char('d') => active_menu_item = MenuItem::Dashboard,
          KeyCode::Char('l') => active_menu_item = MenuItem::Logs,
          KeyCode::Char('a') => active_menu_item = MenuItem::Alerts,
          KeyCode::Char('c') => active_menu_item = MenuItem::Commands,

          KeyCode::Down => {
              if let Some(selected) = pet_list_state.selected() {
                  let amount_pets = read_db().expect("can fetch pet list").len();
                  if selected >= amount_pets - 1 {
                      pet_list_state.select(Some(0));
                  } else {
                      pet_list_state.select(Some(selected + 1));
                  }
              }
          }
          KeyCode::Up => {
              if let Some(selected) = pet_list_state.selected() {
                  let amount_pets = read_db().expect("can fetch pet list").len();
                  if selected > 0 {
                      pet_list_state.select(Some(selected - 1));
                  } else {
                      pet_list_state.select(Some(amount_pets - 1));
                  }
              }
          }
          _ => {}
        },
        Event::Tick => {}
      }
    }

    Ok(())
}

