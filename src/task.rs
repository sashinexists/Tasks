use chrono::{DateTime, FixedOffset, Utc};
use kitchen_fridge::Item;
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

#[derive(Debug)]
pub struct Task {
    id: Uuid,
    creation_date: DateTime<Utc>,
    last_modified: DateTime<Utc>,
    name: String,
    completed: CompletionStatus,
    start_date: Option<DateTime<FixedOffset>>,
    due: Option<DateTime<FixedOffset>>,
    contexts: Vec<String>,
    areas: Vec<String>,
    projects: Vec<String>,
    money_needed: bool,
    time_of_day: Option<TimeOfDay>,
    weather: Option<Weather>,
    parent_task: Option<Uuid>,
}

impl Task {
    pub fn from_item(item: Item) -> Task {
        let id = Uuid::parse_str(item.uid())
            .expect(format!("{} is an Invalid UUID", item.uid()).as_str());
        let creation_date = item
            .creation_date()
            .expect("Item has no creation date")
            .clone();
        let last_modified = item.last_modified().clone();
        let name = item.name().to_string();
        let completed =
            CompletionStatus::from_kitchen_fridge(item.unwrap_task().completion_status());
        let start_date = item.get_date_from_item_attribute("DT_START");
        let due = item.get_date_from_item_attribute("DUE");
        let categories = item
            .unwrap_task()
            .extra_parameters()
            .iter()
            .filter(|x| x.name == "CATEGORIES");
        let contexts = categories
            .clone()
            .map(|x| x.value.as_ref().unwrap().clone())
            .filter(|val| val.starts_with("CONTEXT "))
            .collect();
        let areas = categories
            .clone()
            .map(|x| x.value.as_ref().unwrap().clone())
            .filter(|val| val.starts_with("AREA "))
            .collect();
        let projects = categories
            .clone()
            .map(|x| x.value.as_ref().unwrap().clone())
            .filter(|val| val.starts_with("PROJECT "))
            .collect();
        let money_needed = categories
            .clone()
            .map(|x| x.value.as_ref().unwrap().clone())
            .filter(|val| val == "Money Needed")
            .collect::<Vec<String>>()
            .len()
            > 0;
        let time_of_day: Option<TimeOfDay> = categories
            .clone()
            .map(|x| x.value.as_ref().unwrap().clone())
            .find(|val| val.starts_with("TIMEOFDAY "))
            .map(|val| {
                TimeOfDay::from_str(&val).expect(&format!("{} is a bad time of day value", &val))
            });
        let weather: Option<Weather> = categories
            .clone()
            .map(|x| x.value.as_ref().unwrap().clone())
            .find(|val| val.starts_with("WEATHER "))
            .map(|val| Weather::from_str(&val).expect("bad time of day"));
        let parent_task = item.get_parent_uuid();
        Task {
            id,
            creation_date,
            last_modified,
            name,
            completed,
            start_date,
            due,
            contexts,
            areas,
            projects,
            money_needed,
            time_of_day,
            weather,
            parent_task,
        }
    }
}
#[derive(Debug, EnumString)]
enum CompletionStatus {
    Incomplete,
    Completed(Option<DateTime<Utc>>),
}

impl CompletionStatus {
    pub fn from_kitchen_fridge(
        kitchen_fridge_completion_status: &kitchen_fridge::task::CompletionStatus,
    ) -> Self {
        match kitchen_fridge_completion_status {
            kitchen_fridge::task::CompletionStatus::Completed(data) => {
                Self::Completed(data.clone())
            }
            Incomplete => Self::Incomplete,
        }
    }
}

#[derive(Debug, PartialEq, EnumString)]
enum TimeOfDay {
    #[strum(
        serialize = "TIMEOFDAY  Morning",
        serialize = "Morning",
        serialize = "TIMEOFDAY Morning"
    )]
    Morning,
    Midday,
    Afternoon,
    Evening,
    Specific(DateTime<Utc>),
}
#[derive(Debug, PartialEq, EnumString)]
enum Weather {
    #[strum(
        serialize = "WEATHER  Sunny",
        serialize = "WEATHER Sunny",
        serialize = "Sunny"
    )]
    Sunny,
    Cloudy,
    Rainy,
    Windy,
}

enum Context {
    Laptop,
    Home,
    Errands,
    SocialMedia,
    Phone,
    Kindle,
    Youtube,
}

trait TaskItem {
    fn get_attribute_from_item(&self, attribute_name: &str) -> Option<String>;
    fn get_date_from_item_attribute(&self, attribute_name: &str) -> Option<DateTime<FixedOffset>>;
    fn get_parent_uuid(&self) -> Option<Uuid>;
    fn get_tags(&self) -> Vec<String>;
}
impl TaskItem for Item {
    fn get_attribute_from_item(&self, attribute_name: &str) -> Option<String> {
        self.unwrap_task()
            .extra_parameters()
            .iter()
            .find(|x| x.name == attribute_name)?
            .value
            .clone()
    }

    fn get_date_from_item_attribute(&self, attribute_name: &str) -> Option<DateTime<FixedOffset>> {
        DateTime::parse_from_rfc3339(&self.get_attribute_from_item(attribute_name)?).ok()
    }

    fn get_parent_uuid(&self) -> Option<Uuid> {
        let related = &self.get_attribute_from_item("RELATED_TO")?;
        Uuid::parse_str(&related).ok()
    }

    fn get_tags(&self) -> Vec<String> {
        let categories = self
            .unwrap_task()
            .extra_parameters()
            .iter()
            .filter(|x| x.name == "CATEGORIES")
            .collect();
    }
}
