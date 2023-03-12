mod structs;

#[macro_use]
mod macros;

use structs::*;

use std::fs;
use std::io::{Read, stdin};
use std::env;
use std::path::Path;

const ISA_VALIDATION_ERR_MSG: &str = "Invalid ISA file structure";
const ISA_READ_ERR_MSG: &str = "Couldn't read ISA file";
const ASM_ERR_MSG: &str = "Couldn't read ASM file";

fn deserialize_json_file(file_name: &String) -> Result<ISA, String> {

    let mut contents = String::new();

    match fs::read_to_string(file_name) {
        Ok(v) => contents = v,
        Err(_) => return Err(ISA_READ_ERR_MSG.to_string())
    }

    return match serde_json::from_str(&contents) {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("{} - {}", ISA_VALIDATION_ERR_MSG.to_string(), e.to_string()))
    }
}

fn read_assembly(file_name: &String) -> Result<String, String> {
    match fs::read_to_string(Path::new(&file_name)) {
        Ok(v) => Ok(v),
        Err(_) => return Err(ASM_ERR_MSG.to_string())
    }
}

fn open_files(isa: &mut Option<ISA>, asm: &mut String,
              mut asm_file_name: &mut String, assembler_result: &mut AssemblerResult) {

    let path = env::current_dir().unwrap();

    println!("ASM file name: ");
    stdin().read_line(&mut asm_file_name).unwrap();
    *asm_file_name = "ASM/".to_string() + &asm_file_name[0..&asm_file_name.len() - 1].to_string().trim();

    let asm_result = read_assembly(&asm_file_name);

    match asm_result {
        Ok(v) => *asm = v,
        Err(e) => assembler_result.fails.push(Error::no_line(&asm_file_name, e))
    }

    let isa_declarations: Vec<String> = asm.split("\n")
        .filter(|x| x.starts_with("#isa "))
        .map(|x| x.to_string())
        .collect();

    if isa_declarations.len() == 1 {
        let isa_file_name = isa_declarations.iter().find(|x| x.starts_with("#isa "));
        match isa_file_name {
            Some(v) => {
                match deserialize_json_file(&("ISA/".to_string() + v[4..].trim() + ".json")) {
                    Ok(v) => {
                        assembler_result.info.push(v.cpu_data.cpu_name.clone());
                        assembler_result.info.push("------------------------".to_string());
                        *isa = Some(v);
                    },
                    Err(e) => {
                        assembler_result.fails.push(Error::no_line(&asm_file_name, e));
                        return;
                    }
                }
            },
            None => {
                assembler_result.fails.push(Error::no_line(&asm_file_name, ISA_READ_ERR_MSG.to_string()));
                return;
            }
        }
    }
    else {
        assembler_result.fails.push(
            Error::no_line(&asm_file_name, "Single ISA declaration required".to_string())
        );
    }
}

fn parse(isa: &ISA, asm: &String, asm_file_name: &String, assembler_result: &mut AssemblerResult) -> String {

    let out = String::new();

    let mut line_counter = 0;
    let lines: Vec<Line> = asm.split("\n")
        .map(|x| Line::new(x.to_string(), &mut line_counter))
        .collect();

    for (idx, line) in lines.iter().enumerate() {
        println!("{:?}", line);

        if line.tokens.len() == 0 {
            continue;
        }

        let mnemonic: String = line.tokens[0].content.clone();
        let operands: Vec<Token> = line.tokens[1..].to_vec();

        if isa.instructions.contains_key(&*mnemonic) {

        }
        else {
            assembler_result.fails.push(Error::in_line(&asm_file_name, &idx,
            &format!("Unknown instruction mnemonic {}", mnemonic)));
        }
    }

    return out;
}

pub fn assemble() {
    let path = env::current_dir().unwrap();
    println!("The current directory is {}", path.display());
    loop {
        let mut assembler_result = AssemblerResult {
            info: Vec::new(),
            fails: Vec::new()
        };

        let mut isa = None;

        let mut asm = String::new();
        let mut asm_file_name = String::new();

        open_files(&mut isa, &mut asm, &mut asm_file_name, &mut assembler_result);
        continue_on_err!(assembler_result);

        let isa = isa.unwrap();

        println!("{:?}", isa);

        let bin = parse(&isa, &asm, &asm_file_name, &mut assembler_result);
        continue_on_err!(assembler_result);

        assembler_result.report();
    }
}
