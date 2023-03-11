use std::collections::HashMap;
use serde_derive::Deserialize;
use std::fs::File;
use std::io::{Read, stdin};

const ISA_VALIDATION_ERR_MSG: &str = "Invalid ISA file structure";
const ISA_READ_ERR_MSG: &str = "Couldn't read ISA file";
const ASM_ERR_MSG: &str = "Couldn't read ASM file";

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct CpuData {
    cpu_name: String,
    instruction_length: usize,
    program_memory_lines: usize
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Instruction {
    opcode: String,
    operands: Vec<String>,
    keywords: Vec<String>
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ISA {
    cpu_data: CpuData,
    define: HashMap<String, HashMap<String, String>>,
    instructions: HashMap<String, Instruction>
}

struct Error {
    file: String,
    line: Option<u32>,
    message: String
}

impl Error {
    fn no_line(file: &String, message: String) -> Error {
        Error {
            file: file.to_string(), line: None, message
        }
    }

    fn in_line(file: &String, line: &usize, message: &String) -> Error {
        Error {
            file: file.to_string(), line: Some(*line as u32), message: message.to_string()
        }
    }
}

struct AssemblerResult{
    info: Vec<String>,
    fails: Vec<Error>
}

impl AssemblerResult {
    fn report(&self) {
        match self.fails.len(){
            0 => {
                for element in &self.info {
                    println!("{}", element);
                }
            }
            _ =>
                for element in &self.fails {
                    match element.line {
                        Some(nr) => println!(r#"Error in file "{}", line {}: {}"#, element.file, nr + 1, element.message),
                        None => println!(r#"Error in file "{}": {}"#, element.file, element.message)
                    }
                }
        };
    }
}

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

    match serde_json::from_str(&contents) {
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
    match file.read_to_string(&mut contents) {
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
         assembler_result: &mut AssemblerResult) {

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

        if assembler_result.fails.len() != 0 {
            assembler_result.report();
            continue;
        }

        let isa = isa.unwrap();
        parse(&isa, &isa_file_name, &asm, &asm_file_name, &mut assembler_result);

        if assembler_result.fails.len() != 0 {
            assembler_result.report();
            continue;
        }

        assembler_result.report();
    }
}
