//! [lies](https://docs.rs/lies/) implementation details.

extern crate proc_macro;

use proc_macro_hack::*;
use proc_macro::*;
use quote::quote;

use std::ffi::*;
use std::fs::*;
use std::io::{self, Write};
use std::iter::*;
use std::path::*;
use std::process::*;

#[proc_macro_hack]
pub fn licenses_text(_input: TokenStream) -> TokenStream {
    use_cargo_about(include_bytes!("../templates/about.console.hbs"), "about.console.hbs")
}

#[proc_macro_hack]
pub fn licenses_ansi(_input: TokenStream) -> TokenStream {
    use_cargo_about(include_bytes!("../templates/about.ansi.hbs"), "about.ansi.hbs")
}

fn use_cargo_about(input_text: &[u8], input_name: &str) -> TokenStream {
    let cargo_about = ensure_cargo_about_installed();
    let about_toml = ensure_about_toml_exists();

    let tmp_template_path = std::env::temp_dir().join(format!("{}-{}-{}",
        get_env_path("CARGO_PKG_NAME"   ).display(),
        get_env_path("CARGO_PKG_VERSION").display(),
        input_name
    ));

    File::create(&tmp_template_path)
        .expect("Unable to create temporary .hbs file")
        .write_all(input_text)
        .expect("Unable to write entire temporary .hbs file");

    let output = match cmd_output(format!("{} about generate {}", cargo_about.display(), tmp_template_path.display()).as_str()) {
        Ok(o) => o,
        Err(err) => {
            eprintln!("Failed to '{} about generate {}'", cargo_about.display(), tmp_template_path.display());
            eprintln!("{}", err);
            exit(1);
        },
    };

    let output = reprocess(output.as_str());

    let output = output.as_str();
    let about_toml = about_toml.to_str().expect("Path to about.toml contains invalid unicode");
    TokenStream::from(quote!{
        {
            const _ : &[u8] = include_bytes!(#about_toml); // Ensure license strings are rebuilt when modified [1]
            #output
        }
    })
}
// [1] https://internals.rust-lang.org/t/pre-rfc-add-a-builtin-macro-to-indicate-build-dependency-to-file/9242/2

fn ensure_cargo_about_installed() -> PathBuf {
    let expected_path = PathBuf::from("cargo-about");
    let version = cmd_output(format!("{} about --version", expected_path.display()).as_str()).ok();
    let version = version.as_ref().and_then(|output|{
        let ws = output.find(' ')?;
        let (_name, version) = output.split_at(ws);
        Some(version.trim()) // leading ' ', trailing '\n'
    });

    let install = match version {
        None                                => { eprintln!("Installing cargo-about"); true },
        Some("0.0.1")                       => { eprintln!("Upgrading cargo-about"); true },
        Some(v) if v.starts_with("0.1.")    => false, // Expected version
        Some(v)                             => { eprintln!("cargo-about {} may have breaking changes vs expected version 0.1.x", v); false }, // Newer (0.2.x?) but leave alone
    };

    if install {
        if let Err(err) = cmd_run(format!("cargo install cargo-about --vers ^0.1 --force").as_str()) {
            eprintln!("Failed to install cargo-about 0.0.1: {}", err);
            exit(1);
        }
    }

    expected_path
}

fn ensure_about_toml_exists() -> PathBuf {
    let path = Path::new("about.toml").canonicalize().expect("Cannot canonicalize about.toml path");
    if !path.exists() {
        let mut about = File::create(&path).expect("about.toml does not exist, and cannot be opened for writing");
        about.write_all(include_bytes!("../templates/about.toml")).expect("Created but failed to fully write out about.toml");
    }
    path
}

fn reprocess(text: &str) -> String {
    let mut lines = text.lines().map(|line| line
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&copy;", "(c)")
    ).collect::<Vec<String>>();
    let lines_n = lines.len();

    for start_line in 0..lines_n {
        while lines[start_line].contains('\t') {
            // Find out the size of this "table"
            let mut max_col = 0;
            let mut end_line = start_line;
            while end_line < lines_n {
                if let Some(tab) = lines[end_line].find('\t') {
                    max_col = max_col.max(tab);
                    end_line += 1;
                } else {
                    break;
                }
            }

            max_col += 4; // Ensure minimum spacing

            // Fixup this "table"
            for line in start_line..end_line {
                let line = &mut lines[line];
                let tab = line.find('\t').unwrap(); // Already found it once
                let mut fixed = line[..tab].to_string();
                for _ in fixed.chars().count()..max_col {
                    fixed.push(' ');
                }
                fixed.push_str(&line[tab+1..]);
                *line = fixed;
            }
        }
    }

    lines.join("\n")
}



fn cmd(args: &str) -> Command {
    let mut args = args.split_whitespace();
    let exe = args.next().expect("cmd:  Expected a command");
    let mut cmd = Command::new(exe);
    for arg in args { cmd.arg(arg); }
    cmd
}

fn cmd_run(args: &str) -> io::Result<()> {
    let status = cmd(args).status()?;
    if !status.success() {
        Err(io::Error::new(io::ErrorKind::Other, format!("Failed to successfully run \"{}\": {:?}", args, status)))
    } else {
        Ok(())
    }
}

fn cmd_output(args: &str) -> io::Result<String> {
    let output = cmd(args).output()?;
    if !output.status.success() {
        let mut s = format!("Failed with {}: {}", output.status, args);
        for (channel,   output          ) in [
            ("stdout",  &output.stdout  ),
            ("stderr",  &output.stderr  ),
        ].iter().copied() {
            if !output.is_empty() {
                s.push_str("\n");
                s.push_str(channel);
                s.push_str(":\n");
                s.push_str("-------");
                s.push_str(&String::from_utf8_lossy(output));
            }
        }

        Err(io::Error::new(io::ErrorKind::Other, s))
    } else {
        String::from_utf8(output.stdout).map_err(|err| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{:?} output invalid UTF8: {}", args, err)
        ))
    }
}

fn get_env_path(name: &str) -> PathBuf {
    PathBuf::from(get_env_os(name))
}

fn get_env_os(name: &str) -> OsString {
    match std::env::var_os(name) {
        Some(v) => v,
        None => {
            if cfg!(windows) {
                eprintln!("%{}%: Not set", name);
                exit(1);
            } else {
                eprintln!("${{{}}}: Not set", name);
                exit(1);
            }
        },
    }
}
