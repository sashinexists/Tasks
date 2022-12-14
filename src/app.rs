use crate::task::{self, Task, TimeOfDay, Weather};
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use kitchen_fridge::{
    calendar::cached_calendar::CachedCalendar,
    traits::{CalDavSource, CompleteCalendar, DavCalendar},
    CalDavProvider, Item,
};
use url::Url;
use uuid::Uuid;

pub struct App {
    pub provider: CalDavProvider,
    pub source_url: Url,
    tasks: Vec<Task>,
    events: Log,
}

impl App {
    pub async fn new(mut provider: CalDavProvider, source_url: Url) -> Self {
        provider.sync().await;
        let local_calendar = provider.local().get_calendar(&source_url).await.unwrap();
        let local_calendar = &*local_calendar.lock().unwrap();
        let items: Vec<Item> = local_calendar
            .get_items()
            .await
            .unwrap()
            .into_iter()
            .map(|(_, item)| item.clone())
            .collect();
        let tasks = get_tasks_from_items(items).await;
        Self {
            provider,
            source_url,
            tasks,
            events: Log::new(),
        }
    }

    fn update(tasks: &[Task], event: Message) -> Vec<Task> {
        //this will match on the event and make a change depending on it
        match event {
            Message::SetName(task_id, name) => {
                App::perform_action(tasks, task_id, |t: Task| t.set_name(name.clone()))
            }
            Message::MarkComplete(task_id) => {
                App::perform_action(tasks, task_id, |t: Task| t.mark_complete())
            }
            Message::MarkIncomplete(task_id) => {
                App::perform_action(tasks, task_id, |t: Task| t.mark_incomplete())
            }
            Message::SetStartDate(task_id, start_date) => {
                App::perform_action(tasks, task_id, |t: Task| t.set_start_date(start_date))
            }
            Message::SetDueDate(task_id, due_date) => {
                App::perform_action(tasks, task_id, |t: Task| t.set_due_date(due_date))
            }
            Message::AddContext(task_id, context) => {
                App::perform_action(tasks, task_id, |t: Task| -> Task {
                    t.add_context(context.clone())
                })
            }
            Message::RemoveContext(task_id, context) => {
                App::perform_action(tasks, task_id, |t: Task| -> Task {
                    t.remove_context(context.clone())
                })
            }
            Message::AddProject(task_id, project) => {
                App::perform_action(tasks, task_id, |t: Task| -> Task {
                    t.add_project(project.clone())
                })
            }
            Message::RemoveProject(task_id, project) => {
                App::perform_action(tasks, task_id, |t: Task| -> Task {
                    t.remove_project(project.clone())
                })
            }
            Message::AddArea(task_id, area) => {
                App::perform_action(tasks, task_id, |t: Task| -> Task {
                    t.add_area(area.clone())
                })
            }
            Message::RemoveArea(task_id, area) => {
                App::perform_action(tasks, task_id, |t: Task| -> Task {
                    t.remove_area(area.clone())
                })
            }
            Message::SetMoneyNeeded(task_id, money_needed) => {
                App::perform_action(tasks, task_id, |t: Task| -> Task {
                    t.set_money_needed(money_needed)
                })
            }
            Message::SetWeather(task_id, weather) => {
                App::perform_action(tasks, task_id, |t: Task| -> Task {
                    t.set_weather(weather.clone())
                })
            }
            Message::SetTimeOfDay(task_id, time_of_day) => {
                App::perform_action(tasks, task_id, |t: Task| -> Task {
                    t.set_time_of_day(time_of_day.clone())
                })
            }
            Message::SetParentTask(task_id, parent_task_id) => {
                App::perform_action(tasks, task_id, |t: Task| -> Task {
                    t.set_parent_task(parent_task_id)
                })
            }
            Message::AddTask(task) => {
                let mut tasks = Vec::from(tasks);
                tasks.push(task);
                tasks
            }
            Message::RemoveTask(task_to_remove_id) => {
                let tasks = Vec::from(tasks);
                tasks
                    .clone()
                    .into_iter()
                    .filter(|task| task_to_remove_id != task.id)
                    .collect()
            }
        }
    }

