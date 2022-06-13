use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

pub mod asm;
pub mod vm;

fn parse_args(args: &[String]) -> Result<String, &'static str> {
    if args.len() > 1 {
        Ok(args[1].clone())
    } else {
        Err("not enough arguments")
    }
}

fn list_files(path: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();

    if path.is_file() {
        files.push(path.to_path_buf());
    } else if path.is_dir() {
        let paths = fs::read_dir(path).unwrap();

        for path in paths {
            let path = path.unwrap().path();
            if let Some(ext) = path.extension() {
                if ext == "vm" {
                    files.push(path);
                }
            }
        }
    }

    files
}

fn load_sources(path: &Path) -> Result<Vec<(String, String)>, String> {
    list_files(path)
        .into_iter()
        .map(|file| {
            let name = file.file_stem().unwrap().to_str().unwrap().to_string();
            println!("Reading file {}", file.display());
            match fs::read_to_string(file) {
                Ok(s) => Ok((name, s)),
                Err(e) => Err(format!("Error reading file: {e}")),
            }
        })
        .collect()
}

fn parse_sources<'a>(
    sources: &'a Vec<(String, String)>,
) -> Vec<Result<vm::SourceCommand<'a>, String>> {
    sources
        .into_iter()
        .flat_map(|(file, source)| vm::parse_source(file, &source))
        .collect()
}

fn extract_and_report_errors(
    parse_results: Vec<Result<vm::SourceCommand, String>>,
) -> Result<Vec<vm::SourceCommand>, String> {
    let mut error_count = 0;
    let mut parsed_commands: Vec<vm::SourceCommand> = Vec::new();

    for result in parse_results {
        match result {
            Ok(c) => parsed_commands.push(c),
            Err(e) => {
                error_count = error_count + 1;
                println!("{}", e)
            }
        }
    }

    if error_count > 0 {
        Err(format!("Parse errors found: {error_count}"))
    } else {
        Ok(parsed_commands)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let source = parse_args(&args).unwrap_or_else(|err| {
        println!("Argument Error: {}", err);
        println!("Usage: hack_vmtranslator <vmfile|directory>");
        process::exit(1);
    });

    let source_path = Path::new(&source);
    let sources = load_sources(&source_path)?;
    let ast = parse_sources(&sources);
    let ast = extract_and_report_errors(ast)?;
    let asm = asm::generate_code(ast)?;

    println!("source file = {}", source);

    let target_file_name = if source_path.is_file() {
        source_path.with_extension("asm")
    } else {
        let base_name = source_path.file_stem().unwrap();
        source_path.join(PathBuf::from(base_name).with_extension("asm"))
    };
    println!("output file = {}", target_file_name.to_str().unwrap());
    fs::write(target_file_name, asm.join("\n"))?;

    Ok(())
}
