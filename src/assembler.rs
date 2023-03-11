mod structs;

#[macro_use]
mod macros;

use structs::*;
use macros::*;

use std::fs::File;
use std::io::{Read, stdin};

const ISA_VALIDATION_ERR_MSG: &str = "Invalid ISA file structure";
const ISA_READ_ERR_MSG: &str = "Couldn't read ISA file";
const ASM_ERR_MSG: &str = "Couldn't read ASM file";

fn deserialize_json_file(file_name: &String) -> Result<ISA, String> {
    let mut file = match File::open(file_name) {
        Ok(v) => v,
        Err(_) => return Err(ISA_READ_ERR_MSG.to_string())
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(_) => return Err(ISA_READ_ERR_MSG.to_string())
    }

    return match serde_json::from_str(&contents) {
        Ok(v) => Ok(v),
        Err(_) => Err(ISA_VALIDATION_ERR_MSG.to_string())
    }
}

fn read_assembly(file_name: &String) -> Result<String, String> {
    let mut file = match File::open(file_name) {
        Ok(v) => v,
        Err(_) => return Err(ASM_ERR_MSG.to_string())
    };

    let mut contents = String::new();
    return match file.read_to_string(&mut contents) {
        Ok(_) => Ok(contents),
        Err(_) => Err(ASM_ERR_MSG.to_string())
    }
}

fn open_files(isa: &mut Option<ISA>, mut isa_file_name: &mut String, asm: &mut String,
              mut asm_file_name: &mut String, assembler_result: &mut AssemblerResult) {

    println!("\nISA file name: ");
    stdin().read_line(&mut isa_file_name).unwrap();
    *isa_file_name = "ISA/".to_string() + &isa_file_name[0..&isa_file_name.len() - 1].to_string();
    let isa_result = deserialize_json_file(&isa_file_name);

    match isa_result {
        Ok(v) => {
            assembler_result.info.push(v.cpu_data.cpu_name.clone());
            assembler_result.info.push("------------------------".to_string());
            *isa = Some(v);
        }
        Err(e) => {
            assembler_result.fails.push(Error::no_line(&isa_file_name, e));
            *isa = None;
        }
    }

    println!("ASM file name: ");
    stdin().read_line(&mut asm_file_name).unwrap();
    *asm_file_name = "ASM/".to_string() + &asm_file_name[0..&asm_file_name.len() - 1].to_string();
    let asm_result = read_assembly(&asm_file_name);

    match asm_result {
        Ok(v) => *asm = v,
        Err(e) => assembler_result.fails.push(Error::no_line(&asm_file_name, e))
    }
}

fn parse(isa: &ISA, isa_file_name: &String, asm: &String, asm_file_name: &String,
         assembler_result: &mut AssemblerResult) -> String {

    let out = String::new();

    let mut line_counter = 0;
    let lines: Vec<Line> = asm.split("\n")
        .map(|x| Line::new(x.to_string(), &mut line_counter))
        .collect();

    for line in lines {
        println!("{:?}", line);
        for (idx, token) in line.tokens.iter().enumerate() {

        }
    }

    return out;
}

pub fn assemble() {
    loop {
        let mut assembler_result = AssemblerResult {
            info: Vec::new(),
            fails: Vec::new()
        };

        let mut isa = None;
        let mut isa_file_name = String::new();

        let mut asm = String::new();
        let mut asm_file_name = String::new();

        open_files(&mut isa, &mut isa_file_name, &mut asm, &mut asm_file_name, &mut assembler_result);
        continue_on_err!(assembler_result);

        let isa = isa.unwrap();

        let bin = parse(&isa, &isa_file_name, &asm, &asm_file_name, &mut assembler_result);
        continue_on_err!(assembler_result);

        assembler_result.report();
    }
}
