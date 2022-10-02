use std::path::Path;

use dotenvy::dotenv;
use kitchen_fridge::{
    traits::{CalDavSource, DavCalendar},
    *,
};
mod task;
mod utils;
use url::Url;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let mut calendar_provider = get_calendar().await;
    let calendar_url =
        Url::parse("https://sashin.online/remote.php/dav/calendars/sashin/rust-playground/")
            .unwrap();
    let remote_calendar = calendar_provider
        .remote()
        .get_calendar(&calendar_url)
        .await
        .unwrap();
    let remote_calendar = &*remote_calendar.lock().unwrap();
    let item_urls = remote_calendar.get_item_urls().await.unwrap();
    let vec_urls: Vec<Url> = item_urls.into_iter().collect();
    let items = remote_calendar.get_items_by_url(&vec_urls).await.unwrap();
    calendar_provider.sync().await;
    let items = get_tasks_from_items(items.into_iter().flatten().collect()).await;
    println!("{:#?}", items);
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

async fn get_tasks_from_items(items: Vec<Item>) -> Vec<task::Task> {
    let mut tasks: Vec<task::Task> = Vec::new();
    for item in items {
        let task = task::Task::from_item(item);
        tasks.push(task);
    }
    tasks
}
