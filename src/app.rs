use crate::task::{Task, TimeOfDay, Weather};
use chrono::{DateTime, FixedOffset};
use uuid::Uuid;

struct App {
    tasks: Vec<Task>,
    events: Log,
}

impl App {
    fn update(&self, event: Message) {
        //this will match on the event and make a change depending on it
        todo!()
    }

    fn get_present_state(&self) -> Self {
        //this will loop through all the events and update
        todo!()
    }
}

struct Log {
    prev: Vec<Message>,
    next: Vec<Message>,
}

impl Log {
    fn forward(&mut self) {
        if !self.next.is_empty() {
            let new = self.next.remove(0);
            self.prev.push(new);
        }
    }

    fn back(&mut self) {
        if let Some(new) = self.prev.pop() {
            self.next.insert(0, new);
        }
    }
    pub fn add(&mut self, item: Message) {
        self.next.clear();
        self.prev.push(item);
    }

    pub fn new() -> Self {
        Self {
            prev: Vec::<Message>::new(),
            next: Vec::<Message>::new(),
        }
    }
}

enum Message {
    MarkComplete(Task),
    MarkIncomplete(Task),
    SetDescription(Task, String),
    SetStartDate(Task, Option<DateTime<FixedOffset>>),
    SetDueDate(Task, Option<DateTime<FixedOffset>>),
    AddContext(Task, String),
    RemoveContext(Task, String),
    AddProject(Task, String),
    RemoveProject(Task, String),
    AddArea(Task, String),
    RemoveArea(Task, String),
    SetMoneyNeeded(Task, bool),
    SetWeather(Task, Option<Weather>),
    SetTimeOfDay(Task, Option<TimeOfDay>),
    SetParentTask(Task, Option<Uuid>),
    AddTask(Task),
    RemoveTask(Task),
}
