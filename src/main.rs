mod opcodes;
mod parser;
pub mod vm;

pub use opcodes::OpCode;
use std::fmt::Display;
use std::fs;
use std::io::{BufRead, BufReader};
pub use vm::Mailbox;

macro_rules! mnemonics_type_enum {
    ($($name:ident),*)=>{
        #[derive(Debug,PartialEq)]
        enum MemonicType{
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

mnemonics_type_enum!(ADD, SUB, STA, LDA, BRA, BRZ, BRP, INP, OUT, HLT, COB, DAT);

fn main() {
    //let mailbox = vm::Mailbox::from(vec![901_u16, 308, 901, 309, 508, 209, 902, 000]);
    let code_file = fs::File::open("code2.txt").expect("Failed to open file");
    let lines: Vec<String> = BufReader::new(code_file)
        .lines()
        .collect::<Result<_, _>>()
        .expect("Failed to read file");
    let mailbox = parser::Parser::new(lines).parse().unwrap();
    println!("{:?}", mailbox);
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("mailbox.bin")
            .expect("Failed to create file");
        mailbox.export_to_file(&mut file).unwrap();
    } // just to test the exporting and importing functionality
    let mut file = fs::OpenOptions::new()
        .read(true)
        .open("mailbox.bin")
        .expect("Failed to open file");
    let test = Mailbox::read_from_file(&mut file).unwrap();
    println!("{:?}", test);
    let mut r = vm::Runtime::new(mailbox);
    r.start();
}
