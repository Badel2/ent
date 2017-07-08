# ent
This program calculates the entropy of the files passed as arguments. 
The entropy here is a measure of information: files with high entropy are usually compressed/encrypted or contain random data. Files with low entropy can easyly be compressed, but this doesn't mean that files with high entropy can't. The units for entropy in this context are "bits / byte", going from 0 meaning low entropy and 8 meaning high entropy.
Compile it using `cargo run`.
You can try it by passing files as arguments: `cargo run -- filename`.

### Installation
To use it system wide run `cargo install`. This will copy the ent executable to ~/.cargo/bin/ent.

### Usage
```
ent filenames
```

Example:

```
$ ent test*
0.00000  [   0.0 B ]  test0
0.00000  [   8.2 K ]  test1
7.98274  [  10.2 K ]  test2

```
