use tokio;
use thiserror::Error;

use time::{Time, OffsetDateTime};
use serde::{Deserialize};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    time::{Duration, Instant, SystemTime},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame, Terminal,
};


const DATA: [(f64, f64); 5] = [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (3.0, 3.0), (4.0, 4.0)];

#[derive(Clone)]
pub struct SinSignal {
    x: f64,
    interval: f64,
    period: f64,
    scale: f64,
}

impl SinSignal {
    pub fn new(interval: f64, period: f64, scale: f64) -> SinSignal {
        SinSignal {
            x: 0.0,
            interval,
            period,
            scale,
        }
    }
}

impl Iterator for SinSignal {
    type Item = (f64, f64);
    fn next(&mut self) -> Option<Self::Item> {
        let point = (self.x, (self.x * 1.0 / self.period).sin() * self.scale);
        self.x += self.interval;
        Some(point)
    }
}

struct App {
    data1: Vec<(f64, f64)>,
    window: [f64; 2],
}

impl App {
    fn new(points_to_be_mapped: Vec<(f64, f64)>) -> App {
        let data1 = points_to_be_mapped;

        App {
            data1,
            window: [0.0, 20.0],
        }
    }

    fn on_tick(&mut self) {
        for _ in 0..5 {
            self.data1.remove(0);
        }
        self.window[0] += 1.0;
        self.window[1] += 1.0;
    }
}

#[derive(Error, Debug)]
pub enum UpwardError {
    #[error("the API for key `{0}` is not available")]
    MissingKey(String),
    #[error("unknown data store error")]
    Unknown,
}

#[derive(Deserialize, Debug)]
pub struct ResponseTables{
    list: Vec<LendableTable>,
}

#[derive(Deserialize, Debug)]
pub struct LendableTable {
    result: Option<String>,
    table: Option<i32>,
    #[serde(alias = "_start")] 
    start: Option<String>,
    #[serde(alias = "_stop")] 
    stop:  Option<String>,
    #[serde(alias = "_time")] 
    time:  Option<String>,
    #[serde(alias = "_value")] 
    value: Option<f64>,
    #[serde(alias = "_field")] 
    field: Option<String>,
    #[serde(alias = "_measurement")] 
    measurement: Option<String>,
    country: Option<String>,
    currency: Option<String>,
    isin: Option<String>,
    name: Option<String>,
    source: Option<String>,
    symbol: Option<String>,
}



#[tokio::main]
async fn main() {

    let influx_url =    dotenvy::var("SERVER_URL").unwrap();
    let _org = dotenvy::var("ORG").unwrap();
    let org_id = dotenvy::var("ORGID").unwrap();
    let token = dotenvy::var("INFLUX_AUTH_TOKEN").unwrap();
    let bucket = dotenvy::var("BUCKET").unwrap();
    let query = format!("from(bucket: \"{bucket}\") |> range(start: -1h) |> filter(fn: (r) => r.isin ==\"CA94947L1022\")");

    let req_url = format!(r"{}/api/v2/query\?orgID={}", influx_url, &org_id);
    let client = reqwest::Client::new();
    let csv_response = client.post( &req_url)
    .header("Authorization", format!("Token {}", token))
    .header("Accept", "application/csv")
    .header("Content-Type", "application/vnd.flux")
    .body(query)
    .send()
    .await.unwrap().text().await.unwrap();

    let mut reader = csv::Reader::from_reader(csv_response.as_bytes());

    let mut records = Vec::<LendableTable>::new();

    for record in reader.deserialize() {
        let record: LendableTable = record.unwrap();
        records.push(record);
    }
   //let mut dataset = Vec::new();
  // Duration::
   //let x = SystemTime::deserialize(records.get(1).unwrap().time.unwrap()).unwrap();
//    dataset.push((records.get(1).unwrap().value.unwrap(),10000_f64));
//     // setup terminal
//      enable_raw_mode().unwrap();
//     let mut stdout = io::stdout();
//      execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
//      let backend = CrosstermBackend::new(stdout);
//      let mut terminal = Terminal::new(backend).unwrap();

//     // // create app and run it
//     let tick_rate = Duration::from_secs(5);
//     let app = App::new(dataset);
//     let res = run_app(&mut terminal, app, tick_rate);

//     // // restore terminal
//     disable_raw_mode().unwrap();
//     execute!(
//         terminal.backend_mut(),
//         LeaveAlternateScreen,
//         DisableMouseCapture
//     ).unwrap();
//     terminal.show_cursor().unwrap();

//     if let Err(err) = res {
//         println!("{:?}", err)
//     }
}


fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ]
            .as_ref(),
        )
        .split(size);
    let x_labels = vec![
        Span::styled(
            format!("{}", app.window[0]),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("{}", (app.window[0] + app.window[1]) / 2.0)),
        Span::styled(
            format!("{}", app.window[1]),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ];
    let datasets = vec![
        Dataset::default()
            .name("data3")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Yellow))
            .data(&app.data1),
    ];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(
                    "Chart 1",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("X Axis")
                .style(Style::default().fg(Color::Gray))
                .labels(x_labels)
                .bounds(app.window),
        )
        .y_axis(
            Axis::default()
                .title("Y Axis")
                .style(Style::default().fg(Color::Gray))
                .labels(vec![
                    Span::styled("-20", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw("0"),
                    Span::styled("20", Style::default().add_modifier(Modifier::BOLD)),
                ])
                .bounds([-20.0, 20.0]),
        );
    f.render_widget(chart, chunks[0]);
}