use std::{collections::HashMap, sync::Mutex /* , time::SystemTime*/};

use colored::Colorize;

#[derive(Debug)]
struct Entry {
    //pub time: SystemTime,
    pub data: String,
}

impl Entry {
    pub fn new(data: String) -> Self {
        //let time = SystemTime::now();
        Self { /* time ,*/ data, }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StorageType {
    // a list indexed by element position
    Untagged,
    // a key=value store
    Tagged,
    // a single state with an optional previous state
    CurrentPrevious,
}

impl StorageType {
    pub fn as_u8(&self) -> u8 {
        match self {
            StorageType::CurrentPrevious => 0,
            StorageType::Untagged => 1,
            StorageType::Tagged => 2,
        }
    }
}

const CURRENT_TAG: &str = "__current";
const PREVIOUS_TAG: &str = "__previous";

#[derive(Debug)]
pub struct Storage {
    name: String,
    type_: StorageType,
    inner: Mutex<HashMap<String, Entry>>,
}

impl Storage {
    pub fn new(name: &str, type_: StorageType) -> Self {
        let name = name.to_string();
        let inner = Mutex::new(HashMap::new());
        Self { name, type_, inner }
    }

    pub fn get_type(&self) -> &StorageType {
        &self.type_
    }

    pub fn to_structured_string(&self) -> String {
        let inner = self.inner.lock().unwrap();

        match self.type_ {
            StorageType::Tagged => {
                let mut xml = format!("<{}>\n", &self.name);

                if inner.is_empty() {
                    xml += "  no entries yet\n";
                } else {
                    for (key, entry) in &*inner {
                        xml += &format!("  - {}: {}\n", key, &entry.data);
                    }
                }

                xml += &format!("</{}>", &self.name);

                xml.to_string()
            }
            StorageType::Untagged => {
                let mut xml = format!("<{}>\n", &self.name);

                if inner.is_empty() {
                    xml += "  no entries yet\n";
                } else {
                    for entry in inner.values() {
                        xml += &format!("  - {}\n", &entry.data);
                    }
                }

                xml += &format!("</{}>", &self.name);

                xml.to_string()
            }
            StorageType::CurrentPrevious => {
                if let Some(current) = inner.get(CURRENT_TAG) {
                    let mut str = format!("* Current {}: {}", &self.name, current.data.trim());
                    if let Some(prev) = inner.get(PREVIOUS_TAG) {
                        str += &format!("\n* Previous {}: {}", &self.name, prev.data.trim());
                    }
                    str
                } else {
                    "".to_string()
                }
            }
        }
    }

    pub fn add_tagged(&self, key: &str, data: &str) {
        assert!(matches!(self.type_, StorageType::Tagged));
        println!("<{}> {}={}", self.name.bold(), key, data.yellow());
        self.inner
            .lock()
            .unwrap()
            .insert(key.to_string(), Entry::new(data.to_string()));
    }

    pub fn del_tagged(&self, key: &str) -> Option<String> {
        assert!(matches!(self.type_, StorageType::Tagged));
        if let Some(old) = self.inner.lock().unwrap().remove(key) {
            println!("<{}> {} removed\n", self.name.bold(), key);
            Some(old.data)
        } else {
            None
        }
    }

    pub fn add_untagged(&self, data: &str) {
        assert!(matches!(self.type_, StorageType::Untagged));
        println!("<{}> {}", self.name.bold(), data.yellow());

        let mut inner = self.inner.lock().unwrap();

        let tag = format!("{}", inner.len() + 1);
        inner.insert(tag, Entry::new(data.to_string()));
    }

    pub fn del_untagged(&self, pos: usize) -> Option<String> {
        assert!(matches!(self.type_, StorageType::Untagged));
        let tag = format!("{}", pos);
        if let Some(old) = self.inner.lock().unwrap().remove(&tag) {
            println!("<{}> element {} removed\n", self.name.bold(), pos);
            Some(old.data)
        } else {
            None
        }
    }

    pub fn set_current(&self, data: &str, verbose: bool) {
        assert!(matches!(self.type_, StorageType::CurrentPrevious));
        let mut inner = self.inner.lock().unwrap();

        if verbose {
            println!("<{}> current={}", self.name.bold(), data.yellow());
        }

        let old_current = inner.remove(CURRENT_TAG);

        inner.insert(CURRENT_TAG.to_string(), Entry::new(data.to_string()));
        if let Some(old_curr) = old_current {
            inner.insert(PREVIOUS_TAG.to_string(), old_curr);
        }
    }

    pub fn clear(&self) {
        self.inner.lock().unwrap().clear();
        println!("<{}> cleared", self.name.bold());
    }
}
