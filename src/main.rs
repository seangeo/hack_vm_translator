use std::env;
use std::error::Error;
use std::fs;
use std::process;
use std::path::Path;

pub mod vm;
pub mod asm;

fn parse_args(args: &[String]) -> Result<String, &'static str> {
    if args.len() > 1 {
        Ok(args[1].clone())
    } else {
        Err("not enough arguments")
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let source_file = parse_args(&args).unwrap_or_else(|err| {
        println!("Argument Error: {}", err);
        println!("Usage: hack_vmtranslator <vmfile>");
        process::exit(1);
    });
    let vm_source = fs::read_to_string(&source_file)?;
    let ast = vm::parse_source(&vm_source)?;
    let asm = asm::generate_code(ast)?;

    println!("source file = {}", source_file);

    let target_file_name = Path::new(&source_file).with_extension("asm");
    println!("output file = {}", target_file_name.to_str().unwrap());
    fs::write(target_file_name, asm.join("\n"))?;

    Ok(())
}
