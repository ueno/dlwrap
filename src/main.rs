// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;
use regex::Regex;
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
    #[arg(long)]
    symbol: Vec<String>,

    /// Pattern to match symbol
    #[arg(long)]
    symbol_regex: Vec<Regex>,

    /// File listing symbol names
    #[arg(short, long)]
    symbol_list: Option<PathBuf>,

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

    for symbol_regex in cli.symbol_regex {
        builder.symbol_regex(&symbol_regex);
    }

    if let Some(ref symbol_list) = cli.symbol_list {
        builder.symbol_list(symbol_list);
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

    builder.generate()
}
