pub struct User {
    pub nick: Option<String>,
    pub user: Option<String>,
    pub host: Option<String>,
    pub full_name: Option<String>,
}

impl User {
    pub fn new() -> Self {
        User {
            nick: None,
            user: None,
            host: None,
            full_name: None,
        }
    }
}
