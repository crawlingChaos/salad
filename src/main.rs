use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    result::Result,
};

const PROGRAM_NAME: &'static str = "salad";

#[cfg(target_pointer_width = "64")]
fn get_buffer() -> [u8; 8] {
    [0, 0, 0, 0, 0, 0, 0, 0]
}

#[cfg(target_pointer_width = "32")]
fn get_buffer() -> [u8; 4] {
    [0, 0, 0, 0]
}

struct Prefs {
    min: u8,
    max: u8,
    count: u8,
    words: File,
    stats: bool,
    generate: bool,
}

fn main() {
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("Error: {:?}", err);
            1
        }
    });
}

// Get a count of words matching the min and max criteria.
fn get_count(words: &mut File, min: u8, max: u8) -> Result<usize, &'static str> {
    let min = min as usize;
    let max = max as usize;
    seek_start(words)?;
    let mut count = 0;
    for line in BufReader::new(words).lines() {
        match line {
            Ok(line) => {
                let len = line.len();
                if len > 0 && len >= min && len <= max {
                    count += 1;
                }
            }
            Err(_) => return Err("Error reading from word file"),
        }
    }
    if count > 0 {
        Ok(count)
    } else {
        Err("No valid words found")
    }
}

fn get_prefs() -> Result<Prefs, &'static str> {
    let words = get_word_file()?;
    Ok(Prefs {
        min: 5,
        max: 10,
        count: 6,
        words,
        stats: true,
        generate: true,
    })
}

// Get count random numbers between 0 and max, inclusive.
fn get_random(count: u8, max: usize) -> Result<Vec<usize>, &'static str> {
    let unused_bits = max.leading_zeros();
    let mask = usize::max_value().wrapping_shr(unused_bits);
    let mut buffer = get_buffer();
    let needed_bytes = buffer.len() - (unused_bits as usize) / 8;
    match File::open("/dev/urandom") {
        Ok(mut random) => {
            let mut results = Vec::new();
            while results.len() < count as usize {
                match random.read(&mut buffer[0..needed_bytes]) {
                    Ok(bytes_read) => {
                        if bytes_read < needed_bytes {
                            return Err("Insufficient random bytes read.");
                        }
                        let n = usize::from_le_bytes(buffer) & mask;
                        if n <= max {
                            results.push(n);
                        }
                    }
                    Err(_) => return Err("Unable to read from /dev/urandom"),
                }
            }
            Ok(results)
        }
        Err(_) => return Err("Unable to open /dev/urandom"),
    }
}

fn get_word_file() -> Result<File, &'static str> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match File::open(&args[1]) {
            Ok(file) => Ok(file),
            Err(_) => Err("Unable to open word list"),
        }
    } else {
        if let Ok(home) = env::var("HOME") {
            if let Ok(file) = File::open(home + "/." + PROGRAM_NAME + "/words") {
                return Ok(file);
            }
        }
        match File::open(String::from("/etc/") + PROGRAM_NAME + "/words") {
            Ok(file) => Ok(file),
            Err(_) => Err("Unable to open word list"),
        }
    }
}

// Get the words at the specified indices, ignoring words which do not match the min and max criteria.
fn get_words(
    indices: Vec<usize>,
    words: &mut File,
    min: u8,
    max: u8,
) -> Result<Vec<String>, &'static str> {
    let mut results = vec!["".to_string(); indices.len()];
    let min = min as usize;
    let max = max as usize;
    seek_start(words)?;
    let mut count = 0;
    let mut found = 0;
    for line in BufReader::new(words).lines() {
        match line {
            Ok(line) => {
                let len = line.len();
                if len > 0 && len >= min && len <= max {
                    for (i, &index) in indices.iter().enumerate() {
                        if count == index {
                            results[i] = line.clone();
                            found += 1;
                            if found == indices.len() {
                                return Ok(results);
                            }
                        }
                    }
                    count += 1;
                }
            }
            Err(_) => return Err("Error reading from word file"),
        }
    }
    Err("Unexpected end of word list")
}

fn run() -> Result<(), &'static str> {
    let mut prefs = get_prefs()?;
    let word_count = get_count(&mut prefs.words, prefs.min, prefs.max)?;
    if prefs.stats {
        println!("Word count: {}", word_count);
        println!(
            "Entropy: {}",
            f64::from(prefs.count) * f64::from(word_count as u32).log2()
        );
    }
    if prefs.generate {
        let indices = get_random(prefs.count, word_count - 1)?;
        let words = get_words(indices, &mut prefs.words, prefs.min, prefs.max)?;
        for word in words {
            println!("{} ", word);
        }
    }
    Ok(())
}

fn seek_start(file: &mut File) -> Result<(), &'static str> {
    match file.seek(SeekFrom::Start(0)) {
        Ok(_) => Ok(()),
        Err(_) => return Err("Error seeking to start of word file"),
    }
}
