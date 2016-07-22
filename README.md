# rust-shannon
This program calculates the entropy of the files passed as arguments. 
The entropy here is a measure of information: files with high entropy are usually compressed/encrypted or contain random data. Files with low entropy can easyly be compressed, but this doesn't mean that files with high entropy can't. The units for entropy in this context are "bits / byte", going from 0 meaning low entropy and 8 meaning high entropy.
Compile it using `cargo run`.
You can try it by passing files as arguments: `cargo run -- filename`.

### Installation
To use it system wide run `cargo run --release` and copy the executable from `target/release/rust-shannon` to a path in your $PATH (for example /usr/local/bin).

### Usage
```
rust-shannon filenames
```

