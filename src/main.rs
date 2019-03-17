use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Error, ErrorKind, Read, Result, Seek, SeekFrom},
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

fn get_count(words: &mut File, min: usize, max: usize) -> Result<usize> {
    words.seek(SeekFrom::Start(0))?;
    let mut count = 0;
    for line in BufReader::new(words).lines() {
        let line_len = line?.len();
        if line_len > 0 && line_len >= min && line_len <= max {
            count += 1;
        }
    }
    if count > 0 {
        Ok(count)
    } else {
        Err(Error::new(ErrorKind::Other, "No valid words found."))
    }
}

fn get_random(max: usize) -> Result<usize> {
    let unused_bits = max.leading_zeros();
    let mask = usize::max_value().wrapping_shr(unused_bits);
    let mut buffer = get_buffer();
    let needed_bytes = buffer.len() - (unused_bits as usize) / 8;
    let mut random = File::open("/dev/urandom")?;
    loop {
        let bytes_read = random.read(&mut buffer[0..needed_bytes])?;
        if bytes_read < needed_bytes {
            return Err(Error::new(
                ErrorKind::Other,
                "Insufficient random bytes read.",
            ));
        }
        let n = usize::from_le_bytes(buffer) & mask;
        if n <= max {
            return Ok(n);
        }
    }
}

fn get_word(words: &mut File, index: usize, min: usize, max: usize) -> Result<String> {
    words.seek(SeekFrom::Start(0))?;
    let mut count = 0;
    for line in BufReader::new(words).lines() {
        let line = line?;
        let line_len = line.len();
        if line_len > 0 && line_len >= min && line_len <= max {
            if count == index {
                return Ok(line);
            }
            count += 1;
        }
    }
    Err(Error::new(ErrorKind::Other, "Unexpected end of word list."))
}

fn get_words() -> Result<File> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        return File::open(&args[1]);
    }
    if let Ok(home) = env::var("HOME") {
        if let Ok(file) = File::open(home + "/." + PROGRAM_NAME + "/words") {
            return Ok(file);
        }
    };
    File::open(String::from("/etc/") + PROGRAM_NAME + "/words")
}

fn main() -> std::io::Result<()> {
    let min = 5;
    let max = 10;
    let passphrase_len = 6;
    let mut words = get_words()?;
    let word_count = get_count(&mut words, min, max)?;
    println!("Word count: {}", word_count);
    println!(
        "Entropy: {}",
        f64::from(passphrase_len) * f64::from(word_count as u32).log2()
    );
    for _ in 0..passphrase_len {
        let index = get_random(word_count - 1)?;
        let word = get_word(&mut words, index, min, max)?;
        println!("{} ", word);
    }
    Ok(())
}
