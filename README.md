# ArnOs

This is an attempt to write micro vms and run them with an home made Rust unikernel OS.

## Short term (first and current stage)

Running a very minimal kernel written in Rust with AWS Firecracker.

## Long term (who knows when ?)

Have a minimal kernel providing WASI and running WASM. (Still) run it with AWS Firecracker.

Expand Firecracker to manage multiple VMs on the same process, and provide a REST api for VM management.

## Goal

Since WASM is supported by more and more languages, this would mean that writing secure and fast unikernels in any language will be possible. The micro VM would only support network and block device, that is well enough for _functions as a service_...

Learn Rust with low level / system programming (no standard library use).

Deep dive into kernel writing.

## Usage

Build the project with this command :

```bash
cargo xbuild
```

Run Firecracker.

Then run the following script :

```bash
./run-on-firecracker.sh
```

The Firecracker launched VM should output "Hello 42 times !"

## Credits

This would have not been possible without those two things :