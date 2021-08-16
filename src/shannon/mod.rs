use std::fs::File; 
use std::ffi::OsString;
use std::io::{BufRead, BufReader};
use std::path::Path;

// Idea: create a worker thread whose job is to calculate entropy given a
// chunk, this way the main thread only needs to count freq and send complete
// chunks for computation. After reading the filename, wait for the worker
// thread to finish and return Vec<Chunk> with calculated entropy.
// The worker thread could also do the increment, this way the main thread
// can spend more time waiting for I/O. Is it expensive to send an array to
// a new thread?

pub struct Shannon {
    filename: OsString,
    filesize: u64,
    freq_table: [u64; 256],
    entropy: f64,
    chunks: Vec<Chunk>,
    chunk_size: u32,
}

impl Shannon {
    pub fn read<R: BufRead>(r: &mut R, filename: OsString) -> Result<Shannon, std::io::Error> {
        let mut filesize: u64 = 0;
        let mut freq_table = [0u64; 256];
        let mut chunks = vec![];
        let chunk_size = 16 * 1024;
        let mut buffer = [0u8; 16 * 1024];

        let mut new_chunk = Chunk::new();
        let mut new_chunk_size = 0;

        // Read x bytes using BufRead. At EOF, x == 0
        loop {
            let x = r.read(&mut buffer)?;

            if x == 0 { break; }
            for &byte in buffer.iter().take(x) {
                freq_table[byte as usize] += 1;
                new_chunk.count(byte);
            }

            new_chunk_size += x;
            if new_chunk_size > chunk_size {
                panic!("Noooo!");
            }
            if new_chunk_size == chunk_size {
                // Chunk completed, save!
                new_chunk.calc_entropy();
                chunks.push(new_chunk);
                new_chunk = Chunk::new();
                new_chunk_size = 0;
            }

            filesize += x as u64;
        }

        // Push last (incomplete) chunk, its size can be calculated as
        // self.filesize - (self.chunks.len() - 1) * self.chunk_size
        if new_chunk_size != 0 {
            new_chunk.calc_entropy();
            chunks.push(new_chunk);
        }

        let mut entropy: f64 = 0.0;

        for &c in freq_table.iter(){
            if c != 0 {
                let temp: f64 = c as f64 / filesize as f64;
                entropy += -temp * f64::log2(temp);
            }
        }

        let chunk_size = chunk_size as u32;

        Ok( Shannon { 
            filename,
            filesize,
            freq_table,
            entropy,
            chunks,
            chunk_size,
        })
    }
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Shannon, std::io::Error> {
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        
        Self::read(&mut reader, OsString::from(path.as_ref().as_os_str()))
    }
    pub fn from_stdin() -> Result<Shannon, std::io::Error> {
        let stdin = std::io::stdin();
        let file = &mut stdin.lock();
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
    pub fn random_walk(&self) -> f64 {
        // Start at 0. For every bit, go left if 0 and go right if 1
        // In the end, normalize the scale so it's between -1 and +1

        // This array stores the number of ones in a nibble 
        let one_count = [ 0u8, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4 ];
        let mut start = 0;
        for (i, &x) in self.freq_table.iter().enumerate() {
            // A byte is two nibbles:
            let ones = one_count[i&0xf] + one_count[i>>4];
            start += (ones as i64 - 4) * 2 * x as i64;
        }

        start as f64 / self.filesize() as f64 / 8_f64
    }
    pub fn chunk_entropy(&self) -> Vec<f64> {
        self.chunks.iter().map(|c| c.entropy()).collect()
    }
}

pub struct Chunk {
    // max chunk size of 2^32 - 1 bytes
    freq_table: [u32; 256],
    entropy: f64,
}

impl Chunk {
    fn new() -> Self {
        Self { freq_table: [0; 256], entropy: 0.0 }
    }
    fn count(&mut self, byte: u8) {
        self.freq_table[byte as usize] += 1;
    }
    fn calc_entropy(&mut self) {
        let mut entropy: f64 = 0.0;
        let filesize: u32 = self.freq_table.iter().sum();

        for &c in self.freq_table.iter(){
            if c != 0 {
                let temp: f64 = c as f64 / filesize as f64;
                entropy += -temp * f64::log2(temp);
            }
        }

        self.entropy = entropy;
    }
    fn entropy(&self) -> f64 {
        self.entropy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works(){
        assert_eq!(Shannon::open("test0").unwrap().entropy(), 0.0);
        assert_eq!(Shannon::open("test1").unwrap().entropy(), 0.0);
        let t2 = Shannon::open("test2").unwrap();
        assert_eq!(t2.entropy(), 7.982743005032543);
        assert_eq!(t2.filesize(), 10240);
    }

    #[test]
    #[should_panic]
    fn it_panics(){
        let _ent = Shannon::open("").unwrap();
    }

    #[test]
    #[should_panic]
    fn it_panics_also(){
        let _ent = Shannon::open("filethatdoenstexist").unwrap();
    }

    #[test]
    fn slice_test() {
        use std::io::BufReader;
        let values: &[u8] = &[7, 2, 3, 4, 5, 6, 7];
        let mut r = BufReader::new(values);
        let (a, b) = Shannon::read(&mut r, "-".into()).unwrap().byte_max();
        assert_eq!(a, 7);
        assert_eq!(b, 2);
    }
}
