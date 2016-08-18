#![feature(plugin)]
#![plugin(clippy)]

#![feature(question_mark)]

extern crate iron;
extern crate hyper;
extern crate rustc_serialize;
pub mod processor;
pub mod errors;

use std::fs::File;
use std::path::Path;
use std::io::{BufReader, BufRead};
use std::sync::mpsc::channel;
use std::thread;
use std::sync::Arc;

use iron::prelude::*;
use iron::status;

use rustc_serialize::json;

use processor::word_counter::WordCounter;
use processor::collapser::Collapser;
use errors::ProcessFileError;

use hyper::header::{AccessControlAllowOrigin, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

fn main() {
    const TEXT_FOLDER: &'static str = "./texts";

    let noise_words = Arc::new(load_noise_words().unwrap());
    let separators = Arc::new(load_separators().unwrap());
    let collapser = Arc::new(Collapser::from_file(std::path::Path::new("collapse.csv")).unwrap());

    println!("Opening listening socket on tcp:3005");

    Iron::new(move |req: &mut Request| {
            println!("req == {:?}", req.url.path());

            if req.url.path().is_empty() {
                return Ok(Response::with((status::NotFound, "not found")));
            }

            let requested_file_name = req.url.path()[0];

            let mut texts = std::fs::read_dir(TEXT_FOLDER).unwrap();
            let found = texts.any(|elem| {
                let name = elem.unwrap().file_name().into_string().unwrap();
                name == requested_file_name
            });

            if found {
                let file = Path::new(TEXT_FOLDER);
                let file = file.join(requested_file_name);

                let wc = process_file(noise_words.clone(),
                                      separators.clone(),
                                      collapser.clone(),
                                      file.as_path())
                    .unwrap();

                let mut res: Vec<(&String, &u64)> = wc.hm.iter().collect();
                res.sort_by(|a, b| b.1.cmp(a.1));

                res = res.into_iter().take(100).collect();

                let encoded = json::encode(&res).unwrap();

                for (k, v) in res.into_iter().take(30) {
                    println!("{} - {}", k, v);
                }

                let mut resp = Response::with((status::Ok, encoded));

                resp.headers.set(AccessControlAllowOrigin::Any);
                resp.headers.set(ContentType(Mime(TopLevel::Application,
                                                  SubLevel::Json,
                                                  vec![(Attr::Charset, Value::Utf8)])));

                Ok(resp)
            } else {
                Ok(Response::with((status::NotFound, "not found")))
            }
        })
        .http("localhost:3005")
        .unwrap();
}

fn process_file(noise_words: Arc<Vec<String>>,
                separators: Arc<Vec<char>>,
                collapser: Arc<Collapser>,
                file_name: &Path)
                -> Result<WordCounter, ProcessFileError> {
    const THREADS: usize = 8;

    let mut send_row_channels = Vec::new();
    let mut relinquish_wc_channels = Vec::new();

    for _ in 0..THREADS {
        let separators = separators.clone();
        let noise_words = noise_words.clone();
        let collapser = collapser.clone();

        let (tx_row, rx_row) = channel::<String>();
        send_row_channels.push(tx_row);

        let (tx_relinquish_wc, rx_relinquish_wc) = channel();
        relinquish_wc_channels.push(rx_relinquish_wc);

        thread::spawn(move || {
            let mut wc = WordCounter::new(separators, noise_words, collapser);
            while let Ok(ref row) = rx_row.recv() {
                wc.process_line(row);
            }

            // println!("processed {} lines", wc.rows_processed);
            match tx_relinquish_wc.send(wc) {
                Ok(_) => Ok(()),
                Err(e) => Err(ProcessFileError::RelinquishError(e)),
            }
        });
    }

    for _ in 0..1 {
        let file = File::open(file_name)?;
        let mut file = BufReader::new(file);

        let s = &mut String::new();
        let mut i: usize = 0;

        while file.read_line(s)? > 0 {
            send_row_channels[i % THREADS].send(s.to_string())?;
            i += 1;
            s.clear();
        }
    }

    // close send row channels
    // to do this all we have to do
    // is take ownership and drop it.
    for sc in send_row_channels {
        drop(sc);
    }

    // relinquish completed WordCounters and sum (final reduce)
    let mut wc_final = WordCounter::new(separators, noise_words, collapser);

    for ref rx in relinquish_wc_channels {
        let wc = rx.recv()?;
        // println!("received {:?}", wc.rows_processed);

        for (key, val) in wc.hm {
            wc_final.register_word(key, val);
        }
    }

    Ok(wc_final)
}

fn load_separators() -> Result<Vec<char>, errors::LoadFromFileError> {
    let file = File::open("./separators.txt")?;
    let mut file = BufReader::new(file);

    let mut s = String::new();
    let mut v: Vec<char> = Vec::new();

    while file.read_line(&mut s)? > 0 {
        match s.chars().next() {
            Some(c) => v.push(c),
            None => {
                return Err(errors::LoadFromFileError::FormatError("Char not found".to_string()))
            }
        };
        s.clear();
    }

    v.sort();
    Ok(v)
}

fn load_noise_words() -> Result<Vec<String>, std::io::Error> {
    let file = File::open("./noise_words.txt")?;
    let mut file = BufReader::new(file);

    let mut v = Vec::new();
    let mut s = String::new();

    while file.read_line(&mut s)? > 0 {
        let s_in = s.split('\n').collect();
        v.push(s_in);
        s.clear();
    }

    v.sort();
    Ok(v)
}
