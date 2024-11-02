macro_rules! op_code_enum {
    ($($name:ident),*)=>{
        #[derive(Debug)]
        pub enum OpCode{
            $(
                $name(u16),
            )*
        }
    }
}
op_code_enum!(ADD, SUB, STA, LDA, BRA, BRZ, BRP, INP, OUT, HLT, COB);

impl From<u16> for OpCode {
    fn from(code: u16) -> Self {
        if (100..=199).contains(&code) {
            OpCode::ADD(code - 100)
        } else if (200..=299).contains(&code) {
            OpCode::SUB(code - 200)
        } else if (300..=399).contains(&code) {
            OpCode::STA(code - 300)
        } else if (500..=599).contains(&code) {
            OpCode::LDA(code - 500)
        } else if (600..=699).contains(&code) {
            OpCode::BRA(code - 600)
        } else if (700..=799).contains(&code) {
            OpCode::BRZ(code - 700)
        } else if (800..=899).contains(&code) {
            OpCode::BRP(code - 700)
        } else if code == 901 {
            OpCode::INP(901)
        } else if code == 902 {
            OpCode::OUT(902)
        } else if code == 000 {
            OpCode::HLT(000)
        } else {
            panic!("Unknown numeric code")
        }
    }
}
