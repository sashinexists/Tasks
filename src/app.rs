use crate::task::{Task, TimeOfDay, Weather};
use chrono::{DateTime, FixedOffset};
use uuid::Uuid;

struct App {
    tasks: Vec<Task>,
    events: Log,
}

impl App {
    fn update(tasks: &[Task], event: Message) -> Vec<Task> {
        //this will match on the event and make a change depending on it
        match event {
            Message::MarkComplete(task) => {
                App::perform_action(tasks, task, |t: Task| t.mark_complete())
            }
            Message::MarkIncomplete(task) => {
                App::perform_action(tasks, task, |t: Task| t.mark_incomplete())
            }
            Message::SetStartDate(task, start_date) => {
                App::perform_action(tasks, task, |t: Task| t.set_start_date(start_date))
            }
            Message::SetDueDate(task, due_date) => {
                App::perform_action(tasks, task, |t: Task| t.set_due_date(due_date))
            }
            Message::AddContext(task, context) => {
                App::perform_action(tasks, task, |t: Task| -> Task {
                    t.add_context(context.clone())
                })
            }
            Message::RemoveContext(task, context) => {
                App::perform_action(tasks, task, |t: Task| -> Task {
                    t.remove_context(context.clone())
                })
            }
            Message::AddProject(task, project) => {
                App::perform_action(tasks, task, |t: Task| -> Task {
                    t.add_project(project.clone())
                })
            }
            Message::RemoveProject(task, project) => {
                App::perform_action(tasks, task, |t: Task| -> Task {
                    t.remove_project(project.clone())
                })
            }
            Message::AddArea(task, area) => {
                App::perform_action(tasks, task, |t: Task| -> Task { t.add_area(area.clone()) })
            }
            Message::RemoveArea(task, area) => {
                App::perform_action(tasks, task, |t: Task| -> Task { t.add_area(area.clone()) })
            }
            Message::SetMoneyNeeded(task, money_needed) => {
                App::perform_action(tasks, task, |t: Task| -> Task {
                    t.set_money_needed(money_needed)
                })
            }
            Message::SetWeather(task, weather) => {
                App::perform_action(tasks, task, |t: Task| -> Task {
                    t.set_weather(weather.clone())
                })
            }
            Message::SetTimeOfDay(task, time_of_day) => {
                App::perform_action(tasks, task, |t: Task| -> Task {
                    t.set_time_of_day(time_of_day.clone())
                })
            }
            Message::SetParentTask(task, parent_task_id) => {
                App::perform_action(tasks, task, |t: Task| -> Task {
                    t.set_parent_task(parent_task_id)
                })
            }
            Message::AddTask(task) => {
                let mut tasks = Vec::from(tasks);
                tasks.push(task);
                tasks
            }
            Message::RemoveTask(task) => {
                let tasks = Vec::from(tasks);
                tasks
                    .clone()
                    .into_iter()
                    .filter(|task_to_remove| task != task_to_remove.clone())
                    .collect()
            }
        }
    }

    pub fn perform_action<F>(tasks: &[Task], task_to_change: Task, mut action: F) -> Vec<Task>
    where
        F: FnMut(Task) -> Task,
    {
        tasks
            .into_iter()
            .map(|task| {
                if task.id == task_to_change.id {
                    action(task.clone())
                } else {
                    task.clone()
                }
            })
            .collect()
    }

    fn get_present_state(&self) -> Vec<Task> {
        //this will loop through all the events and update
        self.events
            .clone()
            .prev
            .into_iter()
            .fold(self.clone().tasks.clone(), |current, event| -> Vec<Task> {
                App::update(&current, event)
            })
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
enum Message {
    MarkComplete(Task),
    MarkIncomplete(Task),
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
