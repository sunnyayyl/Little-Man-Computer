mod assembler;
mod error;
mod lexer;
mod opcodes;
pub mod vm;

use crate::assembler::Assembler;
use crate::lexer::LexerLineStructure;
pub use opcodes::MemonicType;
pub use opcodes::OpCode;
use std::collections::HashMap;
use std::io::{stdin, stdout, BufReader, Write};
use std::{env, fs, process};
pub use vm::Mailbox;

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(command) = args.get(1) {
        let mailbox: Mailbox;
        let label_lookup: HashMap<String, u16> = HashMap::new();
        if let Some(filename) = args.get(2) {
            let mut file = fs::OpenOptions::new()
                .read(true)
                .open(filename)
                .expect("Failed to open file");
            if filename.ends_with(".bin") {
                let mailbox_from_bin = Mailbox::read_from_file(&mut file);
                mailbox = mailbox_from_bin.expect("Failed to read mailbox");
            } else {
                let mut new_mailbox = Mailbox([0_u16; 100]);
                let label_lookup;
                let lexer_result: LexerLineStructure;
                {
                    let mut file = fs::OpenOptions::new()
                        .read(true)
                        .open(filename)
                        .expect("Failed to open file");
                    let lines = BufReader::new(&mut file);
                    let lexer = lexer::Lexer::new();
                    let v = lexer.parse(lines);
                    let result = v.0;
                    label_lookup = v.1;
                    match result {
                        Ok(result) => {
                            lexer_result = result;
                        }
                        Err(err) => {
                            println!("{}", err);
                            process::exit(1);
                        }
                    }
                }
                let mut assembler = Assembler::new(lexer_result, label_lookup);
                loop {
                    match assembler.parse_line() {
                        assembler::State::Ok(opcode) => {
                            new_mailbox.set(
                                assembler.current_line() - 1,
                                opcode.to_numeric_representation(),
                            );
                        }
                        assembler::State::Err(err) => {
                            println!("{}", err);
                            process::exit(1);
                        }
                        assembler::State::Done => break,
                    }
                }

                mailbox = new_mailbox;
                println!("{:?}", mailbox);
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
                    let label_info: HashMap<u16, String> = label_lookup.iter().map(|(k, v)| (*v, k.clone())).collect();
                    let mut runtime = vm::Runtime::new(mailbox);
                    let mut breakpoints: std::vec::Vec<u16> = vec![];
                    loop {
                        let mut input = String::new();
                        print!("\n(debug) ");
                        stdout().flush().expect("Failed to flush screen");
                        let _ = stdin().read_line(&mut input).expect("Failed to read line");
                        match input.trim().split(' ').collect::<Vec<&str>>().as_slice() {
                            ["run"] => {
                                while !breakpoints.contains(runtime.get_program_counter()) && runtime.evaluate_current() {}
                                let addr = runtime.get_program_counter();
                                if breakpoints.contains(addr) {
                                    println!("(Breakpoint hit at address: {})", addr);
                                }
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
                                runtime.evaluate_current();
                            }
                            ["mailbox"] => println!("{:?}", runtime.get_mailbox()),
                            ["get", addr] => {
                                let addr = addr.parse::<usize>();
                                if let Ok(addr) = addr {
                                    if (0..=100).contains(&addr) {
                                        println!("{}", runtime.get_mailbox()[addr]);
                                    } else {
                                        println!("Mailbox addresses can only be between 0-100")
                                    }
                                } else {
                                    println!("Mailbox addresses must be positive integer")
                                }
                            }
                            ["breakpoint", addr] => {
                                let addr = addr.parse::<usize>();
                                if let Ok(addr) = addr {
                                    if (0..=100).contains(&addr) {
                                        breakpoints.push(addr as u16);
                                    } else {
                                        println!("Mailbox addresses can only be between 0-100")
                                    }
                                } else {
                                    println!("Mailbox addresses must be positive integer")
                                }
                            }
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
