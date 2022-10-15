use chrono::NaiveDateTime;
use dotenvy::dotenv;
use kitchen_fridge::{
    traits::{CalDavSource, DavCalendar},
    *,
};
use std::path::Path;
mod app;
mod task;
mod utils;
use url::Url;

use crate::task::Weather;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let calendar_provider = get_calendar().await;
    let calendar_url =
        Url::parse("https://sashin.online/remote.php/dav/calendars/sashin/rust-playground/")
            .unwrap();
    let mut app = app::App::new(calendar_provider, calendar_url).await;
    let new_task = task::Task::new("Helllo world".to_string());
    app.new_event(app::Message::AddTask(new_task.clone()));
    app.new_event(app::Message::AddContext(
        new_task.id,
        "Wumpa islands".to_string(),
    ));
    app.new_event(app::Message::AddContext(
        new_task.id,
        "Dragon Kingdom".to_string(),
    ));
    app.new_event(app::Message::AddContext(new_task.id, "Avalar".to_string()));
    app.new_event(app::Message::AddContext(
        new_task.id,
        "Forgotten Realms".to_string(),
    ));
    app.new_event(app::Message::AddContext(
        new_task.id,
        "Warp Room".to_string(),
    ));
    app.new_event(app::Message::RemoveContext(
        new_task.id,
        "Forgotten Realms".to_string(),
    ));
    let start_date = NaiveDateTime::parse_from_str("20221101T140000", "%Y%m%dT%H%M%S").ok();
    app.new_event(app::Message::SetStartDate(new_task.id, start_date));
    let due_date = NaiveDateTime::parse_from_str("20221112T180000", "%Y%m%dT%H%M%S").ok();
    app.new_event(app::Message::SetDueDate(new_task.id, due_date));
    app.sync().await;
    println!("\n{:#?}", app.get_present_state());
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
