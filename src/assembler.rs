mod structs;

#[macro_use]
mod macros;

use structs::*;

use std::fs;
use std::io::{stdin};
use std::env;
use std::path::Path;
use crate::assembler::structs::Kind::*;
use std::slice::Chunks;

const ISA_VALIDATION_ERR_MSG: &str = "Invalid ISA file structure";
const ISA_READ_ERR_MSG: &str = "Couldn't read ISA file";
const ASM_ERR_MSG: &str = "Couldn't read ASM file";

const RESERVED_ISA: &str = "#isa";
const RESERVED_DEFINE: &str = "#define";
const RESERVED_LABEL: &str = ".";
const RESERVED_COMMENT: &str = "--";

fn line_separation(c: char) -> bool {
    c == '\n'
}

fn deserialize_json_file(file_name: &String) -> Result<ISA, String> {

    let contents: String;

    match fs::read_to_string(file_name) {
        Ok(v) => contents = v,
        Err(_) => return Err(ISA_READ_ERR_MSG.to_string())
    }
    return match serde_json::from_str(&contents) {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("{} - {}", ISA_VALIDATION_ERR_MSG, e))
    }
}

fn read_assembly(file_name: &String) -> Result<String, String> {
    match fs::read_to_string(Path::new(&file_name)) {
        Ok(v) => Ok(v),
        Err(_) => return Err(ASM_ERR_MSG.to_string())
    }
}

