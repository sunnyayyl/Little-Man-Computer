mod opcodes;
pub mod vm;
pub use opcodes::OpCode;
use std::fs;
pub use vm::Mailbox;

macro_rules! mnemonics_type_enum {
    ($($name:ident),*)=>{
        #[derive(Debug)]
        enum MemonicType{
            $(
                $name,
            )*
        }
    }
}

mnemonics_type_enum!(ADD, SUB, STA, LDA, BRA, BRZ, BRP, INP, OUT, HLT, COB);

struct Token {
    left: Option<String>,
    statement: MemonicType,
    right: Option<String>,
}

fn main() {
    let mailbox = vm::Mailbox::from(vec![901_u16, 308, 901, 309, 508, 209, 902, 000]);
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
