// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use clang::*;
use regex::{Captures, Regex};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// A builder for generating a bindings header
#[derive(Default)]
pub struct Builder {
    input: PathBuf,
    output_dir: Option<PathBuf>,
    clang_resource_dir: Option<PathBuf>,
    symbol_regex: Vec<Regex>,
    symbol_list: Option<PathBuf>,
    loader_basename: Option<String>,
    prefix: Option<String>,
    symbol_prefix: Option<String>,
    function_prefix: Option<String>,
    soname: Option<String>,
    function_wrapper: Option<String>,
    include: Vec<String>,
}

const LOADER_C_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/loader.c.template"
));
const LOADER_H_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/loader.h.template"
));

fn write_functions(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    clang_resource_dir: &Option<PathBuf>,
    patterns: &[Regex],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut functions = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output.as_ref())?;

    let clang = Clang::new()?;
    let index = Index::new(&clang, false, false);
    let mut parser = index.parser(input.as_ref());
    if let Some(resource_dir) = clang_resource_dir {
        parser.arguments(&["-resource-dir", resource_dir.to_str().unwrap()]);
    }
    let tu = parser.parse()?;
    let funcs = tu
        .get_entity()
        .get_children()
        .into_iter()
        .filter(|e| e.get_kind() == EntityKind::FunctionDecl)
        .collect::<Vec<_>>();
    for func in funcs {
        let name = func.get_name().unwrap();
        if !patterns.iter().any(|pattern| pattern.is_match_at(&name, 0)) {
            continue;
        }

        let result_type = func.get_result_type().unwrap();
        let args = func
            .get_arguments()
            .unwrap()
            .into_iter()
            .map(|arg| {
                let type_ = arg.get_type().unwrap().get_display_name();
                let delim = if type_.ends_with('*') { "" } else { " " };
                format!("{}{}{}", type_, delim, arg.get_display_name().unwrap())
            })
            .collect::<Vec<_>>();
        let cargs = func
            .get_arguments()
            .unwrap()
            .into_iter()
            .map(|arg| arg.get_display_name().unwrap().to_string())
            .collect::<Vec<_>>();
        let macro_ = if result_type.get_kind() == TypeKind::Void {
            "VOID_FUNC"
        } else {
            "FUNC"
        };
        writeln!(
            &mut functions,
            "{}({}, {}, ({}), ({}))",
            macro_,
            result_type.get_display_name(),
            name,
            if args.is_empty() {
                "void".to_string()
            } else {
                args.join(", ")
            },
            cargs.join(", ")
        )?;
    }

    Ok(())
}

