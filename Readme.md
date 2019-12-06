# lies

**LIES** - **LI**cense **E**mbedding **S**ystem

[![GitHub](https://img.shields.io/github/stars/MaulingMonkey/lies.svg?label=GitHub&style=social)](https://github.com/MaulingMonkey/lies)
[![Build Status](https://travis-ci.org/MaulingMonkey/lies.svg)](https://travis-ci.org/MaulingMonkey/lies)
[![Crates.io](https://img.shields.io/crates/v/lies.svg)](https://crates.io/crates/lies)
![unsafe: no](https://img.shields.io/badge/unsafe-no-green.svg)
![rust: 1.39.0+](https://img.shields.io/badge/rust-1.39.0%2B-green.svg)
[![Open issues](https://img.shields.io/github/issues-raw/MaulingMonkey/lies.svg)](https://github.com/MaulingMonkey/lies/issues)
[![License](https://img.shields.io/crates/l/lies.svg)](https://github.com/MaulingMonkey/lies)
[![Docs](https://docs.rs/lies/badge.svg)](https://docs.rs/lies/)
<!--[![dependency status](https://deps.rs/repo/github/MaulingMonkey/lies/status.svg)](https://deps.rs/repo/github/MaulingMonkey/lies)-->

Wraps and reformats the results of [cargo-about] to embed license text and information
inside your console program, as a means of complying with licensing requirements.

# Examples

```rust
println!("{}", lies::licenses_text!()); // Monochrome
println!("{}", lies::licenses_ansi!()); // https://en.wikipedia.org/wiki/ANSI_escape_code
```

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

<!-- https://doc.rust-lang.org/1.4.0/complement-project-faq.html#why-dual-mit/asl2-license? -->
<!-- https://rust-lang-nursery.github.io/api-guidelines/necessities.html#crate-and-its-dependencies-have-a-permissive-license-c-permissive -->
<!-- https://choosealicense.com/licenses/apache-2.0/ -->
<!-- https://choosealicense.com/licenses/mit/ -->

[cargo-about]:              https://github.com/EmbarkStudios/cargo-about/
