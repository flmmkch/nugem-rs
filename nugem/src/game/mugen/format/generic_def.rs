use regex::bytes::{Regex, Captures, Match};
use skip_bom::SkipEncodingBom;
use std::io::{BufReader, Read};
use lazy_static::lazy_static;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum DefLine {
    KeyValue(String, String),
    Simple(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Category {
    name: String,
    lines: Vec<(u64, DefLine)>,
}

pub struct Categories<R: Read> {
    reader: BufReader<SkipEncodingBom<R>>,
    category_name: Option<String>,
    category_line_number: u64,
    line_number: u64,
}

impl<R: Read> Categories<R> {
    /// Read a *.def file and get an iterator on the categories
    pub fn read_def(input: R) -> Self {
        let reader = BufReader::new(SkipEncodingBom::new(input));
        Self {
            reader,
            category_name: None,
            line_number: 1,
            category_line_number: 1,
        }
    }
}

impl<R: Read> Iterator for Categories<R> {
    type Item = (u64, Category);

    fn next(&mut self) -> Option<Self::Item> {
        use std::io::BufRead;
        
        let mut next_category_name = None;
        let mut category_lines = Vec::new();
        let mut line = Vec::new();
        let mut previous_category_line_number = self.category_line_number;
        let mut eof = false;
        while !eof && next_category_name.is_none() {
            previous_category_line_number = self.category_line_number;
            line.clear();
            match self.reader.read_until(b'\n', &mut line) {
                Ok(0) => eof = true,
                Ok(_) => {
                    // read the next category
                    lazy_static! {
                        static ref REGEX_CATEGORY: Regex = Regex::new(r"^\s*\[\s*(.*?)\s*\]").unwrap();
                        static ref REGEX_KV_QUOTED: Regex = Regex::new(r#"^\s*([^;]+?)\s*=\s*"([^\r\n;]+?)""#).unwrap();
                        static ref REGEX_KV_UNQUOTED: Regex = Regex::new(r"^\s*([^;]+?)\s*=\s*((\s*[^\n;\s]+)*)\s*").unwrap();
                        static ref REGEX_SIMPLE: Regex = Regex::new(r"^\s*([^;\n\s]+?(?:\s*[^;\n]+)*)").unwrap();
                    }
                    if let Some(c) = REGEX_CATEGORY.captures(&line) {
                        self.category_line_number = self.line_number;
                        next_category_name = c.get(1).map(str_from_match_capture).map(str::to_owned);
                        if self.category_name.is_none() {
                            self.category_name = next_category_name.take();
                        }
                    }
                    else if let Some(c) = REGEX_KV_QUOTED.captures(&line) {
                        if let Some(l) = DefLine::key_value_from_captures(c) {
                            category_lines.push((self.line_number, l));
                        }
                    }
                    else if let Some(c) = REGEX_KV_UNQUOTED.captures(&line) {
                        if let Some(l) = DefLine::key_value_from_captures(c) {
                            category_lines.push((self.line_number, l));
                        }
                    }
                    else {
                        if let Some(c) = REGEX_SIMPLE.captures(&line) {
                            if let Some(l) = DefLine::simple_from_captures(c) {
                                category_lines.push((self.line_number, l));
                            }
                        }
                    }
                },
                Err(err) => log::error!("Failed to read line {1} from def file: {0}", err, self.line_number),
            }
            self.line_number += 1;
        }
        if next_category_name.is_some() || self.category_name.is_some() {
            let category = Category {
                name: self.category_name.take().unwrap_or(String::new()),
                lines: category_lines,
            };
            self.category_name = next_category_name;
            Some((previous_category_line_number, category))
        }
        else {
            None
        }
    }
}

impl DefLine {
    fn key_value_from_captures(captures: Captures) -> Option<DefLine> {
        let key = str_from_match_capture(captures.get(1)?).to_owned();
        let value = str_from_match_capture(captures.get(2)?).to_owned();
        Some(DefLine::KeyValue(key, value))
    }
    fn simple_from_captures(captures: Captures) -> Option<DefLine> {
        Some(DefLine::Simple(str_from_match_capture(captures.get(1)?).to_owned()))
    }
}

fn str_from_match_capture<'t>(m: Match<'t>) -> &'t str
{
    std::str::from_utf8(m.as_bytes()).unwrap_or("")
}

impl Category {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn into_lines(self) -> Vec<(u64, DefLine)> {
        self.lines
    }
    #[allow(dead_code)]
    pub fn lines(&self) -> &[(u64, DefLine)] {
        self.lines.as_slice()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn def_test() {
        use std::io::Cursor;
        let test_string = b"
            [info]
            hello = world  
            
            other_hello = \"world!\"    ; unrelated comment text

            [info2]


            number = 23; comment with \xC9 and \xE4 non-unicode characters



            test = ok this is it\t

            simple value

;

  ;         autre = yes

            world = \"hello\"

            [info -3]
";
        let categories : Vec<_> = Categories::read_def(Cursor::new(test_string)).collect();
        assert_eq!(
            &vec![
                (2, Category { name: "info".into(), lines: vec![(3, DefLine::KeyValue("hello".into(), "world".into())), (5, DefLine::KeyValue("other_hello".into(), "world!".into()))]}),
                (7, Category { name: "info2".into(), lines: vec![(10, DefLine::KeyValue("number".into(), "23".into())), (14, DefLine::KeyValue("test".into(), "ok this is it".into())), (16, DefLine::Simple("simple value".into())), (22, DefLine::KeyValue("world".into(), "hello".into()))]}),
                (24, Category { name: "info -3".into(), lines: vec![]}),
                ]
            , &categories);
    }
}
