// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clang::*;
use clap::Parser;
use regex::{Captures, Regex};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(about = "Generate function list from header")]
struct Cli {
    /// Path to input file
    #[arg(short, long)]
    input: PathBuf,

    /// Path to output directory
    #[arg(short, long)]
    output: PathBuf,

    /// Pattern to match function name
    #[arg(long)]
    function_regex: Option<Regex>,

    /// File listing matching functions
    #[arg(short, long)]
    function_list: Option<PathBuf>,

    /// Basename of the loader module
    #[arg(long, default_value = "loader")]
    loader: String,

    /// Library prefix
    #[arg(long)]
    library_prefix: String,

    /// Symbol prefix
    #[arg(long)]
    symbol_prefix: String,

    /// Function prefix
    #[arg(long)]
    function_prefix: String,

    /// Library soname
    #[arg(long)]
    soname: String,

    /// Name of the wrapper macro
    #[arg(long)]
    wrapper: String,

    #[arg(long)]
    header: Vec<String>,
}

const LOADER_C_TEMPLATE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/loader.c.template"));
const LOADER_H_TEMPLATE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/loader.h.template"));

fn write_functions(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    patterns: &[Regex],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut functions = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(output.as_ref())?;
    
    let clang = Clang::new()?;
    let index = Index::new(&clang, false, false);
    let tu = index.parser(input.as_ref()).parse()?;
    let funcs = tu.get_entity().get_children().into_iter().filter(|e| {
        e.get_kind() == EntityKind::FunctionDecl
    }).collect::<Vec<_>>();
    for func in funcs {
        let name = func.get_name().unwrap();
        if !patterns.iter().any(|pattern| pattern.is_match_at(&name, 0)) {
            continue;
        }

        let result_type = func.get_result_type().unwrap();
        let args = func.get_arguments().unwrap().into_iter().map(|arg| {
            let type_ = arg.get_type().unwrap().get_display_name();
            let delim = if type_.ends_with("*") { "" } else { " " };
            format!("{}{}{}",
                    type_,
                    delim,
                    arg.get_display_name().unwrap())
        }).collect::<Vec<_>>();
        let cargs = func.get_arguments().unwrap().into_iter().map(|arg| {
            format!("{}",
                    arg.get_display_name().unwrap())
        }).collect::<Vec<_>>();
        let macro_ = if result_type.get_kind() == TypeKind::Void {
            "VOID_FUNC"
        } else {
            "FUNC"
        };
        writeln!(&mut functions, "{}({}, {}, ({}), ({}))",
                 macro_,
                 result_type.get_display_name(),
                 name,
                 args.join(", "),
                 cargs.join(", "))?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    fs::create_dir_all(&cli.output)?;

    let mut patterns = vec![];

    if let Some(ref regex) = cli.function_regex {
        patterns.push(regex.clone());
    }

    if let Some(ref path) = cli.function_list {
        let mut function_list = String::from_utf8_lossy(&fs::read(path)?)
            .split("\n")
            .map(|line| Regex::new(&regex::escape(line)).map_err(Into::into))
            .collect::<Result<Vec<_>>>()?;
        patterns.append(&mut function_list);
    }

    let re = Regex::new("@(.*?)@")?;
    let includes = cli.header.iter().map(|h| format!("#include <{}>", h)).collect::<Vec<_>>().join("\n");
    let loader_h = format!("{}.h", &cli.loader);
    let functions_h = format!("{}funcs.h", &cli.loader);
    let loader_uppercase = cli.loader.to_uppercase().replace(|c: char| {!(c.is_ascii_alphanumeric() || c == '_')}, "_");
    let guard = format!("{}_H_", &loader_uppercase);
    let enable_dlopen = format!("{}_ENABLE_DLOPEN", &cli.library_prefix.to_uppercase());
    let enable_pthread = format!("{}_ENABLE_PTHREAD", &cli.library_prefix.to_uppercase());

    let replacement = |caps: &Captures| {
        match &caps[1] {
            "" => "@@",
            "SYMBOL_PREFIX" => &cli.symbol_prefix,
            "FUNCTION_PREFIX" => &cli.function_prefix,
            "FUNCTIONS_H" => &functions_h,
            "LIBRARY_SONAME" => &cli.soname,
            "WRAPPER" => &cli.wrapper,
            "ENABLE_DLOPEN" => &enable_dlopen,
            "ENABLE_PTHREAD" => &enable_pthread,
            "LIBRARY_PREFIX" => &cli.library_prefix,
            "INCLUDES" => &includes,
            "LOADER_H" => &loader_h,
            "GUARD" => &guard,
            _ => unreachable!(),
        }
    };

    let mut loader_c = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&cli.output.join(format!("{}.c", &cli.loader)))?;

    let loader_c_content = re.replace_all(LOADER_C_TEMPLATE, replacement);
    loader_c.write_all(loader_c_content.into_owned().as_bytes())?;

    let mut loader_h = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&cli.output.join(format!("{}.h", &cli.loader)))?;

    let loader_h_content = re.replace_all(LOADER_H_TEMPLATE, replacement);
    loader_h.write_all(loader_h_content.into_owned().as_bytes())?;

    write_functions(
        &cli.input,
        &cli.output.join(&functions_h),
        &patterns,
    )?;

    Ok(())
}