fn open_files(isa: &mut Option<ISA>, isa_file_name: &mut String, asm: &mut String, mut asm_file_name: &mut String,
              label_declarations: &mut Vec<Label>, define_declarations: &mut Vec<DefinePair>, assembler_result: &mut AssemblerResult) {

    println!("==============");
    println!("ASM file name: ");
    stdin().read_line(&mut asm_file_name).unwrap();
    *asm_file_name = "ASM/".to_string() + &asm_file_name[0..&asm_file_name.len() - 1].to_string().trim();

    let asm_result = read_assembly(&asm_file_name);

    match asm_result {
        Ok(v) => *asm = v,
        Err(e) => assembler_result.fails.push(Error::no_line(&asm_file_name, e))
    }

    let asm_lines = asm.split(line_separation);

    let isa_declarations: Vec<String> = asm_lines
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

    let mut true_line_counter: usize = 0;
    let mut current_line: usize = 0;
    let mut instr_line_counter: usize = 0;

    *asm = asm.split(line_separation)
        .map(|x| {
            if x.starts_with(RESERVED_DEFINE) {
                let mut tokens: Vec<String> = x.split(' ')
                    .map(str::trim)
                    .filter(|x| !x.is_empty())
                    .map(str::to_string)
                    .collect();
                match tokens.len() {
                    0..=2 | 4.. => assembler_result.fails.push(
                        Error::in_line(&asm_file_name, &current_line,
                                       r#"#define requires "<keyword> <replacement>" pair"#.to_string())),
                    3 => {
                        tokens.remove(0);
                        if tokens.iter().all(|y| y.chars().all(char::is_alphanumeric)) {
                            define_declarations.push(
                                DefinePair { key: tokens[0].clone(), value: tokens[1].clone() }
                            );
                        } else {
                            assembler_result.fails.push(
                                Error::in_line(&asm_file_name, &current_line,
                                               "Define keywords must be alphanumeric and separated by whitespaces".to_string())
                            );
                        }
                    }
                    _ => assembler_result.fails.push(
                        Error::in_line(&asm_file_name, &true_line_counter,
                                       "Couldn't parse define statement".to_string())
                    )
                }
            } else if x.starts_with('#') && !x.starts_with(RESERVED_ISA) {
                assembler_result.fails.push(Error::in_line(&asm_file_name, &true_line_counter,
                r##"Unexpected "#" found"##.to_string()));
            }
            true_line_counter += 1;
            current_line = true_line_counter;
            x
        })
        .map(str::trim)
        .filter(|x| {
            return if x.starts_with(RESERVED_LABEL) {
                label_declarations.push(Label { identifier: x[1..].to_string(), line_nr: instr_line_counter });
                false
            } else if x.starts_with(RESERVED_DEFINE)
                || x.starts_with(RESERVED_ISA)
                || x.starts_with(RESERVED_LABEL)
                || x.starts_with(RESERVED_COMMENT) {
                false
            } else if x.is_empty() {
                false
            } else {
                instr_line_counter += 1;
                true
            }
        })
        .collect::<Vec<&str>>()
        .join("\n");
}

fn parse(isa: &ISA, isa_file_name: &String, asm: &String, asm_file_name: &String, label_declarations: &Vec<Label>, define_declarations: &mut Vec<DefinePair>, assembler_result: &mut AssemblerResult) -> String {
    let mut out = String::new();
    let mut line_counter = 0;
    let expected_line_length = isa.cpu_data.instruction_length;

    let lines: Vec<Line> = asm.split(line_separation)
        .map(|x| Line::new(x.to_string(), &mut line_counter))
        .map(|mut line| {
            for mut token in &mut line.tokens {
                for label in label_declarations {
                    if token.content == label.identifier {
                        token.content = label.line_nr.to_string();
                    }
                }
            }
            line
        })
        .collect();

    for (line_nr, line) in lines.iter().enumerate() {
        let mut out_line = String::new();
        let mnemonic: String;

        if line.tokens.is_empty() || line.tokens[0].content.starts_with('#') || line.tokens[0].content.starts_with(RESERVED_LABEL) {
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
                    Opcode(opcode) => {
                        if !opcode_found {
                            out_line += opcode;
                            opcode_found = true;
                        } else {
                            assembler_result.fails.push(Error::no_line(isa_file_name, format!(r#"Instruction "{}" was configured to expect more than one opcode"#, mnemonic)));
                            break;
                        }
                    },
                    Operand(operand_length) => {
                        if nr_handled_operands < expected_operands_len && !operands.is_empty() {
                            let operand: usize;
                            let mut provided_operand = operands.remove(0).content;

                            for keyword in isa.instructions[&mnemonic].keywords.iter() {
                                if isa.define.contains_key(keyword) && isa.define[keyword].contains_key(&provided_operand) {
                                    define_declarations.push(
                                        DefinePair { key: provided_operand.to_string(), value: isa.define[keyword][&provided_operand].to_string() }
                                    );
                                }
                            }
                            for pair in define_declarations.iter() {
                                if pair.key == provided_operand {
                                    provided_operand = pair.value.clone();
                                }
                            }

                            match provided_operand.parse() {
                                Ok(v) => operand = v,
                                Err(_) => {
                                    assembler_result.fails.push(Error::in_line(asm_file_name, &line_nr, format!(r#"Failed to parse token "{}""#, provided_operand)));
                                    break;
                                }
                            };

                            let bin_operand = format!("{operand:b}");
                            out_line += &format!("{bin_operand:0>0$}", operand_length);
                            nr_handled_operands += 1;
                        }
                    },
                    Filler(filler_char, filler_length) => {
                        out_line += &(0..*filler_length).map(|_| filler_char).collect::<String>();
                    }
                }
            }
            if nr_handled_operands < expected_operands_len && operands.is_empty() {
                assembler_result.fails.push(Error::in_line(asm_file_name, &line_nr, format!("Too few operands - expected {}, found {}", expected_operands_len, provided_operands_len)));
            }
            else if !operands.is_empty() {
                assembler_result.fails.push(Error::in_line(asm_file_name, &line_nr, format!("Too many operands - expected {}, found {}", expected_operands_len, provided_operands_len)));
            } else {
                match out_line.len() == expected_line_length {
                    true => {
                        out_line += "\n";
                    }
                    false => assembler_result.fails.push(Error::in_line(&asm_file_name, &line_nr, "Operand/operands out of bounds".to_string()))
                }
            }
        } else {
            assembler_result.fails.push(Error::in_line(&asm_file_name, &line_nr, format!(r#"Unknown instruction mnemonic "{}""#, mnemonic)));
        }
        println!("{}", out_line);
        out += &out_line;
    }
    let out_len = out.split(line_separation).filter(|x| !x.is_empty()).count();
    if out_len > isa.cpu_data.program_memory_lines {
        assembler_result.fails.push(Error::no_line(&asm_file_name, format!("Program memory capacity exceeded - {} lines used, {} available", out_len, isa.cpu_data.program_memory_lines)));
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

        let mut label_declarations: Vec<Label> = Vec::new();
        let mut define_declarations: Vec<DefinePair> = Vec::new();

        open_files(&mut isa, &mut isa_file_name, &mut asm, &mut asm_file_name,  &mut label_declarations, &mut define_declarations, &mut assembler_result);
        continue_on_err!(assembler_result);

        let isa = isa.unwrap();
        let bin = parse(&isa, &isa_file_name, &asm, &asm_file_name, &label_declarations, &mut define_declarations, &mut assembler_result);
        continue_on_err!(assembler_result);

        let out_len = bin.split(line_separation).filter(|x| !x.is_empty()).count();

        let bin_file_name = &asm_file_name.replace("ASM", "BIN").replace(".asm", ".bin");
        match fs::write(Path::new(bin_file_name), bin) {
            Ok(_) => {
                assembler_result.info.push("\nNO ERRORS FOUND\n".to_string());
                assembler_result.info.push(format!("CPU: {}", isa.cpu_data.cpu_name.clone()));
                assembler_result.info.push(format!("Memory lines used: {}/{}\n", out_len, isa.cpu_data.program_memory_lines));
                assembler_result.info.push(format!("ASM file: {}", asm_file_name));
                assembler_result.info.push(format!("ISA file: {}\n", isa_file_name));
                assembler_result.info.push(format!(r#"SAVED TO: "{}"{}"#, bin_file_name, "\n"));
            },
            Err(_) => assembler_result.fails.push(Error::no_line(bin_file_name, format!(r#"Failed to write to "{}""#, bin_file_name)))
        }

        assembler_result.report();
    }
}
