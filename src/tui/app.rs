use crate::vikunja::models::Task;
use std::collections::HashMap;

pub struct App {
    pub running: bool,
    pub tasks: Vec<Task>,
    pub project_map: HashMap<i64, String>,
    pub project_colors: HashMap<i64, String>,
}

impl App {
    pub fn new() -> Self {
        Self { 
            running: true, 
            tasks: Vec::new(),
            project_map: HashMap::new(),
            project_colors: HashMap::new(),
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