impl Builder {
    /// Create a new builder
    pub fn new(input: impl AsRef<Path>) -> Self {
        Self {
            input: input.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    /// Set path to output directory
    pub fn output_dir(&mut self, output_dir: impl AsRef<Path>) -> &mut Self {
        self.output_dir = Some(output_dir.as_ref().to_path_buf());
        self
    }

    /// Set resource directory to clang
    pub fn clang_resource_dir(&mut self, clang_resource_dir: impl AsRef<Path>) -> &mut Self {
        self.clang_resource_dir = Some(clang_resource_dir.as_ref().to_path_buf());
        self
    }

    /// Set pattern to match symbol
    pub fn symbol_regex(&mut self, symbol_regex: &Regex) -> &mut Self {
        self.symbol_regex.push(symbol_regex.to_owned());
        self
    }

    /// Set file listing symbol names
    pub fn symbol_list(&mut self, symbol_list: impl AsRef<Path>) -> &mut Self {
        self.symbol_list = Some(symbol_list.as_ref().to_path_buf());
        self
    }

    /// Set basename of the loader module
    pub fn loader_basename(&mut self, loader_basename: &str) -> &mut Self {
        self.loader_basename = Some(loader_basename.to_owned());
        self
    }

    /// Set library prefix
    pub fn prefix(&mut self, prefix: &str) -> &mut Self {
        self.prefix = Some(prefix.to_owned());
        self
    }

    /// Set symbol prefix
    pub fn symbol_prefix(&mut self, symbol_prefix: &str) -> &mut Self {
        self.symbol_prefix = Some(symbol_prefix.to_owned());
        self
    }

    /// Set function prefix
    pub fn function_prefix(&mut self, function_prefix: &str) -> &mut Self {
        self.function_prefix = Some(function_prefix.to_owned());
        self
    }

    /// Set name of the library soname macro
    pub fn soname(&mut self, soname: &str) -> &mut Self {
        self.soname = Some(soname.to_owned());
        self
    }

    /// Set name of the function wrapper macro
    pub fn function_wrapper(&mut self, function_wrapper: &str) -> &mut Self {
        self.function_wrapper = Some(function_wrapper.to_owned());
        self
    }

    /// Set additional header file to include
    pub fn include(&mut self, include: &str) -> &mut Self {
        self.include.push(include.to_owned());
        self
    }

    /// Generate code
    pub fn generate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let output_dir = match self.output_dir {
            Some(ref output_dir) => output_dir.to_path_buf(),
            None => Default::default(),
        };

        fs::create_dir_all(&output_dir)?;

        let input_file_stem = self
            .input
            .file_stem()
            .and_then(|f| f.to_str())
            .unwrap()
            .to_owned();

        let loader_basename = self
            .loader_basename
            .as_ref()
            .unwrap_or_else(|| &input_file_stem);

        let prefix = self.prefix.as_ref().unwrap_or_else(|| &input_file_stem);

        let mut patterns = vec![];

        for symbol_regex in &self.symbol_regex {
            patterns.push(symbol_regex.clone());
        }

        if let Some(ref path) = self.symbol_list {
            let mut function_list = String::from_utf8_lossy(&fs::read(path)?)
                .split('\n')
                .map(|line| Regex::new(&regex::escape(line)).map_err(Into::into))
                .collect::<Result<Vec<_>>>()?;
            patterns.append(&mut function_list);
        }

        if patterns.is_empty() {
            return Err(anyhow!("no symbol patterns").into());
        }

        let loader_c_path = output_dir.join(loader_basename).with_extension("c");
        let loader_h_path = output_dir.join(loader_basename).with_extension("h");

        let re = Regex::new("@(.*?)@")?;

        let includes = self
            .include
            .iter()
            .map(|h| format!("#include {}", h))
            .collect::<Vec<_>>()
            .join("\n");
        let functions_h = format!("{}funcs.h", loader_basename);
        let loader_h_file_name = loader_h_path.file_name().and_then(|f| f.to_str()).unwrap();
        let loader_h_guard = format!(
            "{}_",
            loader_h_file_name
                .to_uppercase()
                .replace(|c: char| !(c.is_ascii_alphanumeric() || c == '_'), "_"),
        );

        let enable_dlopen = format!("{}_ENABLE_DLOPEN", prefix.to_uppercase());
        let enable_pthread = format!("{}_ENABLE_PTHREAD", prefix.to_uppercase());

        let symbol_prefix = self
            .symbol_prefix
            .take()
            .unwrap_or_else(|| format!("{}_sym", prefix.to_lowercase()));

        let function_prefix = self
            .function_prefix
            .take()
            .unwrap_or_else(|| format!("{}_func", prefix.to_lowercase()));

        let soname = self
            .soname
            .take()
            .unwrap_or_else(|| format!("{}_SONAME", prefix.to_uppercase()));

        let function_wrapper = self
            .function_wrapper
            .take()
            .unwrap_or_else(|| format!("{}_FUNC", prefix.to_uppercase()));

        let replacement = |caps: &Captures| match &caps[1] {
            "" => "@",
            "LIBRARY_PREFIX" => prefix,
            "SYMBOL_PREFIX" => &symbol_prefix,
            "FUNCTION_PREFIX" => &function_prefix,
            "FUNCTIONS_H" => &functions_h,
            "LIBRARY_SONAME" => &soname,
            "WRAPPER" => &function_wrapper,
            "ENABLE_DLOPEN" => &enable_dlopen,
            "ENABLE_PTHREAD" => &enable_pthread,
            "INCLUDES" => &includes,
            "LOADER_H" => &loader_h_file_name,
            "LOADER_H_GUARD" => &loader_h_guard,
            _ => unreachable!(),
        };

        let mut loader_c = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&loader_c_path)?;

        let loader_c_content = re.replace_all(LOADER_C_TEMPLATE, replacement);
        loader_c.write_all(loader_c_content.into_owned().as_bytes())?;

        let mut loader_h = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&loader_h_path)?;

        let loader_h_content = re.replace_all(LOADER_H_TEMPLATE, replacement);
        loader_h.write_all(loader_h_content.into_owned().as_bytes())?;

        write_functions(
            &self.input,
            &output_dir.join(&functions_h),
            &self.clang_resource_dir,
            &patterns,
        )?;

        Ok(())
    }
}