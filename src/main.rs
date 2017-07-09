// file open 
use std::error::Error; 
use std::io::prelude::*; 
use std::ffi::OsString;
// args
use std::env;

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

fn pretty_ascii_table(t: &[u64; 256]) -> String {
    // Assuming a 80 width terminal with unicode support
    let freq_char = " ▁▂▃▄▅▆▇█";
    let freq_char_len = freq_char.chars().count();
    let total_freq = t.iter().fold(0, |a, &b| a + b);
    let max_freq = *t.iter().enumerate().map(|(x, y)| (y, x)).max().unwrap_or((&0u64, 0)).0 as f64;
    let mut s = String::with_capacity(4*80*3);
    s.push_str("   00 ");
    for (i, &x) in t.iter().enumerate() {
        let n = if x == 0 { 0 } else {
            let n = x as f64 / max_freq * (freq_char_len - 1) as f64;
            let mut n = n as usize;
            if n >= freq_char_len - 1 {
                n = freq_char_len - 2;
            }
            n+1
        };
        let c = freq_char.chars().nth(n as usize).unwrap();
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

struct Options {
    show_byte_frequency: bool,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            show_byte_frequency: false,
        }
    }
}

impl Options {
    fn process_file(&self, f: OsString) {
        match Shannon::open(&f) {
            Err(why) => writeln!(&mut std::io::stderr(), 
                    "couldn't open {}: {}", f.to_string_lossy(), why.description())
                    .expect("failed printing to stderr"),
            Ok(s) => {
                self.print_info(&s);
            }
        }
    }

    fn process_stdin(&self) {
        match Shannon::from_stdin() {
            Err(why) => writeln!(&mut std::io::stderr(), 
                    "couldn't open stdin: {}", why.description())
                    .expect("failed printing to stderr"),
            Ok(s) => {
                self.print_info(&s);
            }
        }
    }

    fn print_info(&self, s: &Shannon) {
        println!("{:.5}  [{}]  {}", s.entropy(), pretty_size(s.filesize()), s.filename());
        if self.show_byte_frequency {
            println!("  mean: {}, std: {:.5}, min: {} (0x{:0X}), max: {} (0x{:0X})", s.mean(), s.std_dev(), s.byte_min().1, s.byte_min().0, s.byte_max().1, s.byte_max().0);
            println!("{}",pretty_ascii_table(s.freq_table()));
        }
    }
}

fn main() { 
    let argv0 = env::args_os().next().unwrap();
    let args: Vec<_> = env::args_os().skip(1).collect();

    if args.len() == 0 {
        println!("Usage: {} filenames", argv0.to_string_lossy());
        return;
    }

    let mut o: Options = Default::default();
    let mut parse_args = true;
    for f in args {
        if parse_args && f.to_string_lossy().chars().nth(0) == Some('-') {
            // Arg is option
            match f.to_string_lossy().chars().nth(1) {
                // Help
                Some('h') => {},
                // print byte frequency
                Some('b') => o.show_byte_frequency = true,
                // no more args
                Some('-') => parse_args = false,
                // Unknown argument, retry as filename?
                Some(x) => o.process_file(f),
                // Get input from stdin
                None => o.process_stdin(),
            }
        } else {
            // Arg is filename
            parse_args = false;
            o.process_file(f);
        }
    }
}

#[test]
fn it_works(){
    use shannon::Shannon;
    assert_eq!(Shannon::open("test0").unwrap().entropy(), 0.0);
    assert_eq!(Shannon::open("test1").unwrap().entropy(), 0.0);
    let t2 = Shannon::open("test2").unwrap();
    assert_eq!(t2.entropy(), 7.982743005032543);
    assert_eq!(t2.filesize(), 10240);
}

#[test]
#[should_panic]
fn it_panics(){
    use shannon::Shannon;
    let _ent = Shannon::open("").unwrap();
}

#[test]
#[should_panic]
fn it_panics_also(){
    use shannon::Shannon;
    let _ent = Shannon::open("filethatdoenstexist").unwrap();
}

