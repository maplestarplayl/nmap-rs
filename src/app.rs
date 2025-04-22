use std::{
    collections::HashMap, io, sync::{Arc, Mutex}, time::{Duration, Instant}
};
use crate::{net::{self, parse_cidr, Results}, scan::ScanResult, ui::draw};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;
// Refresh time in ms
const TICK_TIME: Duration = Duration::from_millis(300);
#[derive(PartialEq, Eq)]
pub enum FocusZone {
    AddressList,
    PortList,
    InputList,
}
pub struct App {
    state: Arc<Mutex<Results>>,
    pub targets: Vec<String>,
    pub ports: String,
    pub targets_selected: usize, 
    pub port_results: HashMap<String, Vec<ScanResult>>,
    pub port_results_selected: usize,
    pub input_selected: usize,
    pub progress: f32,
    pub focus_zone: FocusZone,
    exit: bool,
    pub input_trigger: bool,
    pub ready_to_run: bool,
    pub input_mode: bool,
    pub target_input: String,
    pub port_input: String,
    pub total_targets: usize,
}

impl App {
    pub fn init(state: Arc<Mutex<Results>>) -> Self{
        Self {
            state,
            targets: Vec::new(),
            ports: String::new(),
            targets_selected: 0,
            port_results: HashMap::new(),
            port_results_selected: 0,
            input_selected: 0,
            progress: 0.0,
            focus_zone: FocusZone::InputList,
            exit: false,
            input_trigger: false,
            ready_to_run: false,
            input_mode: false,
            target_input: String::new(),
            port_input: String::new(),
            total_targets: 0,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut last_tick = Instant::now();

        loop {
            if self.total_targets > 0 {
                self.progress = self.state.lock().unwrap().len() as f32 / self.total_targets as f32;
            }
            terminal.draw(|frame| draw(frame, self))?;
            let timeout = TICK_TIME.saturating_sub(last_tick.elapsed());
            match self.input_mode {
                true => self.handle_input_events()?,
                false => self.handle_events(timeout)?,
            }
            if self.ready_to_run {
                self.target_input.split(" ").for_each(|s| self.targets.push(s.to_string()));
                self.total_targets = self.targets.iter().map(|target| net::parse_cidr(target).unwrap().len()).sum();
                self.ports = self.port_input.clone();
                // clone data
                let state = self.state.clone();
                let targets = self.targets.clone();
                let ports = self.ports.clone(); 
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        net::execute_scan(state, targets, ports).await;
                    });
                });
                self.ready_to_run = false;
            }

            if last_tick.elapsed() > TICK_TIME {
                self.on_tick(); //TODO
                last_tick = Instant::now();
            }

            if self.exit {
                break;
            }
        }
        Ok(())
    }
    fn handle_events(&mut self, timeout: Duration) -> io::Result<()> {
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Tab => self.focus_zone = match self.focus_zone {
                FocusZone::AddressList => {
                    self.port_results_selected = 0;
                    FocusZone::PortList
                },
                FocusZone::PortList => {
                    self.input_selected = 0;
                    FocusZone::InputList
                },
                FocusZone::InputList => {
                    self.targets_selected = 0;
                    FocusZone::AddressList
                },
            },
            KeyCode::Up => {
                match self.focus_zone {
                    FocusZone::AddressList => {
                        if self.targets_selected > 0 {
                            self.targets_selected -= 1;
                        }
                    }
                    FocusZone::PortList => {
                        let target = self.targets.get(self.targets_selected);
                        if target.is_some() {
                            let port_results = self.port_results.get(target.unwrap());
                            if port_results.is_some() {
                                if self.port_results_selected > 0 {
                                    self.port_results_selected -= 1;
                                }
                            }
                        }
                    },
                    FocusZone::InputList => {
                        if self.input_selected > 0 {
                            self.input_selected -= 1;
                        }
                    },
                }
            }
            KeyCode::Down => {
                match self.focus_zone {
                    FocusZone::AddressList => {
                        if self.targets_selected + 1 < self.targets.len() {
                            self.targets_selected += 1;
                        }
                    }
                    FocusZone::PortList => {
                        let target = self.targets.get(self.targets_selected);
                        if target.is_some() {
                            let port_results = self.port_results.get(target.unwrap());
                            if port_results.is_some() {
                                if self.port_results_selected + 1 < port_results.unwrap().len() {
                                    self.port_results_selected += 1;
                                }
                            }
                        }
                    },
                    FocusZone::InputList => {
                        if self.input_selected + 1 < 2 {
                            self.input_selected += 1;
                        }
                    },
                }
            }
            KeyCode::Enter => {
                self.input_trigger = !self.input_trigger;
                self.input_mode = true;
            }
            KeyCode::Char('e') => {
                self.ready_to_run = true;
            }
            _ => {}
        }
    }

    fn handle_input_events(&mut self) -> io::Result<()> {
        let key = event::read()?;
        match key {
            Event::Key(key_event) => {
                match key_event.code {
                    KeyCode::Enter => {
                        self.input_trigger = !self.input_trigger;
                        self.input_mode = false;
                    },
                    KeyCode::Char(c) => {
                        match self.input_selected {
                            0 => self.target_input.push(c),
                            1 => self.port_input.push(c),
                            _ => {}
                        }
                    },
                    KeyCode::Backspace => {
                        match self.input_selected {
                            0 => self.target_input.pop(),
                            1 => self.port_input.pop(),
                            _ => Option::None
                        };
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        Ok(())
    }
    fn on_tick(&mut self) {
        self.targets = self.state.lock().unwrap().keys().cloned().collect();
        self.port_results = self.state.lock().unwrap().clone();
    }
}
