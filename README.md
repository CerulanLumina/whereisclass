# whereisclass

`whereisclass` is a small command-line based application that provides a
number of utilities for the RPI master schedule.

### Features
-  Parsing of the RCOS XML made available by the registrar
-  Parsing of the RPI SIS course listing table
-  Finding empty rooms on campus at a given time and day
-  Finding out which classes are being held in a given room

### Usage
The command-line program is somewhat documented through its `--help` but
in general, the workflow is:
1. Import and parse master course schedule if you haven't done so yet.
2. Use the output of that (a JSON file) to run the other commands

I don't distribute binaries - too many license quagmires. Go ahead and
compile it yourself! (It's not hard)

### Compiling
[Grab the latest version of the Rust compiler](https://www.rust-lang.org/),
then run:

```cargo build --release```.

The build artifacts will be in `target/release/whereisclass`. Everything
is statically linked, so no need to worry about library files.

### License
Licensed under the GPL 3.0. *Infectious*~~

