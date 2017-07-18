use regex::{ Regex, Captures };
use std::io::{ Read, BufRead, BufReader, Lines };

pub struct GenericDef {
}

pub struct Category {
    name: String,
    key_values: Vec<KeyValue>,
}

pub struct KeyValue {
    key: String,
    value: String,
}

pub struct Categories<T: BufRead> {
    lines: Lines<T>,
    file_ended: bool,
    next_name: String,
    first_category: bool,
}

impl<T: BufRead> Iterator for Categories<T> {
    type Item = Category;

    fn next(&mut self) -> Option<Category> {
        let mut empty_category = true;
        let mut next_category = false;
        let mut name = String::new();
        let mut key_values = Vec::new();
        while !(self.file_ended || next_category) {
            match self.lines.next() {
                Some(Ok(line)) => {
                    // read the next category
                    lazy_static! {
                        static ref CATEGORY_REGEX: Regex = Regex::new("^[ \t]*\\[[ \t]*([^\\]]+?)[ \t]*\\][ \t\r]*(?:;.*)?$").unwrap();
                        static ref KV_QUOTED_REGEX: Regex = Regex::new("^[ \t]*([^=]+?)[ \t]*=[ \t]*\"([^\r\"]+?)\"[ \t\r]*(?:;.*)?$").unwrap();
                        static ref KV_REGEX: Regex = Regex::new("^[ \t]*([^=]+?)[ \t]*=[ \t]*([^\r]+?)[ \t\r]*(?:;.*)?$").unwrap();
                    }
                    if let Some(c) = CATEGORY_REGEX.captures(&line) {
                        fn captures_to_category_name(captures: Captures) -> String {
                            captures.get(1).map_or("", |m| m.as_str()).to_owned()
                        };
                        if self.first_category {
                            self.first_category = false;
                            empty_category = false;
                            name = captures_to_category_name(c);
                        }
                        else {
                            if empty_category {
                                empty_category = false;
                                name = self.next_name.to_owned();
                            }
                            else {
                                self.next_name = captures_to_category_name(c);
                                next_category = true;
                            }
                        }
                    }
                    else {
                        fn captures_to_kv(captures: Captures) -> KeyValue {
                            let key = captures.get(1).map_or("", |m| m.as_str()).to_owned();
                            let value = captures.get(2).map_or("", |m| m.as_str()).to_owned();
                            KeyValue {
                                key,
                                value,
                            }
                        };
                        if let Some(c) = KV_QUOTED_REGEX.captures(&line) {
                            key_values.push(captures_to_kv(c));
                            empty_category = false;
                        }
                        else {
                            if let Some(c) = KV_REGEX.captures(&line) {
                                key_values.push(captures_to_kv(c));
                                empty_category = false;
                            }
                        }
                    }
                },
                Some(Err(_)) => return self.next(),
                None => {
                    if !self.first_category {
                        empty_category = false;
                        name = self.next_name.to_owned();
                    }
                    self.file_ended = true;
                }
            }
        }
        if empty_category {
            if self.file_ended {
                None
            }
            else {
                self.next()
            }
        }
        else {
            let category = Category {
            name,
            key_values
        };
            Some(category)
        }
    }
}

impl Category {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn key_values(&self) -> &[KeyValue] {
        &self.key_values[..]
    }
}

impl KeyValue {
    pub fn key(&self) -> &str {
        self.key.as_str()
    }
    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

impl GenericDef {
    /// Read a *.def file and get an iterator on the categories
    pub fn read<T: Read>(input: T) -> Categories<BufReader<T>> {
        use std::io::BufRead;
        let buffer = BufReader::new(input);
        Categories {
            lines: buffer.lines(),
            file_ended: false,
            next_name: String::new(),
            first_category: true,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn def_test() {
        use std::io::Cursor;
        let test_string = r#"
            [info]
            hello = world
            
            other_hello = "world!"    ; unrelated comment text

            [info2]
            number = 23
            "#;
        let categories : Vec<_> = GenericDef::read(Cursor::new(test_string)).collect();
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].name(), "info");
        assert_eq!(categories[0].key_values().len(), 2);
        assert_eq!(categories[1].name(), "info2");
        assert_eq!(categories[1].key_values().len(), 1);
    }
}
