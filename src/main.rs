// open.rs 
use std::error::Error; 
use std::fs::File; 
use std::path::Path;
use std::io::prelude::*; 
// args
use std::env;

fn calculate_entropy<P: AsRef<Path>>(path : P) -> Result<f64,std::io::Error> {
    let mut file = try!(File::open(&path));

    // This doesn't work for pipes
    //let filesize : u64 = try!(fs::metadata(&path)).len();
    let mut filesize : u64 = 0;
    let mut freq_table = [0u64; 256];
    let mut buffer = [0; 1024];

    let mut x = try!(file.read(&mut buffer));
    while x > 0 {
        // Process x bytes:
        for &byte in buffer.iter().take(x){
            freq_table[byte as usize] += 1;
        }
        filesize += x as u64;
        x = try!(file.read(&mut buffer));
    }

    let mut entropy : f64 = 0.0;

    for &c in freq_table.iter(){
        if c != 0 {
            let temp : f64 = c as f64 / filesize as f64;
            entropy += -temp * f64::log2(temp);
        }
    }
    Ok(entropy)
}

fn main() { 
    let args: Vec<_> = env::args_os().skip(1).collect();

    if args.len() == 0 {
        println!("Usage: ./rust-shannon filenames");
        return;
    }

    for f in args{
        match calculate_entropy(&f){
            Err(why) => writeln!(&mut std::io::stderr(), 
                    "couldn't open {}: {}", f.to_string_lossy(), why.description())
                    .expect("failed printing to stderr"),
            Ok(ent) => println!("{:.5}\t{}", ent, f.to_string_lossy()),
        }
    }
}

#[test]
fn it_works(){
    assert_eq!(calculate_entropy("test0").unwrap(), 0.0);
    assert_eq!(calculate_entropy("test1").unwrap(), 0.0);
    assert_eq!(calculate_entropy("test2").unwrap(), 7.982743005032543);
}

#[test]
#[should_panic]
fn it_panics(){
    let _ent = calculate_entropy("").unwrap();
}

#[test]
#[should_panic]
fn it_panics_also(){
    let _ent = calculate_entropy("filethatdoenstexist").unwrap();
}
