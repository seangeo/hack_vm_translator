use std::env;
use std::process;
use std::fs;
use std::error::Error;

mod vm;

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

    println!("source file = {}", source_file);
    println!("ast = {:#?}", ast);

    Ok(())
}
