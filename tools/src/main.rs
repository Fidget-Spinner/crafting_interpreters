#[allow(unused_imports)]
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: generate_ast <output directory>");
        process::exit(64);
    }
    let output_dir = &args[0];
    define_ast(
        output_dir,
        "expr",
        vec![
            "Binary   : Expr left, Token operator, Expr right",
            "Grouping : Expr expression",
            "Literal  : Object value",
            "Unary    : Token operator, Expr right",
        ],
    );
}

fn define_ast(
    output_dir: &String,
    base_name: &'static str,
    types: Vec<&'static str>,
) -> Result<(), std::io::Error> {
    let path: String = format!("{}/{}.java", output_dir, base_name);
    let mut writer = File::create(path)?;
    writer.write_fmt(format_args!(
        "mod {} {{\n",
        str::from_utf8(base_name).unwrap()
    ));
    // The AST classes
    for type_ in types {
        let mut split = type_.split(":").collect::<Vec<_>>();
        let (class_name, fields) = (split[0].trim(), split[1].trim());
        define_type(&mut writer, base_name, class_name, fields);
    }
    writer.write(b"}\n");

    Ok(())
}

fn define_type(writer: &mut File, base_name: &'static str, class_name: &str, fields: &str) {
    writer.write_fmt(format_args!(""));
}
