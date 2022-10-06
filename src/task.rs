use chrono::{DateTime, FixedOffset, Utc};
use ical::property::Property;
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
        let contexts = item.get_attribute_from_tag("CONTEXT ");
        let areas = item.get_attribute_from_tag("AREA ");
        let projects = item.get_attribute_from_tag("PROJECT ");
        let money_needed = item.get_attribute_from_tag("MONEYNEEDED ").len() > 0;
        let time_of_day: Option<TimeOfDay> = {
            let time_of_day_tags = &item.get_attribute_from_tag("TIMEOFDAY ");
            if time_of_day_tags.len() > 0 {
                TimeOfDay::from_str(&time_of_day_tags[0]).ok()
            } else {
                None
            }
        };
        let weather: Option<Weather> = {
            let weather_tags = &item.get_attribute_from_tag("WEATHER ");
            if weather_tags.len() > 0 {
                Weather::from_str(&weather_tags[0]).ok()
            } else {
                None
            }
        };
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
pub enum TimeOfDay {
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
pub enum Weather {
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
    fn get_attribute_from_tag(&self, tag: &str) -> Vec<String>;
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
        let categories_string: Option<String> = match self
            .unwrap_task()
            .extra_parameters()
            .into_iter()
            .filter(|x| x.name == "CATEGORIES")
            .collect::<Vec<&Property>>()
            .first()
        {
            Some(categories) => categories.value.clone(),
            None => None,
        };
        match categories_string {
            Some(categories_string) => {
                let strs: Vec<&str> = categories_string.split(',').collect();
                strs.into_iter().map(|s| s.to_string()).collect()
            }
            None => Vec::new(),
        }
    }

    fn get_attribute_from_tag(&self, tag: &str) -> Vec<String> {
        self.get_tags()
            .iter()
            .filter(|t| t.starts_with(tag))
            .flat_map(|t| t.split("  ").last())
            .map(|t| t.to_string())
            .collect()
    }
}
