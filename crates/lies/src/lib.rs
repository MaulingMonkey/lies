//! **LI**cense **E**mbedding **S**ystem
//! 
//! Wraps and reformats the results of [cargo-about] to embed license text and information
//! inside your console program, as a means of complying with licensing requirements.
//! 
//! # Examples
//! 
//! ```
//! println!("{}", lies::licenses_text!()); // Monochrome
//! println!("{}", lies::licenses_ansi!()); // https://en.wikipedia.org/wiki/ANSI_escape_code
//! ```
//! 
//! [cargo-about]:                  https://github.com/EmbarkStudios/cargo-about/

use proc_macro_hack::proc_macro_hack;

use std::io;
use std::process::Command;



/// Format licensing information as a string for display with fixed-width fonts.
/// 
/// # Example
/// 
/// ```rust
/// println!("{}", lies::licenses_text!());
/// ```
#[proc_macro_hack]
pub use lies_impl::licenses_text;

/// Format licensing information as a string with [ANSI Color Escape Codes] for display with fixed-width fonts.
/// 
/// # Example
/// 
/// ```rust
/// println!("{}", lies::licenses_ansi!());
/// ```
/// 
/// [ANSI Color Escape Codes]:      https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
/// [cargo-about]:                  https://github.com/EmbarkStudios/cargo-about/
#[proc_macro_hack]
pub use lies_impl::licenses_ansi;

/// Format licensing information as a complete HTML5 page.
///
/// # Example
///
/// ```rust
/// println!("{}", lies::licenses_html_page!());
/// ```
///
/// # Output
///
/// ```html
/// <!DOCTYPE html>
///     <html><head>
///     <meta charset="UTF-8">
///     ...
/// ```
#[proc_macro_hack]
pub use lies_impl::licenses_html_page;

/// Format licensing information as an unstyled HTML5 div.
///
/// # Example
///
/// ```rust
/// println!("{}", lies::licenses_html_div!());
/// ```
///
/// # Output
///
/// ```html
/// <div class="lies-container">
///     <div class="lies-intro">
///         <h1>Third Party Licenses</h1>
///         ...
/// ```
#[proc_macro_hack]
pub use lies_impl::licenses_html_div;

/// Open a browser window to the contents of [`licenses_html_page!`](licenses_html_page)
///
/// # Example
///
/// ```rust,no_run
/// lies::open_licenses_html_page!().unwrap();
/// ```
#[macro_export]
#[cfg(any(windows, target_os = "linux", target_os = "macos"))]
macro_rules! open_licenses_html_page {
    () => {
        $crate::open_html_page(
            concat!(env!("CARGO_PKG_NAME"), "-", env!("CARGO_PKG_VERSION"), "-licenses.html"),
            $crate::licenses_html_page!()
        )
    };
}

#[doc(hidden)]
#[cfg(any(windows, target_os = "linux", target_os = "macos"))]
pub fn open_html_page(name: &str, html: &str) -> io::Result<()> {
    let temp_html = std::env::temp_dir().join(name);
    std::fs::write(&temp_html, html)?;

    let mut cmd : Command;
    if cfg!(windows) {
        cmd = Command::new("cmd");
        cmd.arg("/C").arg("start").arg("").arg(&temp_html);
    } else if cfg!(target_os = "linux") {
        cmd = Command::new("xdg-open");
        cmd.arg(&temp_html);
    } else if cfg!(target_os = "macos") {
        cmd = Command::new("open");
        cmd.arg(&temp_html);
    } else {
        return Err(io::Error::new(io::ErrorKind::Other, "`lies` doesn't implement opening browsers for this platform yet"));
    }

    match cmd.status()?.code() {
        Some(0)         => Ok(()),
        Some(_) | None  => Err(io::Error::new(io::ErrorKind::Other, "unable to launch browser for page")),
    }
}
