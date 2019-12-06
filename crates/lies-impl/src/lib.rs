//! [lies](https://docs.rs/lies/) implementation details.

extern crate proc_macro;

use proc_macro_hack::*;
use proc_macro::*;

use std::ffi::*;
use std::fs::*;
use std::io::{self, Write};
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
    let rev = "743f3c99c673e29ef0f011fb4d98830d4d25a620";
    let cargo_about = ensure_cargo_about_installed(rev);
    ensure_about_toml_exists();

    let tmp_template_path = std::env::temp_dir().join(format!("{}-{}-{}",
        PathBuf::from(get_env_os("CARGO_PKG_NAME"   )).display(),
        PathBuf::from(get_env_os("CARGO_PKG_VERSION")).display(),
        input_name
    ));

    File::create(&tmp_template_path)
        .expect("Unable to create temporary .hbs file")
        .write_all(input_text)
        .expect("Unable to write entire temporary .hbs file");

    let output = cmd_output(format!(
        "{} about generate {}",
        cargo_about.display(),
        tmp_template_path.display()
    ).as_str()).expect("Failed to 'cargo about generate ...'");

    let output = reprocess(output.as_str());

    TokenStream::from(TokenTree::Literal(Literal::string(output.as_str())))
}

fn ensure_cargo_about_installed(rev: &str) -> PathBuf {
    let root            = [std::env::temp_dir().as_path(), Path::new(rev)].iter().collect::<PathBuf>();
    let expected_path   = [root.as_path(), Path::new("bin"), Path::new("cargo-about")].iter().collect::<PathBuf>();
    let version = cmd_output(format!("{} about --version", expected_path.display()).as_str()).ok();
    let version = version.as_ref().and_then(|output|{
        let ws = output.find(' ')?;
        let (_name, version) = output.split_at(ws);
        Some(version.trim()) // leading ' ', trailing '\n'
    });

    if version.map(|v| v < "0.0.1").unwrap_or(true) {
        eprintln!("Installing cargo-about {}", rev);
        cmd_run(format!("cargo install --git https://github.com/EmbarkStudios/cargo-about.git --rev {} --root {}", rev, root.display()).as_str()).expect("Failed to install cargo-about 0.0.1");
    }

    expected_path
}

fn ensure_about_toml_exists() {
    let path = PathBuf::from(get_env_os("CARGO_MANIFEST_DIR")).join("about.toml");
    if !path.exists() {
        let mut about = File::create(path).expect("about.toml does not exist, and cannot be opened for writing");
        about.write_all(include_bytes!("../templates/about.toml")).expect("Created but failed to fully write out about.toml");
    }
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
    let wd = get_env("CARGO_MANIFEST_DIR");
    let mut args = args.split_whitespace();
    let exe = args.next().expect("cmd:  Expected a command");
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
        Err(io::Error::new(io::ErrorKind::Other, format!("Failed to successfully run \"{}\": {:?}", args, output.status)))
    } else {
        String::from_utf8(output.stdout).map_err(|err| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{:?} output invalid UTF8: {}", args, err)
        ))
    }
}

fn get_env_os(name: &str) -> OsString {
    match std::env::var_os(name) {
        Some(v) => v,
        None => {
            panic!("{}: Not set", name);
        },
    }
}

fn get_env(name: &str) -> String {
    match std::env::var(name) {
        Ok(v) => v,
        Err(err) => {
            panic!("{}: {}", name, err);
        },
    }
}
