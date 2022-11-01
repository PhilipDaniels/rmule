use std::borrow::Borrow;
use std::hash::Hash;

use case_insensitive_hashmap::CaseInsensitiveHashMap;

pub struct IniFile {
    lines: Vec<String>,
    sections: CaseInsensitiveHashMap<IniFileSection>
}

pub struct IniFileSection {
    name: String,
    entries: CaseInsensitiveHashMap<IniFileEntry>
}

pub struct IniFileEntry {
    value: String,
    line_index: usize
}

impl IniFileSection {
    fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            entries: CaseInsensitiveHashMap::new()
        }
    }

    // pub fn keys(&self) -> std::collections::hash_map::Keys<String, String> {
    //     self.entries.keys()
    // }

    // pub fn values(&self) -> std::collections::hash_map::Values<String, String> {
    //     self.entries.values()
    // }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains_key(&self, key: &str) -> bool
    {
        self.entries.contains_key(key)
    }

    pub fn get(&self, key: &str) -> Option<&IniFileEntry>
    where
    {
        self.entries.get(key)
    }

    
}

impl IniFileEntry {
    fn new<S: Into<String>>(value: S, line_index: usize) -> Self {
        Self {
            value: value.into(),
            line_index
        }
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }
}

impl IniFile {
    /// The section that keys are put into if there is no
    /// open section.
    pub const ROOT_SECTION: &str = "";

    pub fn new<T>(lines: T) -> Self
    where T : IntoIterator<Item = String>
    {
        let lines = lines.into_iter().collect::<Vec<_>>();
        let sections = Self::parse_lines(&lines);
        IniFile {
            lines,
            sections,
        }
    }

    pub fn get_section(&self, section: &str) -> Option<&IniFileSection> {
        self.sections.get(section)
    }

    /// Gets a value in a section.
    /// If the key is not found, the empty string is returned.
    pub fn get_entry_in_section(&self, section: &str, key: &str) -> Option<&IniFileEntry> {
        self.get_section(section).and_then(|s| s.get(key))
    }

    /// Gets a value in a section, or the default if not found.
    pub fn get_value_or_default<'a, 'b, 'c, 'd: 'a>(
        &'a self,
        section: &'b str,
        key: &'c str,
        default: &'d str,
    ) -> &'a str {
        match self.sections.get(section) {
            Some(section) => match section.get(key) {
                Some(value) => &value.value,
                None => default,
            },
            None => default,
        }
    }
    
    fn parse_lines(lines: &[String]) -> CaseInsensitiveHashMap<IniFileSection>{
        let mut sections = CaseInsensitiveHashMap::new();
        let mut current_section = IniFileSection::new(Self::ROOT_SECTION);

        for line_idx in 0..lines.len() {
            let line = &lines[line_idx];
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            // A section name, means we are starting a new section.
            if line.starts_with('[') && line.ends_with(']') {
                let section_name = line[1..line.len() - 1].trim();
                if current_section.name != Self::ROOT_SECTION || current_section.len() > 0 {
                    sections.insert(current_section.name.clone(), current_section);
                }
                current_section = IniFileSection::new(section_name);
                continue;
            }

            match line.find('=') {
                Some(equals_index) => {
                    let key = line[0..equals_index].trim();
                    let value = line[equals_index + 1..].trim();
                    let ini_entry = IniFileEntry::new(value, line_idx);
                    current_section.entries.insert(key, ini_entry);
                }
                None => {
                    // A key with no value. Just store the key.
                    // Not sure this ever happens with Mule.
                    let ini_entry = IniFileEntry::new("", line_idx);
                    current_section.entries.insert(line.to_string(), ini_entry);
                }
            }
        }

        sections.insert(current_section.name.clone(), current_section);

        sections
    }
}
