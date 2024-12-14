#[cfg(feature="std")]
use std::fmt::Display;
#[cfg(not(feature="std"))]
use core::fmt::Display;
macro_rules! mnemonics_type_enum {
    ($($name:ident),*)=>{
        #[derive(Debug,PartialEq,Clone, Copy)]
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

#[derive(Debug)]
pub enum OpCodeError {
    InvalidOpCode(u16),
    AddressExpected(MemonicType),
}
macro_rules! op_code_enum {
    ($($name:ident),*)=>{
        #[derive(Debug)]
        pub enum OpCode{
            $(
                $name(Option<u16>),
            )*
        }
        impl Display for OpCode{
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $(
                    OpCode::$name(a) => {
                        if let Some(addr)=a{
                            write!(f, "{}({})", stringify!($name), addr)
                        }else{
                            write!(f, "{}", stringify!($name))
                        }
                    },
                    )*
                }
            }
        }
        impl OpCode{
            pub fn get_address(&self)->&Option<u16>{
                match self {
                    $(
                    OpCode::$name(a) => a,
                    )*
                }
            }
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
    pub fn try_from_mnemonic_type(
        mnemonic_type: MemonicType,
        addresses: Option<u16>,
    ) -> Result<Self, OpCodeError> {
        match mnemonic_type {
            MemonicType::ADD => {
                if let Some(addr) = addresses {
                    Ok(OpCode::ADD(Some(addr)))
                } else {
                    Err(OpCodeError::AddressExpected(MemonicType::ADD))
                }
            }
            MemonicType::SUB => {
                if let Some(addr) = addresses {
                    Ok(OpCode::SUB(Some(addr)))
                } else {
                    Err(OpCodeError::AddressExpected(MemonicType::SUB))
                }
            }
            MemonicType::STA => {
                if let Some(addr) = addresses {
                    Ok(OpCode::STA(Some(addr)))
                } else {
                    Err(OpCodeError::AddressExpected(MemonicType::STA))
                }
            }
            MemonicType::LDA => {
                if let Some(addr) = addresses {
                    Ok(OpCode::LDA(Some(addr)))
                } else {
                    Err(OpCodeError::AddressExpected(MemonicType::LDA))
                }
            }
            MemonicType::BRA => {
                if let Some(addr) = addresses {
                    Ok(OpCode::BRA(Some(addr)))
                } else {
                    Err(OpCodeError::AddressExpected(MemonicType::BRA))
                }
            }
            MemonicType::BRZ => {
                if let Some(addr) = addresses {
                    Ok(OpCode::BRZ(Some(addr)))
                } else {
                    Err(OpCodeError::AddressExpected(MemonicType::BRZ))
                }
            }
            MemonicType::BRP => {
                if let Some(addr) = addresses {
                    Ok(OpCode::BRP(Some(addr)))
                } else {
                    Err(OpCodeError::AddressExpected(MemonicType::BRP))
                }
            }
            MemonicType::INP => {
                if let Some(addr) = addresses {
                    Ok(OpCode::INP(Some(addr)))
                } else {
                    Ok(OpCode::INP(None))
                }
            }
            MemonicType::OUT => {
                if let Some(addr) = addresses {
                    Ok(OpCode::OUT(Some(addr)))
                } else {
                    Ok(OpCode::OUT(None))
                }
            }
            MemonicType::HLT => {
                if let Some(addr) = addresses {
                    Ok(OpCode::HLT(Some(addr)))
                } else {
                    Ok(OpCode::HLT(None))
                }
            }
            MemonicType::COB => {
                if let Some(addr) = addresses {
                    Ok(OpCode::COB(Some(addr)))
                } else {
                    Ok(OpCode::COB(None))
                }
            }
            MemonicType::DAT => Ok(OpCode::DAT(Some(addresses.unwrap_or(0)))),
            MemonicType::SOUT => {
                if let Some(addr) = addresses {
                    Ok(OpCode::SOUT(Some(addr)))
                } else {
                    Ok(OpCode::SOUT(None))
                }
            }
        }
    }
    pub fn from_mnemonic_type(mnemonic_type: MemonicType, addresses: Option<u16>) -> Self {
        Self::try_from_mnemonic_type(mnemonic_type, addresses).unwrap()
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
