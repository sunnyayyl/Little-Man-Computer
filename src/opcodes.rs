use crate::MemonicType;

#[derive(Debug)]
pub enum OpCodeError {
    InvalidOpCode(u16),
}
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
op_code_enum!(ADD, SUB, STA, LDA, BRA, BRZ, BRP, INP, OUT, HLT, COB, DAT, SOUT); // Added SOUT, StringOutput

impl TryFrom<u16> for OpCode {
    type Error = OpCodeError;

    fn try_from(code: u16) -> Result<Self, Self::Error> {
        if (100..=199).contains(&code) {
            Ok(OpCode::ADD(Some(code - 100)))
        } else if (200..=299).contains(&code) {
            Ok(OpCode::SUB(Some(code - 200)))
        } else if (300..=399).contains(&code) {
            Ok(OpCode::STA(Some(code - 300)))
        } else if (500..=599).contains(&code) {
            Ok(OpCode::LDA(Some(code - 500)))
        } else if (600..=699).contains(&code) {
            Ok(OpCode::BRA(Some(code - 600)))
        } else if (700..=799).contains(&code) {
            Ok(OpCode::BRZ(Some(code - 700)))
        } else if (800..=899).contains(&code) {
            Ok(OpCode::BRP(Some(code - 800)))
        } else if code == 901 {
            Ok(OpCode::INP(None))
        } else if code == 902 {
            Ok(OpCode::OUT(None))
        } else if code == 904 {
            Ok(OpCode::SOUT(None))
        } else if code == 000 {
            Ok(OpCode::HLT(None))
        } else {
            Err(OpCodeError::InvalidOpCode(code))
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
            MemonicType::DAT => OpCode::DAT(addresses.unwrap_or(0).into()),
            MemonicType::SOUT => OpCode::SOUT(addresses),
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
            OpCode::DAT(a) => a.unwrap_or(0),
            OpCode::SOUT(_) => 904,
        }
    }
}
