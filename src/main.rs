mod assembler;
mod opcodes;
pub mod vm;

use crate::assembler::Parser;
pub use opcodes::OpCode;
use std::collections::HashMap;
use std::fmt::Display;
use std::io::{stdin, stdout, BufRead, BufReader, Write};
use std::{env, fs, process};
pub use vm::Mailbox;

macro_rules! mnemonics_type_enum {
    ($($name:ident),*)=>{
        #[derive(Debug,PartialEq)]
        pub enum MemonicType{
            $(
                $name,
            )*
        }
        impl MemonicType{
            pub fn from_string(s: &str)->Option<MemonicType>{
                match s {
                    $(
                    stringify!($name) => Some(MemonicType::$name),
                    )*
                    _ => None,
                }
            }
        }
        impl Display for MemonicType{
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $(
                    MemonicType::$name => write!(f, "{}", stringify!($name)),
                    )*
                }
            }
        }
    }
}

mnemonics_type_enum!(ADD, SUB, STA, LDA, BRA, BRZ, BRP, INP, OUT, HLT, COB, DAT, SOUT);

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(command) = args.get(1) {
        let mailbox: Mailbox;
        let mut parser: Option<Parser> = None;
        if let Some(filename) = args.get(2) {
            let mut file = fs::OpenOptions::new()
                .read(true)
                .open(filename)
                .expect("Failed to open file");
            if filename.ends_with(".bin") {
                let mailbox_from_bin = Mailbox::read_from_file(&mut file);
                mailbox = mailbox_from_bin.expect("Failed to read mailbox");
            } else {
                let lines = BufReader::new(&mut file)
                    .lines()
                    .collect::<Result<_, _>>()
                    .expect("Failed to read file");
                let mut p = assembler::Parser::new(lines);
                mailbox = p.parse().unwrap();
                parser = Some(p);
            }
            match command.as_str() {
                "run" => {
                    let mut runtime = vm::Runtime::new(mailbox);
                    runtime.start();
                }
                "assemble" => {
                    let target_filename =
                        filename.split(".").collect::<Vec<&str>>()[0].to_owned() + "_mailbox.bin"; // slightly scuffed
                    let mut target_file = fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(target_filename)
                        .expect("Failed to create file");
                    mailbox.export_to_file(&mut target_file).expect("Failed to write assembled file");
                }

                "debug" => {
                    let mut label_info: HashMap<u16, String> = HashMap::new();
                    if let Some(parser) = parser {
                        label_info = parser.get_label_lookup().iter().map(|(k, v)| (*v, k.clone())).collect();
                    }
                    let mut runtime = vm::Runtime::new(mailbox);
                    loop {
                        let mut input = String::new();
                        print!("\n(debug) ");
                        stdout().flush().expect("Failed to flush screen");
                        let _ = stdin().read_line(&mut input).expect("Failed to read line");
                        match input.trim().split(' ').collect::<Vec<&str>>().as_slice() {
                            ["run"] => {
                                runtime.start();
                                return;
                            }
                            ["step"] => {
                                let line = runtime.get_program_counter();
                                let current = runtime.get_current_instruction();
                                let mut line_label = String::from("");
                                if let Some(label) = label_info.get(line) {
                                    line_label += label;
                                    line_label += " ";
                                } else {
                                    line_label = runtime.get_program_counter().to_string() + " ";
                                }
                                if let (Some(op_code), _) = current {
                                    if let Some(addr) = op_code.get_address() {
                                        println!("{}{} {}", line_label, op_code, label_info.get(addr).unwrap_or(&String::from("")));
                                    } else {
                                        println!("{}{}", line_label, op_code);
                                    }
                                } else if let (None, literal) = current {
                                    println!("{}{}", line_label, literal);
                                }
                                runtime.evaluate_next();
                            }
                            ["mailbox"] => println!("{:?}", runtime.get_mailbox()),
                            ["get", addr] => {
                                let addr = addr.parse::<usize>();
                                if let Ok(addr) = addr {
                                    if (0..=999).contains(&addr) {
                                        println!("{}", runtime.get_mailbox()[addr]);
                                    } else {
                                        println!("Mailbox addresses can only be between 0-999")
                                    }
                                } else {
                                    println!("Mailbox addresses must be positive integer")
                                }
                            },
                            ["counter"] => println!("{}", runtime.get_program_counter()),
                            ["program_counter"] => println!("{}", runtime.get_program_counter()),
                            ["accumulator"] => println!("{}", runtime.get_accumulator()),
                            ["help"] => println!("Available command: step, mailbox, counter, program_counter or counter, get address-here, accumulator"),
                            _ => println!("Unknown command"),
                        }
                    }
                }
                "help" => println!("Available command: step, mailbox, counter, program_counter or counter, accumulator"),
                &_ => println!("Unknown command, use the help command for options"),
            }
        } else {
            println!("Missing file name");
            process::exit(1);
        }
    } else {
        process::exit(0);
    }
}
