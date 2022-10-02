pub enum Context {
    Laptop,
    Phone,
    Kindle,
    Errands,
    Home,
    SocialMedia,
    YouTube
}

pub enum Property {
    Context(Context),
    Time,
    Status,
    Area,
    Project,
    Weather
}
