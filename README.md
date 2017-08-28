# rust-efi-app
High-level bindings for writing UEFI applications in Rust.

Currently in very early stages. The goal is that the library will make it easy and safe to write an application
(e.g. an OS kernel) that is capable of functioning across call to `ExitBootServices()` without depending on the programmer's
judgement for safety, transparently dealing with issues like memory allocations and console output.
