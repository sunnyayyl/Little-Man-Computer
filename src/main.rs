mod assembler;
mod opcodes;
pub mod vm;

pub use opcodes::OpCode;
use std::fmt::Display;
use std::io::{BufRead, BufReader};
use std::{env, fs};
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
    if args.len() > 2 {
        if args[1] == "assemble" {
            let target_file =
                args[2].split(".").collect::<Vec<&str>>()[0].to_owned() + "_mailbox.bin";
            println!("Compiling file {} into {}", args[2], target_file);
            let code_file = fs::File::open(args[2].as_str()).expect("Failed to open file");
            let lines: Vec<String> = BufReader::new(code_file)
                .lines()
                .collect::<Result<_, _>>()
                .expect("Failed to read file");
            let mailbox = assembler::Parser::new(lines).parse().unwrap();
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(target_file)
                .expect("Failed to create file");
            mailbox.export_to_file(&mut file).unwrap();
        } else if args[1] == "exec" {
            let mut file = fs::OpenOptions::new()
                .read(true)
                .open(&args[2])
                .expect("Failed to open file");
            let mailbox = Mailbox::read_from_file(&mut file).unwrap();
            let mut r = vm::Runtime::new(mailbox);
            r.start();
        } else if args[1] == "run" {
            let target_file =
                args[2].split(".").collect::<Vec<&str>>()[0].to_owned() + "_mailbox.bin";
            println!("Compiling file {} into {}", args[2], target_file);
            let code_file = fs::File::open(args[2].as_str()).expect("Failed to open file");
            let lines: Vec<String> = BufReader::new(code_file)
                .lines()
                .collect::<Result<_, _>>()
                .expect("Failed to read file");
            let mailbox = assembler::Parser::new(lines).parse().unwrap();
            let mut r = vm::Runtime::new(mailbox);
            r.start();
        } else if args[1] == "debug" {
            let mut file = fs::OpenOptions::new()
                .read(true)
                .open(&args[2])
                .expect("Failed to open file");
            let mailbox = Mailbox::read_from_file(&mut file).unwrap();
            let mut r = vm::Runtime::new(mailbox);
            r.debug();
        } else {
            println!("Invalid command: {}", args[1]);
        }
    }
}
