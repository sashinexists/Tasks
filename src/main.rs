use std::path::Path;

use dotenvy::dotenv;
use kitchen_fridge::{
    traits::{CalDavSource, DavCalendar},
    *,
};
mod app;
mod task;
mod utils;
use url::Url;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let calendar_provider = get_calendar().await;
    let calendar_url =
        Url::parse("https://sashin.online/remote.php/dav/calendars/sashin/rust-playground/")
            .unwrap();
    let mut app = app::App::new(calendar_provider, calendar_url).await;

    let new_task = task::Task::new("Adding a new task".to_string());
    app.new_event(app::Message::AddTask(new_task.clone()));
    app.new_event(app::Message::MarkComplete(new_task.clone()));
    app.new_event(app::Message::AddContext(
        new_task.clone(),
        "Laptop".to_string(),
    ));
    app.new_event(app::Message::SetName(
        new_task.clone(),
        "This Task has been renamed".to_string(),
    ));
    println!("{:#?}", app.get_present_state());
    // calendar_provider.sync().await;
}

async fn get_calendar() -> CalDavProvider {
    let username: String = std::env::var("USERNAME").unwrap();
    let password: String = std::env::var("PASSWORD").unwrap();
    let client = Client::new(
        "https://sashin.online/remote.php/dav/calendars/sashin/tasks/",
        username,
        password,
    )
    .unwrap();
    let cache = Cache::new(Path::new("tasks_data"));
    CalDavProvider::new(client, cache)
}

// async fn get_tasks_from_items(items: Vec<Item>) -> Vec<task::Task> {
//     let mut tasks: Vec<task::Task> = Vec::new();
//     for item in items {
//         let task = task::Task::from_item(item);
//         tasks.push(task);
//     }
//     tasks
// }
