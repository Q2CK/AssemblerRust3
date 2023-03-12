pub use serde_derive::{Serialize, Deserialize};
pub use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CpuData {
    pub cpu_name: String,
    pub instruction_length: usize,
    pub program_memory_lines: usize
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Kind {
    Opcode(String),
    Operand(usize),
    Filler(usize)
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Instruction {
    pub layout: Vec<Kind>,
    pub keywords: Vec<String>
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ISA {
    pub cpu_data: CpuData,
    pub define: HashMap<String, HashMap<String, String>>,
    pub instructions: HashMap<String, Instruction>,
}

pub struct Error {
    pub file: String,
    pub line: Option<u32>,
    pub message: String
}

impl Error {
    pub fn no_line(file: &String, message: String) -> Error {
        return Error {
            file: file.to_string(), line: None, message: message.to_string()
        }
    }

    pub fn in_line(file: &String, line: &usize, message: &String) -> Error {
        return Error {
            file: file.to_string(), line: Some(*line as u32), message: message.to_string()
        }
    }
}

pub struct AssemblerResult{
    pub info: Vec<String>,
    pub fails: Vec<Error>
}

impl AssemblerResult {
    pub fn report(&self) {
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

#[derive(Debug, Deserialize, Clone)]
pub struct Token {
    pub content: String,
}

impl Token {
    pub fn new(content: &String) -> Token{
        Token {
            content: content.to_string()
        }
    }

    pub fn tokenize(line: String) -> Vec<Token> {
        line.split(|c| c == ',' || c == ' ')
            .map(str::to_string)
            .filter(|s| !s.is_empty())
            .map(|x| Token::new(&x))
            .collect()
    }

    pub fn to_bin(&self, bits: usize) -> Result<String, String> {
        match self.content.clone().parse::<usize>() {
            Ok(v) => Ok(format!("{v:b>bits$}")),
            Err(_) => Err("Failed to parse operand".to_string())
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Line {
    pub tokens: Vec<Token>,
    pub line_nr: usize
}

impl Line {
    pub fn new(line_string: String, line_counter: &mut usize) -> Line {
        let line_nr= *line_counter;
        *line_counter += 1;
        Line {
            tokens: Token::tokenize(line_string.trim().to_string()),
            line_nr
        }
    }
}