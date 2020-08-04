use std::{
    env,
    error::Error,
    fmt,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    iter,
    result::Result,
};
use Phrase::*;
use Problem::*;

const PROGRAM: &'static str = "salad";
const USAGE: &'static str = "USAGE: salad [OPTION]...

Generate a passphrase from a file containing a list of words.

EXAMPLE  
salad -M floyd -min 4 -max 8

DEFAULTS  
-m -n 6 -max 12 -min 5

OPTIONS  
-h, --help  
  Display usage help

-max N  
  Ignore words larger than N. N must be less than 256.

-min N  
  Ignore words smaller than N. N must be less than 256.

-n N  
  Generate a passphrase with N words. N must be less than 256.

-r  
  Generate a passphrase of random words. Mutually exclusive with -m and -M.

-m  
  Generate a passphrase using a ramdomly chosen mnemonic. Mutually exclusive with -r and -M.

-M MNEMONIC  
  Generate a passphrase using the specified mnemonic. Mutually exclusive with -r and -m. The option -n is ignored if this is used.

-w FILE  
  Use a custom word-file. If no custom word-file is provided, salad will look in $HOME/.salad/words first and /etc/salad/words second. 
";

#[derive(Debug)]
enum ArgState {
    BeginArg,
    Max,
    Min,
    Num,
    Mnemonic,
    Words,
}

#[derive(Clone, Debug)]
enum Phrase {
    DynamicMnemonic,
    FixedMnemonic(String),
    Random,
}

#[derive(Debug)]
enum Problem {
    NoMatchingWords,
    OpenRandomFile,
    OpenWordFile,
    ReadRandomFile,
    ReadWordFile,
    SeekStartOfWordFile,
    TooFewRandomBytes,
    UnexpectedEndOfWordFile,
    Usage,
    WordFileTooLarge,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            NoMatchingWords => "No Matching words found.",
            OpenRandomFile => "Can not open random file.",
            OpenWordFile => "Can not open word file.",
            ReadRandomFile => "Can not read form random file.",
            ReadWordFile => "Can not read from word file.",
            SeekStartOfWordFile => "Can not seek to start of word file.",
            TooFewRandomBytes => "Too few random bytes read.",
            UnexpectedEndOfWordFile => "Unexpected end of word file.",
            Usage => &USAGE,
            WordFileTooLarge => "Word file is too large.",
        };
        write!(f, "{}", msg)
    }
}
impl Error for Problem {}

#[derive(Debug)]
struct Prefs {
    max_chars: u8,
    min_chars: u8,
    num_words: u8,
    phrase: Phrase,
}

fn default_word_file() -> Result<File, Problem> {
    if let Ok(home) = env::var("HOME") {
        if let Ok(file) = File::open(home + "/." + PROGRAM + "/words") {
            return Ok(file);
        }
    }
    match File::open(String::from("/etc/") + PROGRAM + "/words") {
        Ok(file) => Ok(file),
        Err(_) => Err(OpenWordFile),
    }
}

fn generate() -> Result<(), Problem> {
    let (prefs, mut word_file) = process_args()?;
    let max_chars = prefs.max_chars;
    let min_chars = prefs.min_chars;
    let num_words = prefs.num_words;
    let phrase_words = match prefs.phrase {
        DynamicMnemonic => {
            let m_max_nums = word_counts(&mut word_file, num_words, num_words, &[])?;
            let mnemonic =
                random_words(&mut word_file, num_words, num_words, &[], &m_max_nums)?[0].clone();
            let starting_chars: Vec<char> = mnemonic.chars().collect();
            let max_nums = word_counts(&mut word_file, max_chars, min_chars, &starting_chars)?;
            random_words(
                &mut word_file,
                max_chars,
                min_chars,
                &starting_chars,
                &max_nums,
            )?
        }
        FixedMnemonic(mnemonic) => {
            let starting_chars: Vec<char> = mnemonic.chars().collect();
            let max_nums = word_counts(&mut word_file, max_chars, min_chars, &starting_chars)?;
            random_words(
                &mut word_file,
                max_chars,
                min_chars,
                &starting_chars,
                &max_nums,
            )?
        }
        Random => {
            let max_num = word_counts(&mut word_file, max_chars, min_chars, &[])?[0];
            let max_nums: Vec<u32> = iter::repeat(max_num).take(usize::from(num_words)).collect();
            random_words(&mut word_file, max_chars, min_chars, &[], &max_nums)?
        }
    };
    for w in phrase_words {
        println!("{}", w);
    }
    Ok(())
}

fn main() {
    std::process::exit(match generate() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("Error: {}", err);
            1
        }
    });
}

fn process_arg(
    arg_state: &ArgState,
    arg: &str,
    prefs: &mut Prefs,
    word_file: &mut Option<File>,
) -> Result<ArgState, Problem> {
    use ArgState::*;
    let arg_state = match arg_state {
        BeginArg => match arg {
            "-h" | "--help" => {
                println!("{}", USAGE);
                std::process::exit(0);
            }
            "-max" => Max,
            "-min" => Min,
            "-n" => Num,
            "-r" => {
                prefs.phrase = Phrase::Random;
                BeginArg
            }
            "-m" => {
                prefs.phrase = Phrase::DynamicMnemonic;
                BeginArg
            }
            "-M" => Mnemonic,
            "-w" => Words,
            _ => return Err(Usage),
        },
        Max => {
            let max = process_int_arg(arg)?;
            prefs.max_chars = max;
            if prefs.min_chars > max {
                prefs.min_chars = max;
            }
            BeginArg
        }
        Min => {
            let min = process_int_arg(arg)?;
            prefs.min_chars = min;
            if prefs.max_chars < min {
                prefs.max_chars = min;
            }
            BeginArg
        }
        Num => {
            prefs.num_words = process_int_arg(arg)?;
            BeginArg
        }
        Mnemonic => {
            prefs.phrase = Phrase::FixedMnemonic(arg.to_string());
            BeginArg
        }
        Words => {
            if let Ok(file) = File::open(arg) {
                *word_file = Some(file);
                BeginArg
            } else {
                return Err(OpenWordFile);
            }
        }
    };
    Ok(arg_state)
}

