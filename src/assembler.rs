mod structs;

#[macro_use]
mod macros;

use structs::*;

use std::fs;
use std::io::{Read, stdin};
use std::env;
use std::path::Path;
use crate::assembler::structs::Kind::Operand;

const ISA_VALIDATION_ERR_MSG: &str = "Invalid ISA file structure";
const ISA_READ_ERR_MSG: &str = "Couldn't read ISA file";
const ASM_ERR_MSG: &str = "Couldn't read ASM file";

const RESERVED_ISA: &str = "#isa";
const RESERVED_DEFINE: &str = "#define";
const RESERVED_LABEL: &str = ".";

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

fn open_files(isa: &mut Option<ISA>, mut isa_file_name: &mut String, asm: &mut String,
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
        .filter(|x| x.starts_with(RESERVED_ISA))
        .map(|x| x.to_string())
        .collect();

    if isa_declarations.len() == 1 {
        let isa_file_name_option = isa_declarations.iter().find(|x| x.starts_with("#isa "));
        match isa_file_name_option {
            Some(v) => {
                let isa_file_name_no_prefix = &(v[4..].trim().to_string() + ".json");
                match deserialize_json_file(&("ISA/".to_string() + isa_file_name_no_prefix)) {
                    Ok(v) => {
                        assembler_result.info.push(v.cpu_data.cpu_name.clone());
                        assembler_result.info.push("------------------------".to_string());
                        *isa = Some(v);
                        *isa_file_name = isa_file_name_no_prefix.to_string();
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
    } else {
        assembler_result.fails.push(Error::no_line(&asm_file_name, "Single ISA declaration required".to_string()));
    }
}

fn parse(isa: &ISA, isa_file_name: &String, asm: &String, asm_file_name: &String, assembler_result: &mut AssemblerResult) -> String {
    let mut out = String::new();
    let mut line_counter = 0;
    let expected_line_length = isa.cpu_data.instruction_length;

    let define_declarations: Vec<String> = asm.split("\n")
        .filter(|x| x.starts_with(RESERVED_DEFINE))
        .map(|x| x.to_string())
        .collect();

    let label_declarations: Vec<String> = asm.split("\n")
        .filter(|x| x.starts_with(RESERVED_LABEL))
        .map(|x| x.to_string())
        .collect();

    let lines: Vec<Line> = asm.split("\n")
        .map(|x| Line::new(x.to_string(), &mut line_counter))
        .collect();

    for (line_nr, line) in lines.iter().enumerate() {
        let mut out_line = String::new();
        let mnemonic: String;

        if line.tokens.len() == 0 || line.tokens[0].content.starts_with(RESERVED_ISA) || line.tokens[0].content.starts_with(RESERVED_DEFINE) || line.tokens[0].content.starts_with(RESERVED_LABEL) {
            continue;
        } else {
            mnemonic = line.tokens[0].content.clone();
        }

        if isa.instructions.contains_key(&mnemonic) {
            let expected = isa.instructions[&mnemonic].layout.iter();
            let expected_operands_len = expected.clone().filter(|x| matches!(x, Kind::Operand(_))).count();

            let mut opcode_found = false;

            let mut operands: Vec<Token> = line.tokens[1..].to_vec();
            let provided_operands_len = operands.len();

            let mut nr_handled_operands: usize = 0;

            for item in expected {
                match item {
                    Kind::Opcode(opcode) => {
                        if opcode_found == false {
                            out_line += opcode;
                            opcode_found = true;
                        } else {
                            assembler_result.fails.push(Error::no_line(isa_file_name, format!(r#"Instruction "{}" was configured to expect more than one opcode"#, mnemonic)));
                        }
                    },
                    Kind::Operand(operand_length) => {
                        println!("{} {} {}", operands.len(), nr_handled_operands, expected_operands_len);
                        if nr_handled_operands < expected_operands_len && operands.len() > 0 {
                            let mut operand: usize = 0;
                            let provided_operand = operands.remove(0).content;
                            match provided_operand.parse() {
                                Ok(v) => operand = v,
                                Err(_) => {
                                    assembler_result.fails.push(Error::in_line(asm_file_name, &line_nr, format!(r#"Failed to parse token "{}""#, provided_operand)));
                                }
                            };
                            let bin_operand = format!("{operand:b}");
                            out_line += &format!("{bin_operand:0>0$}", operand_length);
                            nr_handled_operands += 1;
                        }
                        else if nr_handled_operands < expected_operands_len && operands.len() == 0 {
                            assembler_result.fails.push(Error::in_line(asm_file_name, &line_nr, format!("Too few operands - expected {}, found {}", expected_operands_len, provided_operands_len)));
                        }
                    },
                    Kind::Filler(filler_char, filler_length) => {
                        out_line += &(0..*filler_length).map(|_| filler_char).collect::<String>();
                    },
                    _ => {
                        assembler_result.fails.push(Error::in_line(asm_file_name, &line_nr,
                        format!(r#"Token/tokens failed to match with any token type"#)));
                    }
                }
            }
            if operands.len() > 0 {
                assembler_result.fails.push(Error::in_line(asm_file_name, &line_nr, format!("Too many operands - expected {}, found {}", expected_operands_len, provided_operands_len)));
            }
            match out_line.len() == expected_line_length {
                true => {
                    println!("{}", out_line);
                    out_line += "\n";
                }
                false => assembler_result.fails.push(Error::in_line(&asm_file_name, &line_nr, format!("Operand exceeded max binary value")))
            }
        } else {
            assembler_result.fails.push(Error::in_line(&asm_file_name, &line_nr, format!(r#"Unknown instruction mnemonic "{}""#, mnemonic)));
        }
        out += &out_line;
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
