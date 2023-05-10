struct User {
    id: String,
}

impl From<String> for User {
    fn from(value: String) -> Self {
        Self { id: value }
    }
}

impl Into<String> for User {
    fn into(self) -> String {
        self.id
    }
}

struct Group {
    id: String,
    user_ids: Vec<String>,
}
