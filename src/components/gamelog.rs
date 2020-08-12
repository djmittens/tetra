pub struct GameLog {
    pub entries: Vec<String>,
}

impl GameLog {
    pub fn say(&mut self, str: String) {
        self.entries.push(str);
    }
}