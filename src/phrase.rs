use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use rustc_serialize::json;

#[derive(Clone, Default, PartialEq, RustcDecodable, RustcEncodable)]
pub struct PhraseConfig {
    pub phrases: Option<Vec<String>>,
}

/// Config file for echo phrases (Borrowed from
/// https://github.com/aatxe/irc/blob/master/src/client/data/config.rs)
impl PhraseConfig {
    /// Loads a JSON configuration from the desired path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<PhraseConfig> {
        let mut file = try!(File::open(path));
        let mut data = String::new();
        try!(file.read_to_string(&mut data));
        json::decode(&data[..]).map_err(|_|
            Error::new(ErrorKind::InvalidInput, "Failed to decode configuration file.")
        )
    }

    /// Saves a JSON configuration to the desired path.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = try!(File::create(path));
        file.write_all(try!(json::encode(self).map_err(|_|
            Error::new(ErrorKind::InvalidInput, "Failed to encode configuration file.")
        )).as_bytes())
    }

    /// Check if the phrase exists in the phrases list
    pub fn check(&self, phrase: &String) -> (bool,Option<&String>) {
        for x in self.phrases.as_ref().unwrap() {
            if x.eq(phrase) {
                return (true,Some(x));
            }
        }
        return (false,None);
    }
}

#[cfg(test)]
mod test {
    use super::PhraseConfig;

    #[test]
    fn load() {
        let cfg = PhraseConfig {
            phrases: Some(vec![format!("test")]),
        };
        assert!(cfg.check(&String::from("test")).0);
        assert_eq!(cfg.check(&String::from("test")).1.unwrap(),"test");
        assert!(!cfg.check(&String::from("test2")).0);
    }
}
