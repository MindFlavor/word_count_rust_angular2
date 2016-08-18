use std::collections::hash_map::HashMap;
use std::sync::Arc;
use processor::collapser::Collapser;

pub struct WordCounter {
    pub hm: HashMap<String, u64>,
    separators: Arc<Vec<char>>,
    noise_words: Arc<Vec<String>>,
    collapser: Arc<Collapser>,
    pub rows_processed: u64,
}

impl WordCounter {
    pub fn new(separators: Arc<Vec<char>>,
               noise_words: Arc<Vec<String>>,
               collapser: Arc<Collapser>)
               -> WordCounter {
        WordCounter {
            hm: HashMap::new(),
            separators: separators,
            noise_words: noise_words,
            rows_processed: 0u64,
            collapser: collapser,
        }
    }

    pub fn process_line(&mut self, s: &str) {
        // println!("process_line({:?})", s);

        // count row as processed
        self.rows_processed += 1u64;

        let tokens = self.split(s);

        for mut to in tokens {
            // ignore leading /t
            if to.starts_with('\t') || to.starts_with('\r') || to.starts_with('\n') {
                to = &to[1..];
            }

            // ignore \r\n
            if to.ends_with("\r\n") {
                to = &to[..(to.len() - 2)];
            }

            // ignore empty words or single char words
            if to.len() < 2 {
                continue;
            }

            // ignore numbers
            if to.chars().all(|c| c.is_digit(10)) {
                continue;
            }

            let toc = to.to_lowercase();

            // ignore noise words
            let found = match self.noise_words.binary_search(&toc) {
                Ok(_) => true,
                _ => false,
            };

            if !found {
                // collapse synonims
                let toc = match self.collapser.collapse(&toc) {
                    Some(s) => s.to_owned(),
                    None => toc,
                };

                self.register_word(toc, 1u64);
            }
        }
    }

    fn split<'b>(&mut self, s: &'b str) -> Vec<&'b str> {
        s.split(|c| self.separators.iter().any(|i| *i == c)).collect()
    }

    pub fn register_word(&mut self, word: String, count: u64) {
        let value = match self.hm.get(&word) {
            None => 0u64,
            Some(num) => *num,
        };

        self.hm.insert(word, value + count);
    }
}
