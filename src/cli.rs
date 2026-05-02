// This file is part of the uutils awk package.
//
// For the full copyright and license information, please view the LICENSE
// files that was distributed with this source code.

use std::{ffi::OsString, path::PathBuf};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version, name = "uutils AWK")]
#[clap(about = ::std::concat!("uutils awk ", ::std::env!("CARGO_PKG_VERSION")))]
pub struct Args {
    // POSIX
    pub code: OsString,
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,
    #[arg(short = 'F', long)]
    field_separator: Option<OsString>,
    #[arg(short = 'v', long, value_parser = parse_kv)]
    assign: Vec<(String, String)>,
    #[arg(short = 'b', long)]
    characters_as_bytes: bool,
    #[arg(short = 'c', long)]
    traditional: bool,
    #[arg(short = 'C', long)]
    copyright: bool,
    #[arg(short = 'd', long)]
    dump_variables: Option<PathBuf>,
    #[arg(short = 'D', long)]
    debug: Option<PathBuf>,
    #[arg(short = 'e', long)]
    source: Vec<u8>,
    #[arg(short = 'E', long)]
    exec: Option<PathBuf>,
    #[arg(short = 'g', long)]
    gen_pot: bool,
    #[arg(short = 'i', long)]
    include: Option<PathBuf>,
    #[arg(short = 'I', long)]
    trace: bool,
    #[arg(short = 'l', long)]
    load: Vec<OsString>,
    #[arg(short = 'L', long)]
    lint: Vec<String>,
    #[arg(short = 'M', long)]
    bignum: bool,
    #[arg(short = 'n', long)]
    non_decimal_data: bool,
    #[arg(short = 'N', long)]
    use_lc_numeric: bool,
    #[arg(short = 'o', long)]
    pretty_print: Option<PathBuf>,
    #[arg(short = 'O', long, default_value_t = true)]
    optimize: bool,
    #[arg(short = 's', long = "no-optimize")]
    no_optimize: bool,
    #[arg(short = 'p', long)]
    profile: Option<PathBuf>,
    #[arg(short = 'P', long)]
    posix: bool,
    #[arg(short = 'r', long, default_value_t = true)]
    re_interval: bool,
    #[arg(short = 'S', long)]
    sandbox: bool,
    #[arg(short = 't', long)]
    lint_old: bool,
}

fn parse_kv(s: &str) -> Result<(String, String), String> {
    let (k, v) = s.split_once('=').ok_or("expected key=value")?;
    Ok((k.to_string(), v.trim_matches(['"', '\'']).to_string()))
}