    pub fn perform_action<F>(tasks: &[Task], task_to_change_id: Uuid, mut action: F) -> Vec<Task>
    where
        F: FnMut(Task) -> Task,
    {
        tasks
            .into_iter()
            .map(|task| {
                if task.id == task_to_change_id {
                    action(task.clone())
                } else {
                    task.clone()
                }
            })
            .collect()
    }

    pub fn get_present_state(&self) -> Vec<Task> {
        //this will loop through all the events and update
        self.events
            .clone()
            .prev
            .into_iter()
            .fold(self.clone().tasks.clone(), |current, event| -> Vec<Task> {
                App::update(&current, event)
            })
    }

    pub fn new_event(&mut self, event: Message) {
        self.events.add(event);
    }

    pub async fn sync(&mut self) {
        let current_items = self.get_local_calendar_items().await;
        let tasks = self.get_present_state();
        for item in current_items.iter() {
            if tasks.iter().any(|task| task.id.to_string() == item.uid()) {
                println!("Updating Task {}", item.uid());
                self.provider
                    .local_mut()
                    .get_calendar(&self.source_url)
                    .await
                    .expect("Failed to get calendar")
                    .lock()
                    .expect("failed to unlock calendar")
                    .add_item_sync(
                        tasks
                            .iter()
                            .find(|task| task.id.to_string() == item.uid())
                            .expect("failed to get task")
                            .to_item(&self.source_url),
                    )
                    .expect("Failed to update item");
            } else {
                self.provider
                    .local_mut()
                    .get_calendar(&self.source_url)
                    .await
                    .expect("Failed to get calendar")
                    .lock()
                    .expect("failed to unlock calendar")
                    .mark_for_deletion_sync(item.url())
                    .expect("Failed to update item");
            }
        }

        for task in tasks.iter() {
            if !current_items
                .iter()
                .any(|item| task.id.to_string() == item.uid())
            {
                println!("Adding task {}", &task.id.to_string());
                self.provider
                    .local_mut()
                    .get_calendar(&self.source_url)
                    .await
                    .expect("Failed to get calendar")
                    .lock()
                    .expect("failed to unlock calendar")
                    .add_item_sync(task.to_item(&self.source_url))
                    .expect("Failed to update item");
            }
        }
        self.provider.sync().await;
    }

    async fn get_local_calendar(&mut self) -> CachedCalendar {
        let local_calendar = self
            .provider
            .local_mut()
            .get_calendar(&self.source_url)
            .await
            .unwrap();
        let local_calendar = local_calendar.lock().unwrap();
        local_calendar.to_owned()
    }

    async fn get_local_calendar_items(&mut self) -> Vec<Item> {
        let local_calendar = self.get_local_calendar().await;
        local_calendar
            .get_items()
            .await
            .unwrap()
            .into_iter()
            .map(|(_, item)| item.clone())
            .collect()
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
pub enum Message {
    SetName(Uuid, String),
    MarkComplete(Uuid),
    MarkIncomplete(Uuid),
    SetStartDate(Uuid, Option<NaiveDateTime>),
    SetDueDate(Uuid, Option<NaiveDateTime>),
    AddContext(Uuid, String),
    RemoveContext(Uuid, String),
    AddProject(Uuid, String),
    RemoveProject(Uuid, String),
    AddArea(Uuid, String),
    RemoveArea(Uuid, String),
    SetMoneyNeeded(Uuid, bool),
    SetWeather(Uuid, Option<Weather>),
    SetTimeOfDay(Uuid, Option<TimeOfDay>),
    SetParentTask(Uuid, Option<Uuid>),
    AddTask(Task),
    RemoveTask(Uuid),
}

async fn get_tasks_from_items(items: Vec<Item>) -> Vec<task::Task> {
    let mut tasks: Vec<task::Task> = Vec::new();
    for item in items {
        let task = task::Task::from_item(item);
        tasks.push(task);
    }
    tasks
}
