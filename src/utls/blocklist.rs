use std::fs;

use serde::*;

#[derive(Serialize, Deserialize, Default)]
pub struct Blocklist {
    list : Vec<String>
}

impl Blocklist {
    pub fn new() -> Blocklist {
        let path = std::path::Path::new("blocklist.json");
        if !path.exists() {
            return Blocklist::create_blocklist();
        }

        let json = fs::read_to_string(path)
            .expect("Unable to read blocklist.json");

        let list : Blocklist = serde_json::from_str(&json)
            .expect("Unable to deserialize blocklist.json");
        list
    }

    pub fn contains(&self, snowflake : u64) -> bool {
        self.list.contains(&snowflake.to_string())
    }

    pub fn block(&mut self, snowflake : u64) {
        let snowflake = snowflake.to_string();
        self.list.push(snowflake);
        self.write();
    }

    pub fn unblock(&mut self, snowflake : u64) {
        let snowflake = snowflake.to_string();
        self.list.retain(|x| *x != snowflake);
        self.write();
    }

    pub fn write(&self) {
        let json = serde_json::to_string(self)
            .expect("Unable to serialize blocklist.json");

        fs::write("blocklist.json", json)
            .expect("Unable to create blocklist.json!");
    }

    fn create_blocklist() -> Blocklist {
        let list = Blocklist {
            list : Default::default()
        };
        list.write();
        list
    }
}

