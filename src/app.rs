use std::{
    process::Command,
    time::{Duration, Instant},
};

use ratatui::{
    prelude::Color,
    widgets::TableState,
};

use crate::system_data::{ProcessInfo, get_system_processes};

pub const REFRESH_RATE: u64 = 2;

#[derive(PartialEq, Copy, Clone)]
pub enum SortColumn {
    Pid,
    Name,
    Memory,
}

#[derive(PartialEq, Copy, Clone)]
pub enum InputMode {
    Normal,
    Search,
    ConfirmKill,
}

pub struct App {
    pub processes: Vec<ProcessInfo>,
    pub table_state: TableState,
    pub last_refresh: Instant,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub input_mode: InputMode,
    pub search_query: String,
    pub filtered_processes: Vec<usize>, // Indices to processes
    pub message: Option<(String, Color)>,
    pub message_time: Option<Instant>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            processes: get_system_processes(),
            table_state: TableState::default(),
            last_refresh: Instant::now(),
            sort_column: SortColumn::Memory,
            sort_ascending: false,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            filtered_processes: Vec::new(),
            message: None,
            message_time: None,
        };

        app.sort_processes();
        app.apply_filters();
        app.table_state.select(Some(0));
        app
    }

    pub fn sort_processes(&mut self) {
        match self.sort_column {
            SortColumn::Pid => self.processes.sort_by(|a, b| {
                let a_num = a.pid.parse::<u32>().unwrap_or(0);
                let b_num = b.pid.parse::<u32>().unwrap_or(0);
                if self.sort_ascending {
                    a_num.cmp(&b_num)
                } else {
                    b_num.cmp(&a_num)
                }
            }),
            SortColumn::Name => self.processes.sort_by(|a, b| {
                if self.sort_ascending {
                    a.name.cmp(&b.name)
                } else {
                    b.name.cmp(&a.name)
                }
            }),
            SortColumn::Memory => self.processes.sort_by(|a, b| {
                if self.sort_ascending {
                    a.memory_mb
                        .partial_cmp(&b.memory_mb)
                        .unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    b.memory_mb
                        .partial_cmp(&a.memory_mb)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
            }),
        }
    }

    pub fn toggle_sort(&mut self, column: SortColumn) {
        if self.sort_column == column {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = column;
            self.sort_ascending = true;
        }
        self.sort_processes();
        self.apply_filters();
    }

    pub fn apply_filters(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_processes = (0..self.processes.len()).collect();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_processes = self
                .processes
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    p.name.to_lowercase().contains(&query) || p.pid.to_lowercase().contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }

        // Adjust the selection to be within bounds
        if let Some(selected) = self.table_state.selected() {
            if self.filtered_processes.is_empty() {
                self.table_state.select(None);
            } else if selected >= self.filtered_processes.len() {
                self.table_state
                    .select(Some(self.filtered_processes.len() - 1));
            }
        }
    }

    pub fn refresh(&mut self) {
        if self.last_refresh.elapsed() >= Duration::from_secs(REFRESH_RATE) {
            let selected_pid = self.selected_process().map(|p| p.pid.clone());
            self.processes = get_system_processes();
            self.sort_processes();
            self.apply_filters();
            self.last_refresh = Instant::now();

            // Try to maintain selection by PID
            if let Some(pid) = selected_pid {
                if let Some(index) = self
                    .filtered_processes
                    .iter()
                    .position(|&i| self.processes[i].pid == pid)
                {
                    self.table_state.select(Some(index));
                }
            }
        }

        // Clear message after timeout
        if let Some(time) = self.message_time {
            if time.elapsed() > Duration::from_secs(3) {
                self.message = None;
                self.message_time = None;
            }
        }
    }

    pub fn next(&mut self) {
        if self.filtered_processes.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => (i + 1) % self.filtered_processes.len(),
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.filtered_processes.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => (i + self.filtered_processes.len() - 1) % self.filtered_processes.len(),
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn kill_selected_process(&mut self) {
        if let Some(process) = self.selected_process() {
            let pid = process.pid.parse::<u32>().unwrap_or(0);
            if pid > 0 {
                #[cfg(target_os = "windows")]
                let kill_result = Command::new("taskkill")
                    .args(["/F", "/PID", &process.pid])
                    .output();

                #[cfg(not(target_os = "windows"))]
                let kill_result = Command::new("kill").arg(&process.pid).output();

                match kill_result {
                    Ok(_) => {
                        self.set_message(format!("Process {} killed", process.name), Color::Green);
                        // Immediately refresh process list
                        self.last_refresh = Instant::now()
                            .checked_sub(Duration::from_secs(REFRESH_RATE + 1))
                            .unwrap_or(Instant::now());
                    }
                    Err(e) => {
                        self.set_message(format!("Failed to kill process: {}", e), Color::Red);
                    }
                }
            }
        }
        self.input_mode = InputMode::Normal;
    }

    pub fn set_message(&mut self, message: String, color: Color) {
        self.message = Some((message, color));
        self.message_time = Some(Instant::now());
    }

    pub fn selected_process(&self) -> Option<&ProcessInfo> {
        self.table_state.selected().and_then(|i| {
            self.filtered_processes
                .get(i)
                .map(|&idx| &self.processes[idx])
        })
    }
}