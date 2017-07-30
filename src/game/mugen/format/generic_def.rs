use regex::{Regex, Captures};
use std::io::{BufRead, BufReader, Lines};
use std::vec;

pub struct GenericDef {
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum DefLine {
    KeyValue(String, String),
    Simple(String),
}

#[derive(Clone, Debug)]
pub struct Category {
    name: String,
    lines: Vec<DefLine>,
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
        let mut category_has_name = false;
        let mut next_category = false;
        let mut name = String::new();
        let mut lines = Vec::new();
        while !(self.file_ended || next_category) {
            match self.lines.next() {
                Some(Ok(line)) => {
                    // read the next category
                    lazy_static! {
                        static ref CATEGORY_REGEX: Regex = Regex::new("^[ \t]*\\[[ \t]*([^\\]]+?)[ \t]*\\][ \t\r]*(?:;.*)?$").unwrap();
                        static ref KV_QUOTED_REGEX: Regex = Regex::new("^[ \t]*([^=]+?)[ \t]*=[ \t]*\"([^\r\"]+?)\"[ \t\r]*(?:;.*)?$").unwrap();
                        static ref KV_REGEX: Regex = Regex::new("^[ \t]*([^=]+?)[ \t]*=[ \t]*([^\r]+?)?[ \t\r]*(?:;.*)?$").unwrap();
                        static ref SIMPLE_REGEX: Regex = Regex::new("^[ \t]*(([^ \r;]+[ \r]?)+?)[ \t\r]*(?:;.*)?$").unwrap();
                    }
                    if let Some(c) = CATEGORY_REGEX.captures(&line) {
                        fn captures_to_category_name(captures: Captures) -> String {
                            captures.get(1).map_or("", |m| m.as_str()).to_owned()
                        };
                        if self.first_category && (&name == "") {
                            empty_category = false;
                            name = captures_to_category_name(c);
                            category_has_name = true;
                        }
                        else {
                            next_category = true;
                            if !category_has_name {
                                name = self.next_name.to_owned();
                                category_has_name = true;
                            }                            
                            self.next_name = captures_to_category_name(c);
                            empty_category = false;
                        }
                    }
                    else {
                        if let Some(c) = KV_QUOTED_REGEX.captures(&line) {
                            if let Some(l) = DefLine::from_captures(c, true) {
                                lines.push(l);
                                empty_category = false;
                            }
                        }
                        else {
                            if let Some(c) = KV_REGEX.captures(&line) {
                                if let Some(l) = DefLine::from_captures(c, true) {
                                    lines.push(l);
                                    empty_category = false;
                                }
                            }
                            else {
                                if let Some(c) = SIMPLE_REGEX.captures(&line) {
                                    if let Some(l) = DefLine::from_captures(c, false) {
                                        lines.push(l);
                                        empty_category = false;
                                    }
                                }
                            }
                        }
                    }
                },
                Some(Err(_)) => continue,
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
                lines,
            };
            self.first_category = false;
            Some(category)
        }
    }
}

impl DefLine {
    fn from_captures(captures: Captures, key_value: bool) -> Option<DefLine> {
        if captures.len() >= 2 {
            if key_value {
                let key = captures.get(1).map_or("", |m| m.as_str()).to_owned();
                let value = captures.get(2).map_or("", |m| m.as_str()).to_owned();
                Some(DefLine::KeyValue(key, value))
            }
            else {
                Some(DefLine::Simple(captures.get(1).map_or("", |m| m.as_str()).to_owned()))
            }
        }
        else {
            None
        }
    }
}

impl Category {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn lines(self) -> vec::IntoIter<DefLine> {
        self.lines.into_iter()
    }
    #[allow(dead_code)]
    pub fn lines_ref(&self) -> &[DefLine] {
        &self.lines[..]
    }
}

impl GenericDef {
    /// Read a *.def file and get an iterator on the categories
    pub fn read<T: BufRead>(input: T) -> Categories<BufReader<T>> {
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



            test = ok

;

            world = "hello"

            [info3]
            "#;
        let categories : Vec<_> = GenericDef::read(Cursor::new(test_string)).collect();
        assert_eq!(categories.len(), 3);
        assert_eq!(categories[0].name(), "info");
        println!("{:?}", categories[0].lines_ref());
        assert_eq!(categories[0].lines_ref().len(), 2);
        assert_eq!(categories[1].name(), "info2");
        assert_eq!(categories[1].lines_ref().len(), 3);
    }
}
