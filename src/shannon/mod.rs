use std;
use std::fs::File; 
use std::ffi::OsString;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct Shannon {
    filename: OsString,
    filesize: u64,
    freq_table: [u64; 256],
    entropy: f64,
}

impl Shannon {
    pub fn read<R: BufRead>(r: &mut R, filename: OsString) -> Result<Shannon, std::io::Error> {
        let mut filesize : u64 = 0;
        let mut freq_table = [0u64; 256];
        let mut buffer = [0; 1024];

        // Read x bytes using a buffer. At EOF, x == 0
        loop {
            let x = r.read(&mut buffer)?;
            if x == 0 { break; }

            // Process x bytes:
            for &byte in buffer.iter().take(x){
                freq_table[byte as usize] += 1;
            }

            filesize += x as u64;
        }

        let mut entropy: f64 = 0.0;

        for &c in freq_table.iter(){
            if c != 0 {
                let temp: f64 = c as f64 / filesize as f64;
                entropy += -temp * f64::log2(temp);
            }
        }

        Ok( Shannon { 
            filename,
            filesize,
            freq_table,
            entropy,
        })
    }
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Shannon, std::io::Error> {
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        
        Self::read(&mut reader, OsString::from(path.as_ref().as_os_str()))
    }
    pub fn from_stdin() -> Result<Shannon, std::io::Error> {
        let stdin = std::io::stdin();
        let file = &mut stdin.lock() as &mut BufRead;
        let mut reader = BufReader::new(file);
        Self::read(&mut reader, OsString::from("-"))
    }
    pub fn filename(&self) -> String {
        self.filename.to_string_lossy().into_owned()
    }
    pub fn filesize(&self) -> u64 {
        self.filesize
    }
    pub fn freq_table(&self) -> &[u64; 256] {
        &self.freq_table
    }
    pub fn entropy(&self) -> f64 {
        self.entropy
    }
    pub fn mean(&self) -> f64 {
        self.filesize as f64 / 256_f64
    }
    // https://doc.rust-lang.org/1.1.0/src/test/stats.rs.html
    pub fn std_dev(&self) -> f64 {
        let mean = self.mean();
        let mut v: f64 = 0.0;
        for s in self.freq_table.iter() {
            let x = *s as f64 - mean;
            v += x*x;
        }
        let denom = (256 - 1) as f64;
        (v/denom).sqrt()
    }
    pub fn byte_min(&self) -> (u8, u64) {
        let (a, b) = self.freq_table.iter().enumerate().map(|(x, y)| (y, x)).min().unwrap();
        (b as u8, *a)
    }
    pub fn byte_max(&self) -> (u8, u64) {
        let (a, b) = self.freq_table.iter().enumerate().map(|(x, y)| (y, x)).max().unwrap();
        (b as u8, *a)
    }
}
