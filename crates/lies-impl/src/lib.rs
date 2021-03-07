//! [lies](https://docs.rs/lies/) implementation details.

extern crate proc_macro;

use proc_macro_hack::*;
use proc_macro::*;

use quote::quote;

use serde::*;

use std::ffi::*;
use std::fs::{self, *};
use std::io::{self, Write};
use std::path::*;
use std::process::*;



macro_rules! fatal {
    (user,      $($tt:tt)+) => {{ eprintln!($($tt)+); exit(1); }};
    (system,    $($tt:tt)+) => {{ eprintln!($($tt)+); exit(1); }};
    (bug,       $($tt:tt)+) => {{ eprintln!($($tt)+); eprintln!("This is a bug!  Please file an issue against https://github.com/MaulingMonkey/lies/issues if one doesn't already exist"); exit(1); }};
}

#[derive(Deserialize)] struct Metadata {
    workspace_root:     PathBuf,
    target_directory:   PathBuf,
}

impl Metadata {
    fn get() -> Self {
        let output = Command::new("cargo").args("metadata --format-version 1".split(' ')).stderr(Stdio::inherit()).output()
            .unwrap_or_else(|err| fatal!(system, "unable to execute `cargo metadata --format-version 1`: {}", err));
        if output.status.code() != Some(0) { fatal!(system, "error executing `cargo metadata --format-version 1`"); }
        serde_json::from_slice(&output.stdout[..]).unwrap_or_else(|err| fatal!(bug, "error parsing `cargo metadata --format-version 1`: {}", err))
    }
}

mod features {
    pub const ABOUT_PER_CRATE       : bool = cfg!(feature = "about-per-crate");
    pub const ABOUT_PER_WORKSPACE   : bool = cfg!(feature = "about-per-workspace");
}

lazy_static::lazy_static! {
    // NOTE:  I intentionally avoid listing most file paths here, to force you to use ensure_* methods to e.g. create them first.
    static ref CARGO_METADATA       : Metadata                  = Metadata::get();
    static ref WORKSPACE_DIR        : PathBuf                   = CARGO_METADATA.workspace_root.clone();
    static ref CARGO_MANIFEST_DIR   : PathBuf                   = get_env_path("CARGO_MANIFEST_DIR");
    static ref ABOUT_TOML_DIR       : PathBuf                   = get_about_toml_dir();
}

#[proc_macro_hack]
pub fn licenses_text(_input: TokenStream) -> TokenStream {
    emit_quote_cargo_about(include_bytes!("../templates/about.console.hbs"), "about.console.hbs", true)
}

#[proc_macro_hack]
pub fn licenses_ansi(_input: TokenStream) -> TokenStream {
    emit_quote_cargo_about(include_bytes!("../templates/about.ansi.hbs"), "about.ansi.hbs", true)
}

#[proc_macro_hack]
pub fn licenses_html_page(_input: TokenStream) -> TokenStream {
    emit_quote_cargo_about(include_bytes!("../templates/about.html.page.hbs"), "about.html.page.hbs", false)
}

#[proc_macro_hack]
pub fn licenses_html_div(_input: TokenStream) -> TokenStream {
    emit_quote_cargo_about(include_bytes!("../templates/about.html.div.hbs"), "about.html.div.hbs", false)
}

