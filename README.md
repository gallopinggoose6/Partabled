# Partabled
A UEFI application for creating, securely deleting, and even merging partitions.

# Building
There are a few things you must do before building this application:

1. Have a working `cargo` instance with a nightly build installed (if you don't,
see [rustup.rs](https://rustup.rs) for help installing it)
2. Python 3 
3. OPTIONAL: An x86_64 QEMU system and OVMF firmware images installed 

To build the application, simply run:
```
python3 ./build.py build
```
For a release build:
```
python3 ./build.py --release build
```

The outputted `.efi` file can be found in the `target/x86_64-unknown-uefi/` 
directory under either `debug` or `release` depending on the build you chose.

To try out the application in a QEMU VM first, simply run the same commands as 
above, with `build` replaced with `run`. This will compile the program and then
run it inside a QEMU x86_64 virtual machine.