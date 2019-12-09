//! **LI**cense **E**mbedding **S**ystem
//! 
//! Wraps and reformats the results of [cargo-about] to embed license text and information
//! inside your console program, as a means of complying with licensing requirements.
//! 
//! # Examples
//! 
//! ```no_compile
//! println!("{}", lies::licenses_text!()); // Monochrome
//! println!("{}", lies::licenses_ansi!()); // https://en.wikipedia.org/wiki/ANSI_escape_code
//! ```
//! 
//! [cargo-about]:                  https://github.com/EmbarkStudios/cargo-about/

use proc_macro_hack::proc_macro_hack;

/// Format licensing information as a string for display with fixed-width fonts.
/// 
/// # Example
/// 
/// ```no_compile
/// println!("{}", lies::licenses_text!());
/// ```
#[proc_macro_hack]
pub use lies_impl::licenses_text;

/// Format licensing information as a string with [ANSI Color Escape Codes] for display with fixed-width fonts.
/// 
/// # Example
/// 
/// ```no_compile
/// println!("{}", lies::licenses_ansi!());
/// ```
/// 
/// [ANSI Color Escape Codes]:      https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
/// [cargo-about]:                  https://github.com/EmbarkStudios/cargo-about/
#[proc_macro_hack]
pub use lies_impl::licenses_ansi;

