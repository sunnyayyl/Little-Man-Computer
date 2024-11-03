use crate::MemonicType;

macro_rules! op_code_enum {
    ($($name:ident),*)=>{
        #[derive(Debug)]
        pub enum OpCode{
            $(
                $name(Option<u16>),
            )*
        }
    }
}
op_code_enum!(ADD, SUB, STA, LDA, BRA, BRZ, BRP, INP, OUT, HLT, COB);

impl From<u16> for OpCode {
    fn from(code: u16) -> Self {
        if (100..=199).contains(&code) {
            OpCode::ADD(Some(code - 100))
        } else if (200..=299).contains(&code) {
            OpCode::SUB(Some(code - 200))
        } else if (300..=399).contains(&code) {
            OpCode::STA(Some(code - 300))
        } else if (500..=599).contains(&code) {
            OpCode::LDA(Some(code - 500))
        } else if (600..=699).contains(&code) {
            OpCode::BRA(Some(code - 600))
        } else if (700..=799).contains(&code) {
            OpCode::BRZ(Some(code - 700))
        } else if (800..=899).contains(&code) {
            OpCode::BRP(Some(code - 800))
        } else if code == 901 {
            OpCode::INP(None)
        } else if code == 902 {
            OpCode::OUT(None)
        } else if code == 000 {
            OpCode::HLT(None)
        } else {
            panic!("Unknown numeric code: {:?}", code)
        }
    }
}
impl OpCode {
    pub fn from_mnemonic_type(mnemonic_type: MemonicType, addresses: Option<u16>) -> Self {
        match mnemonic_type {
            MemonicType::ADD => OpCode::ADD(addresses.expect("ADD requires an address").into()),
            MemonicType::SUB => OpCode::SUB(addresses.expect("SUB requires an address").into()),
            MemonicType::STA => OpCode::STA(addresses.expect("STA requires an address").into()),
            MemonicType::LDA => OpCode::LDA(addresses.expect("LDA requires an address").into()),
            MemonicType::BRA => OpCode::BRA(addresses.expect("BRA requires an address").into()),
            MemonicType::BRZ => OpCode::BRZ(addresses.expect("BRZ requires an address").into()),
            MemonicType::BRP => OpCode::BRP(addresses.expect("BRP requires an address").into()),
            MemonicType::INP => OpCode::INP(addresses),
            MemonicType::OUT => OpCode::OUT(addresses),
            MemonicType::HLT => OpCode::HLT(addresses),
            MemonicType::COB => OpCode::COB(addresses),
        }
    }
    pub fn to_numeric_representation(&self) -> u16 {
        match self {
            OpCode::ADD(a) => 100 + a.expect("ADD requires an address"),
            OpCode::SUB(a) => 200 + a.expect("SUB requires an address"),
            OpCode::STA(a) => 300 + a.expect("STA requires an address"),
            OpCode::LDA(a) => 500 + a.expect("LDA requires an address"),
            OpCode::BRA(a) => 600 + a.expect("BRA requires an address"),
            OpCode::BRZ(a) => 700 + a.expect("BRZ requires an address"),
            OpCode::BRP(a) => 800 + a.expect("BRP requires an address"),
            OpCode::INP(_) => 901,
            OpCode::OUT(_) => 902,
            OpCode::HLT(_) => 000,
            OpCode::COB(_) => 000,
        }
    }
}
