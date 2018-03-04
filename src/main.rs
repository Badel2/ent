extern crate clap;
use clap::{App, Arg};
use std::error::Error; 
use std::ffi::OsString;

mod shannon;
use shannon::Shannon;

fn pretty_size(s: u64) -> String {
    let units = ['B', 'K', 'M', 'G', 'T', 'P', 'E', 'Z', 'Y'];

    let mut i = 0;
    let mut r = s as f64;
    while r >= 1000_f64 {
        r /= 1000_f64;
        i += 1;
    }
    
    format!("{:>6.1} {} ", r, units[i])
}

const FREQ_CHAR: [char; 9] = [
    ' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█',
];

fn pretty_ascii_table(t: &[u64; 256]) -> String {
    // Assuming a 80 width terminal with unicode support
    //let total_freq = t.iter().fold(0, |a, &b| a + b);
    let max_freq = *t.iter().enumerate().map(|(x, y)| (y, x)).max().unwrap_or((&0u64, 0)).0 as f64;
    let mut s = String::with_capacity(4*80*3);
    s.push_str("   00 ");
    for (i, &x) in t.iter().enumerate() {
        let n = if x == 0 { 0 } else {
            let n = x as f64 / max_freq * (FREQ_CHAR.len() - 1) as f64;
            let mut n = n as usize;
            if n >= FREQ_CHAR.len() - 1 {
                n = FREQ_CHAR.len() - 2;
            }
            n+1
        };
        let c = FREQ_CHAR[n as usize];
        s.push(c);
        match i { 
            0x1F => s.push_str("   20 "),
            0x3F => s.push_str("\n   40 "),
            0x5F => s.push_str("   60 "),
            0x7F => s.push_str("\n   80 "),
            0x9F => s.push_str("   A0 "),
            0xBF => s.push_str("\n   C0 "),
            0xDF => s.push_str("   E0 "),
            _ => {}
        } 
    }

    s
}

fn pretty_chunk_bar(ce: &[f64]) -> String {
    // TODO: make this configurable
    let chunk_size = "16KiB";
    let line_size = "16KiB * 64 = 1MiB";
    
    let mut s = String::with_capacity(4*80*3);
    s.push_str(&format!(
        "  Chunk size: {}    Line size: {}\n   {:04} ", chunk_size, line_size, 0));
    for (i, &x) in ce.iter().enumerate() {
        // Map entropy from [0.0, 8.0] to [1, 8]
        let mut n = (x + 0.5) as usize + 1;
        if n >= 9 {
            n = 8;
        }
        let c = FREQ_CHAR[n as usize];
        s.push(c);
        match i % 64 { 
            63 => s.push_str(&format!("\n   {:04} ", 1 + i / 64)),
            _ => {}
        }
    }

    s
}

struct Options {
    show_byte_frequency: bool,
    show_free_space: bool,
    show_chunks: bool,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            show_byte_frequency: false,
            show_free_space: false,
            show_chunks: false,
        }
    }
}

impl Options {
    fn process_file(&self, f: OsString) {
        match Shannon::open(&f) {
            Err(why) => eprintln!(
                    "couldn't open {}: {}", f.to_string_lossy(), why.description()),
            Ok(s) => {
                self.print_info(&s);
            }
        }
    }

    fn process_stdin(&self) {
        match Shannon::from_stdin() {
            Err(why) => eprintln!(
                    "couldn't open stdin: {}", why.description()),
            Ok(s) => {
                self.print_info(&s);
            }
        }
    }

    fn print_info(&self, s: &Shannon) {
        let filesize = if self.show_free_space {
            (s.filesize() as f64 * (1_f64 - s.entropy()/8_f64)) as u64
        } else {
            s.filesize()
        };
        println!("{:.5}  [{}]  {}", s.entropy(), pretty_size(filesize), s.filename());
        if self.show_byte_frequency {
            println!("  mean: {}, std: {:.5}, min: {} (0x{:0X}), max: {} (0x{:0X})", s.mean(), s.std_dev(), s.byte_min().1, s.byte_min().0, s.byte_max().1, s.byte_max().0);
            println!("  Random walk ends at: {:+.5}", s.random_walk());
            println!("{}",pretty_ascii_table(s.freq_table()));
        }
        if self.show_chunks {
            //println!("{:#?}", s.chunk_entropy());
            println!("{}", pretty_chunk_bar(&s.chunk_entropy()));
        }
    }
}

fn main() { 
    let matches = App::new("ent")
        .arg(Arg::with_name("filenames")
            .help("Input files")
            .index(1)
            .multiple(true)
            .required(true))
        .arg(Arg::with_name("table")
            .help("Print ascii table")
            .short("b")
            .long("table"))
        .arg(Arg::with_name("free")
            .help("Show how many bytes could be freed if the files were compressed")
            .short("f")
            .long("free"))
        .arg(Arg::with_name("chunks")
            .help("Show details for each chunk")
            .short("c")
            .long("chunks"))
        .get_matches();

    let mut o: Options = Default::default();

    if matches.is_present("table") {
        o.show_byte_frequency = true;
    }
    if matches.is_present("free") {
        o.show_free_space = true;
    }
    if matches.is_present("chunks") {
        o.show_chunks = true;
    }

    if let Some(in_v) = matches.values_of("filenames") {
        for f in in_v {
            if f == "-" {
                o.process_stdin()
            } else {
                o.process_file(OsString::from(f));
            }
        }
    }
}


pub mod tests {
    use super::*;

    #[test]
    fn constant_len_9() {
        assert!(FREQ_CHAR.len() == 9);
    }
}
