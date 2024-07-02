// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;
use regex::Regex;
use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;

mod dlwrap;
use dlwrap::Builder;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(about = "Generate function list from header")]
struct Cli {
    /// Path to input file
    #[arg(short, long)]
    input: PathBuf,

    /// Path to output directory
    #[arg(short, long)]
    output_dir: Option<PathBuf>,

    /// Resource directory to clang
    #[arg(long)]
    clang_resource_dir: Option<PathBuf>,

    /// Symbol to match
    #[arg(long, conflicts_with = "symbol_file")]
    symbol: Vec<String>,

    /// File listing symbol names
    #[arg(short, long)]
    symbol_file: Option<PathBuf>,

    /// Pattern to match symbol
    #[arg(long)]
    symbol_regex: Vec<Regex>,

    /// Basename of the loader module
    #[arg(long)]
    loader_basename: Option<String>,

    /// Library prefix
    #[arg(long)]
    prefix: Option<String>,

    /// Symbol prefix
    #[arg(long)]
    symbol_prefix: Option<String>,

    /// Function prefix
    #[arg(long)]
    function_prefix: Option<String>,

    /// Name of the library soname macro
    #[arg(long)]
    soname: Option<String>,

    /// Name of the function wrapper macro
    #[arg(long)]
    function_wrapper: Option<String>,

    /// Additional header files to include
    #[arg(long)]
    include: Vec<String>,

    /// License of the input file
    #[arg(long, conflicts_with = "license_file")]
    license: Option<String>,

    /// File containing license of the input file
    #[arg(long)]
    license_file: Option<PathBuf>,

    /// Name of the header guard macro
    header_guard: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut builder = Builder::new(&cli.input);

    if let Some(ref output_dir) = cli.output_dir {
        builder.output_dir(output_dir);
    }

    if let Some(ref clang_resource_dir) = cli.clang_resource_dir {
        builder.clang_resource_dir(clang_resource_dir);
    }

    for symbol in cli.symbol {
        builder.symbol(&symbol);
    }

    if let Some(ref symbol_file) = cli.symbol_file {
        let f = fs::File::open(symbol_file)?;
        for line in io::BufReader::new(f).lines() {
            builder.symbol(&line?);
        }
    }

    for symbol_regex in cli.symbol_regex {
        builder.symbol_regex(&symbol_regex);
    }

    if let Some(ref loader_basename) = cli.loader_basename {
        builder.loader_basename(loader_basename);
    }

    if let Some(ref prefix) = cli.prefix {
        builder.prefix(prefix);
    }

    if let Some(ref symbol_prefix) = cli.symbol_prefix {
        builder.symbol_prefix(symbol_prefix);
    }

    if let Some(ref function_prefix) = cli.function_prefix {
        builder.function_prefix(function_prefix);
    }

    if let Some(ref soname) = cli.soname {
        builder.soname(soname);
    }

    if let Some(ref function_wrapper) = cli.function_wrapper {
        builder.function_wrapper(function_wrapper);
    }

    for include in cli.include {
        builder.include(&include);
    }

    if let Some(ref license) = cli.license {
        builder.license(license);
    } else if let Some(ref license_file) = cli.license_file {
        let license = fs::read_to_string(license_file)?;
        builder.license(&license);
    }

    if let Some(ref header_guard) = cli.header_guard {
        builder.header_guard(header_guard);
    }

    builder.generate()
}
