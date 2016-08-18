use std::collections::HashMap;
use std::path::Path;
use std::io::{BufReader, BufWriter, BufRead};
use std::fs::File;
use std::io::Write;
use std::io::Error;

use errors::LoadFromFileError;

#[derive(Debug, Default)]
pub struct Collapser {
    std_words: Vec<String>,
    map: HashMap<String, usize>,
}

impl Collapser {
    pub fn new() -> Collapser {
        Collapser {
            std_words: Vec::new(),
            map: HashMap::new(),
        }
    }

    pub fn from_file(p: &Path) -> Result<Collapser, LoadFromFileError> {
        let mut c = Collapser::new();

        let file = File::open(p)?;
        let mut br = BufReader::new(file);

        let mut s = String::new();
        while br.read_line(&mut s)? > 0 {
            {
                let mut tokens = s.split(';');

                let word = match tokens.next() {
                    Some(w) => w.trim(),
                    None => {
                        return Err(LoadFromFileError::FormatError("First word not found"
                            .to_string()))
                    }
                };
                let syn = match tokens.next() {
                    Some(w) => w.trim(),
                    None => {
                        return Err(LoadFromFileError::FormatError("Second word not found"
                            .to_string()))
                    }
                };

                // println!("word == {:?}, syn == {:?}", word, syn);
                c.add(syn, word);
            }

            s.clear();
        }

        Ok(c)
    }

    pub fn add(&mut self, word: &str, synonim: &str) {
        let pos = self.std_words.iter().position(|item| item == word);

        let pos = match pos {
            None => {
                self.std_words.push(word.to_owned());
                self.std_words.len() - 1
            }
            Some(val) => val,
        };

        self.map.insert(synonim.to_owned(), pos);
    }

    pub fn collapse(&self, s: &str) -> Option<&str> {
        match self.map.get(s) {
            None => None,
            Some(idx) => Some(&self.std_words[*idx]),
        }
    }

    pub fn save_to_file(&self, file: &Path) -> Result<(), Error> {
        let file = File::create(file)?;
        let mut bw = BufWriter::new(file);

        for (word, syn) in &self.map {
            // println!("word = {:?}, syn = {:?}", word, syn);
            let mut s: String = String::new();
            s.push_str(word);
            s.push(';');
            s.push_str(&self.std_words[*syn]);

            s.push('\n');

            bw.write_all(s.as_bytes())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Collapser;

    #[test]
    fn add1() {
        let mut c = Collapser::new();

        c.add("cosa", "cose");
        c.add("cosa", "cosetta");
        c.add("cosa", "cosette");

        c.add("pesce", "pesci");

        assert_eq!(c.collapse("cose"), Some("cosa"));
        assert_eq!(c.collapse("cosetta"), Some("cosa"));
        assert_eq!(c.collapse("cosette"), Some("cosa"));

        assert_eq!(c.collapse("pesci"), Some("pesce"));

        assert_eq!(c.collapse("cosette"), c.collapse("cose"));

        assert_eq!(c.collapse("cane"), None);
    }
}
