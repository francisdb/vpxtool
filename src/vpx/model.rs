#[derive(Debug, PartialEq)]
pub enum StringEncoding {
    Latin1,
    Utf8,
}

/// Because we want to have a exact copy after reading/writing a vpx file we need to
/// keep old latin1 encoding if we read that from a file.
#[derive(Debug, PartialEq)]
pub struct StringWithEncoding {
    pub encoding: StringEncoding,
    pub string: String,
}
impl StringWithEncoding {
    pub fn new(string: String) -> StringWithEncoding {
        StringWithEncoding {
            encoding: StringEncoding::Utf8,
            string,
        }
    }

    pub fn from(s: &str) -> StringWithEncoding {
        StringWithEncoding {
            encoding: StringEncoding::Utf8,
            string: s.to_owned(),
        }
    }

    pub fn empty() -> StringWithEncoding {
        StringWithEncoding {
            encoding: StringEncoding::Utf8,
            string: String::new(),
        }
    }
}