fn emit_quote_cargo_about(input_text: &[u8], input_name: &str, unhtml: bool) -> TokenStream {
    let cargo_lock      = WORKSPACE_DIR.join("Cargo.lock");
    let about_toml      = ensure_about_toml_exists();
    let about_out_txt   = ensure_about_out_txt_exists(input_text, input_name, &cargo_lock, &about_toml, unhtml);

    let cargo_lock      = cargo_lock    .to_str().unwrap_or_else(|| fatal!(system, "Path to Cargo.lock contains invalid unicode: {}", cargo_lock.display()));
    let about_toml      = about_toml    .to_str().unwrap_or_else(|| fatal!(system, "Path to about.toml contains invalid unicode: {}", about_toml.display()));
    let about_out_txt   = about_out_txt .to_str().unwrap_or_else(|| fatal!(system, "Path to about.out.txt contains invalid unicode: {}", about_out_txt.display()));

    TokenStream::from(quote!{
        {
            // Ensure license strings are rebuilt when modified [1]
            const _ : &'static [u8] = include_bytes!(#about_toml);
            const _ : &'static [u8] = include_bytes!(#cargo_lock);

            include_str!(#about_out_txt)
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
        Some(v) if v.starts_with("0.2.")    => false, // Expected version
        Some(v)                             => { eprintln!("cargo-about {} may have breaking changes vs expected version 0.1.x or 0.2.x", v); false }, // Newer (0.3.x?) but leave alone
    };

    if install {
        cmd_run(format!("cargo +stable install cargo-about --vers ^0.1 --force").as_str()).unwrap_or_else(|err|
            fatal!(system, "Failed to install cargo-about 0.0.1: {}", err)
        );
    }

    expected_path
}

fn ensure_about_toml_exists() -> PathBuf {
    let path = ABOUT_TOML_DIR.join("about.toml");
    if !path.exists() {
        let mut about = File::create(&path).unwrap_or_else(|err| fatal!(system, "about.toml does not exist, and cannot be opened for writing: {}", err));
        about.write_all(include_bytes!("../templates/about.toml")).unwrap_or_else(|err| fatal!(system, "Created but failed to fully write out about.toml: {}", err));
    }
    path
}

fn ensure_about_out_txt_exists(input_text: &[u8], input_name: &str, cargo_lock: &PathBuf, about_toml: &PathBuf, unhtml: bool) -> PathBuf {
    let cargo_about = ensure_cargo_about_installed();

    let target_lies = CARGO_METADATA.target_directory.join("lies");
    if !target_lies.exists() {
        create_dir_all(&target_lies).unwrap_or_else(|err| fatal!(system, "Failed to create target/lies directory: {}", err));
    }

    let about_out_txt = if !features::ABOUT_PER_WORKSPACE {
        format!("{}-{}-{}.out.txt", get_env_path("CARGO_PKG_NAME").display(), get_env_path("CARGO_PKG_VERSION").display(), input_name)
    } else {
        format!("{}.out.txt", input_name)
    };
    let about_out_txt = target_lies.join(about_out_txt);
    if let Ok(about_out_txt_mod) = about_out_txt.metadata().and_then(|md| md.modified()) {
        let mut up_to_date = true;
        for dependency in [cargo_lock, about_toml].iter() {
            let dep_mod = dependency
                .metadata().unwrap_or_else(|err| fatal!(system, "Cannot read {} metadata: {}", dependency.display(), err))
                .modified().unwrap_or_else(|err| fatal!(system, "Cannot read {} last modified time: {}", dependency.display(), err));
            if dep_mod >= about_out_txt_mod { // Dependency was modified more recently than result
                up_to_date = false;
            }
        }
        if up_to_date {
            return about_out_txt;
        }
    }

    let tmp_template_path = std::env::temp_dir().join(format!("{}-{}-{}",
        get_env_path("CARGO_PKG_NAME"   ).display(),
        get_env_path("CARGO_PKG_VERSION").display(),
        input_name
    ));

    File::create(&tmp_template_path)
        .unwrap_or_else(|err| fatal!(system, "Unable to create output .hbs file: {}", err))
        .write_all(input_text)
        .unwrap_or_else(|err| fatal!(system, "Unable to write entire output .hbs file: {}", err));

    let output = cmd_output(format!("{} about generate {}", cargo_about.display(), tmp_template_path.display()).as_str()).unwrap_or_else(|err|
        fatal!(system, "Failed to '{} about generate {}'\n{}", cargo_about.display(), tmp_template_path.display(), err)
    );

    let output = if unhtml { reprocess(output.as_str()) } else { output };
    fs::write(&about_out_txt, output).unwrap_or_else(|err| fatal!(system, "Failed to write {}: {}", about_out_txt.display(), err));
    about_out_txt
}

fn reprocess(text: &str) -> String {
    let mut blank_in_a_row = 0;
    let mut lines = text.lines().map(|line| line
        .trim_end_matches(|ch| " \t\r\n".find(ch).is_some()) // Only trim basic ASCII whitespace, not NBSP
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&copy;", "(c)")
    ).filter(|line|{
        if line == "" {
            blank_in_a_row += 1;
        } else {
            blank_in_a_row = 0;
        }
        blank_in_a_row < 2
    }).collect::<Vec<String>>();

    while let Some(line) = lines.last() {
        if line == "" {
            lines.pop();
        } else {
            break;
        }
    }

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
                let tab = line.find('\t').unwrap_or_else(|| fatal!(bug, "Markdown table line missing tabs after previous enumeration found tabs"));
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

fn get_about_toml_dir() -> PathBuf {
    let (workspace_dir, crate_dir) = (&*WORKSPACE_DIR, &*CARGO_MANIFEST_DIR);
    match (features::ABOUT_PER_WORKSPACE, features::ABOUT_PER_CRATE) {
        (true,  false) => workspace_dir.clone(),
        (false, true ) => crate_dir.clone(),
        (true,  true ) => fatal!(user, "The \"about-per-crate\" and \"about-per-workspace\" features were enabled"),
        (false, false) => {
            if workspace_dir != crate_dir {
                fatal!(user, "The workspace path doesn't match the crate path, so you must specify the \"about-per-crate\" or \"about-per-workspace\" feature.");
            }
            workspace_dir.clone()
        },
    }
}




fn cmd(args_str: &str) -> Command {
    let wd = get_about_toml_dir();
    let mut args = args_str.split_whitespace();
    let exe = args.next().unwrap_or_else(|| fatal!(bug, "cmd expected an exe: {:?}", args_str));
    let mut cmd = Command::new(exe);
    cmd.current_dir(wd);
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
    std::env::var_os(name).unwrap_or_else(||{
        if cfg!(windows) {
            fatal!(system, "%{}%: Not set", name);
        } else {
            fatal!(system, "${{{}}}: Not set", name);
        }
    })
}
