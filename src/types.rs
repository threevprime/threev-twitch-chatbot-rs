#[derive(Debug, Clone)]
pub struct User {
    pub user_id: String,
    pub first: i64,
    pub second: i64,
    pub third: i64,
    pub daily: i64,
}

#[derive(Debug)]
pub struct Command {
    name: String,
    alias: Option<String>,
    response: String,
}

impl Command {
    pub fn new(name: String, alias: Option<String>, response: String) -> Self {
        Self {
            name: name,
            alias: alias,
            response: response,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    pub fn response(&self) -> &str {
        &self.response
    }
}