fn process_args() -> Result<(Prefs, File), Problem> {
    let mut prefs = Prefs {
        max_chars: 12,
        min_chars: 5,
        num_words: 6,
        phrase: DynamicMnemonic,
    };
    let mut word_file: Option<File> = None;
    let mut arg_state = ArgState::BeginArg;
    let args = env::args();
    for arg in args.skip(1) {
        arg_state = process_arg(&arg_state, &arg, &mut prefs, &mut word_file)?;
    }
    match arg_state {
        ArgState::BeginArg => match word_file {
            Some(word_file) => Ok((prefs, word_file)),
            None => Ok((prefs, default_word_file()?)),
        },
        _ => Err(Usage),
    }
}

fn process_int_arg(arg: &str) -> Result<u8, Problem> {
    if let Ok(num) = arg.parse() {
        Ok(num)
    } else {
        Err(Usage)
    }
}

fn random_numbers(max_numbers: &[u32]) -> Result<Vec<u32>, Problem> {
    let num_results = max_numbers.len();
    let mut rand_nums = Vec::with_capacity(num_results);
    for max_number in max_numbers.iter() {
        let unused_bits = max_number.leading_zeros();
        let mask = u32::max_value().wrapping_shr(unused_bits);
        let mut buffer = [0, 0, 0, 0];
        let needed_bytes = buffer.len() - (unused_bits as usize) / 8;
        match File::open("/dev/urandom") {
            Ok(mut random) => loop {
                match random.read(&mut buffer[0..needed_bytes]) {
                    Ok(bytes_read) => {
                        if bytes_read < needed_bytes {
                            return Err(TooFewRandomBytes);
                        }
                        let n = u32::from_le_bytes(buffer) & mask;
                        if n <= *max_number {
                            rand_nums.push(n);
                            break;
                        }
                    }
                    Err(_) => return Err(ReadRandomFile),
                }
            },
            Err(_) => return Err(OpenRandomFile),
        }
    }
    Ok(rand_nums)
}

fn random_words(
    word_file: &mut File,
    max_chars: u8,
    min_chars: u8,
    starting_chars: &[char],
    max_nums: &[u32],
) -> Result<Vec<String>, Problem> {
    let indices = random_numbers(&max_nums)?;
    words(word_file, max_chars, min_chars, starting_chars, &indices)
}

fn seek_start(file: &mut File) -> Result<(), Problem> {
    match file.seek(SeekFrom::Start(0)) {
        Ok(_) => Ok(()),
        Err(_) => return Err(SeekStartOfWordFile),
    }
}

//Get counts of words that match each of the given starting characters.
fn word_counts(
    word_file: &mut File,
    max_chars: u8,
    min_chars: u8,
    starting_chars: &[char],
) -> Result<Vec<u32>, Problem> {
    seek_start(word_file)?;
    let min = usize::from(min_chars);
    let max = usize::from(max_chars);
    let num_results = match starting_chars.len() {
        0 => 1,
        num => num,
    };
    let mut counts = vec![0_u32; num_results];
    for line in BufReader::new(word_file).lines() {
        match line {
            Ok(line) => {
                let len = line.len();
                if len > 0 {
                    let starting_char = line.chars().next().unwrap();
                    for i in 0..num_results {
                        if len >= min && len <= max {
                            let c = starting_chars.get(i);
                            if c.is_some() {
                                if *c.unwrap() != starting_char {
                                    continue;
                                }
                            }
                            let (new_count, overflow) = counts[i].overflowing_add(1);
                            if overflow {
                                return Err(WordFileTooLarge);
                            }
                            counts[i] = new_count;
                        }
                    }
                }
            }
            Err(_) => return Err(ReadWordFile),
        }
    }
    for count in counts.iter() {
        if *count == 0 {
            return Err(NoMatchingWords);
        }
    }
    Ok(counts)
}

// Get the word at the specified index, only counting words which match the criteria.
fn words(
    word_file: &mut File,
    max_chars: u8,
    min_chars: u8,
    starting_chars: &[char],
    indices: &[u32],
) -> Result<Vec<String>, Problem> {
    seek_start(word_file)?;
    let min = usize::from(min_chars);
    let max = usize::from(max_chars);
    let num_results = indices.len();
    let mut counts = vec![0_u32; num_results];
    let mut words = vec![String::new(); num_results];
    let mut found = 0;
    for line in BufReader::new(word_file).lines() {
        match line {
            Ok(line) => {
                let len = line.len();
                if len > 0 {
                    let starting_char = line.chars().next().unwrap();
                    for i in 0..num_results {
                        if len >= min && len <= max {
                            let c = starting_chars.get(i);
                            if c.is_some() {
                                if *c.unwrap() != starting_char {
                                    continue;
                                }
                            }
                            let count = counts[i];
                            if count == indices[i] {
                                words[i] = line.clone();
                                found += 1;
                                if found == num_results {
                                    return Ok(words);
                                }
                            }
                            let (new_count, overflow) = count.overflowing_add(1);
                            if overflow {
                                return Err(WordFileTooLarge);
                            }
                            counts[i] = new_count;
                        }
                    }
                }
            }
            Err(_) => return Err(ReadWordFile),
        }
    }
    Err(UnexpectedEndOfWordFile)
}
