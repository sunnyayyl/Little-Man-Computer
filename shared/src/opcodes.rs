#[cfg(not(feature = "std"))]
use core::{fmt, fmt::Display};
#[cfg(feature = "std")]
use std::{fmt, fmt::Display};
#[derive(Debug)]
pub enum AddressType {
    Pointer,
    Literal,
}
impl AddressType {
    fn is_pointer(&self) -> bool {
        matches!(self, AddressType::Pointer)
    }
    fn is_literal(&self) -> bool {
        matches!(self, AddressType::Literal)
    }
}
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
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $(
                    MemonicType::$name => write!(f, "{}", stringify!($name)),
                    )*
                }
            }
        }
    }
}

mnemonics_type_enum!(
    ADD, SUB, STA, LDA, BRA, BRZ, BRP, INP, OUT, HLT, COB, DAT, SOUT, MULT, PUSH, POP
);

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
                $name(Option<u16>, AddressType),
            )*
        }
        impl Display for OpCode{
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $(
                    OpCode::$name(a, t) => {
                        match (a,t){
                            (Some(addr), AddressType::Pointer) => write!(f, "[{}({})]", stringify!($name), addr),
                            (Some(addr), AddressType::Literal) => write!(f, "{}({})", stringify!($name), addr),
                            _ => write!(f, "{}", stringify!($name)),

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
                    OpCode::$name(a, _) => a,
                    )*
                }
            }
        }
    }
}
op_code_enum!(ADD, SUB, STA, LDA, BRA, BRZ, BRP, INP, OUT, HLT, COB, DAT, SOUT, MULT, PUSH, POP); // Added SOUT, StringOutput

impl TryFrom<u16> for OpCode {
    type Error = OpCodeError;

    fn try_from(mut code: u16) -> Result<Self, Self::Error> {
        let mut adress_type: AddressType;
        if code & 1024 == 0 {
            // if the 10th bit is off, it's a normal address
            adress_type = AddressType::Literal;
        } else {
            // if 10th bit, handel the address as an address to a pointer
            code ^= 1024;
            adress_type = AddressType::Pointer;
        }
        if (100..=199).contains(&code) {
            Ok(OpCode::ADD(Some(code - 100), adress_type))
        } else if (200..=299).contains(&code) {
            Ok(OpCode::SUB(Some(code - 200), adress_type))
        } else if (300..=399).contains(&code) {
            Ok(OpCode::STA(Some(code - 300), adress_type))
        } else if (400..=499).contains(&code) {
            Ok(OpCode::MULT(Some(code - 400), adress_type))
        } else if (500..=599).contains(&code) {
            Ok(OpCode::LDA(Some(code - 500), adress_type))
        } else if (600..=699).contains(&code) {
            Ok(OpCode::BRA(Some(code - 600), adress_type))
        } else if (700..=799).contains(&code) {
            Ok(OpCode::BRZ(Some(code - 700), adress_type))
        } else if (800..=899).contains(&code) {
            Ok(OpCode::BRP(Some(code - 800), adress_type))
        } else if code == 901 {
            Ok(OpCode::INP(None, adress_type))
        } else if code == 902 {
            Ok(OpCode::OUT(None, adress_type))
        } else if code == 904 {
            Ok(OpCode::SOUT(None, adress_type))
        } else if code == 999 {
            Ok(OpCode::PUSH(None, adress_type))
        } else if code == 998 {
            Ok(OpCode::POP(None, adress_type))
        } else if code == 000 {
            Ok(OpCode::HLT(None, adress_type))
        } else {
            Err(OpCodeError::InvalidOpCode(code))
        }
    }
}

impl OpCode {
    pub fn try_from_mnemonic_type(
        mnemonic_type: MemonicType,
        addresses: Option<u16>,
        address_type: AddressType,
    ) -> Result<Self, OpCodeError> {
        match mnemonic_type {
            MemonicType::ADD => {
                if let Some(addr) = addresses {
                    Ok(OpCode::ADD(Some(addr), address_type))
                } else {
                    Err(OpCodeError::AddressExpected(MemonicType::ADD))
                }
            }
            MemonicType::SUB => {
                if let Some(addr) = addresses {
                    Ok(OpCode::SUB(Some(addr), address_type))
                } else {
                    Err(OpCodeError::AddressExpected(MemonicType::SUB))
                }
            }
            MemonicType::MULT => {
                if let Some(addr) = addresses {
                    Ok(OpCode::MULT(Some(addr), address_type))
                } else {
                    Err(OpCodeError::AddressExpected(MemonicType::MULT))
                }
            }
            MemonicType::STA => {
                if let Some(addr) = addresses {
                    Ok(OpCode::STA(Some(addr), address_type))
                } else {
                    Err(OpCodeError::AddressExpected(MemonicType::STA))
                }
            }
            MemonicType::LDA => {
                if let Some(addr) = addresses {
                    Ok(OpCode::LDA(Some(addr), address_type))
                } else {
                    Err(OpCodeError::AddressExpected(MemonicType::LDA))
                }
            }
            MemonicType::BRA => {
                if let Some(addr) = addresses {
                    Ok(OpCode::BRA(Some(addr), address_type))
                } else {
                    Err(OpCodeError::AddressExpected(MemonicType::BRA))
                }
            }
            MemonicType::BRZ => {
                if let Some(addr) = addresses {
                    Ok(OpCode::BRZ(Some(addr), address_type))
                } else {
                    Err(OpCodeError::AddressExpected(MemonicType::BRZ))
                }
            }
            MemonicType::BRP => {
                if let Some(addr) = addresses {
                    Ok(OpCode::BRP(Some(addr), address_type))
                } else {
                    Err(OpCodeError::AddressExpected(MemonicType::BRP))
                }
            }
            MemonicType::INP => {
                if let Some(addr) = addresses {
                    Ok(OpCode::INP(Some(addr), address_type))
                } else {
                    Ok(OpCode::INP(None, address_type))
                }
            }
            MemonicType::OUT => {
                if let Some(addr) = addresses {
                    Ok(OpCode::OUT(Some(addr), address_type))
                } else {
                    Ok(OpCode::OUT(None, address_type))
                }
            }
            MemonicType::HLT => {
                if let Some(addr) = addresses {
                    Ok(OpCode::HLT(Some(addr), address_type))
                } else {
                    Ok(OpCode::HLT(None, address_type))
                }
            }
            MemonicType::COB => {
                if let Some(addr) = addresses {
                    Ok(OpCode::COB(Some(addr), address_type))
                } else {
                    Ok(OpCode::COB(None, address_type))
                }
            }
            MemonicType::DAT => Ok(OpCode::DAT(Some(addresses.unwrap_or(0)), address_type)),
            MemonicType::SOUT => {
                if let Some(addr) = addresses {
                    Ok(OpCode::SOUT(Some(addr), address_type))
                } else {
                    Ok(OpCode::SOUT(None, address_type))
                }
            }
            MemonicType::POP => {
                if let Some(addr) = addresses {
                    Ok(OpCode::POP(Some(addr), address_type))
                } else {
                    Ok(OpCode::POP(None, address_type))
                }
            }
            MemonicType::PUSH => {
                if let Some(addr) = addresses {
                    Ok(OpCode::PUSH(Some(addr), address_type))
                } else {
                    Ok(OpCode::PUSH(None, address_type))
                }
            }
        }
    }
    pub fn from_mnemonic_type(
        mnemonic_type: MemonicType,
        addresses: Option<u16>,
        address_type: AddressType,
    ) -> Self {
        Self::try_from_mnemonic_type(mnemonic_type, addresses, address_type).unwrap()
    }
    fn get_pointer_toggle(c: bool) -> u16 {
        if c {
            1024
        } else {
            0
        }
    }
    pub fn to_numeric_representation(&self) -> u16 {
        match self {
            OpCode::ADD(a, t) => {
                (100 + a.expect("ADD requires an address"))
                    ^ Self::get_pointer_toggle(t.is_pointer())
            }
            OpCode::SUB(a, t) => {
                (200 + a.expect("SUB requires an address"))
                    ^ Self::get_pointer_toggle(t.is_pointer())
            }
            OpCode::MULT(a, t) => {
                (400 + a.expect("MULT requires an address"))
                    ^ Self::get_pointer_toggle(t.is_pointer())
            }
            OpCode::STA(a, t) => {
                (300 + a.expect("STA requires an address"))
                    ^ Self::get_pointer_toggle(t.is_pointer())
            }
            OpCode::LDA(a, t) => {
                (500 + a.expect("LDA requires an address"))
                    ^ Self::get_pointer_toggle(t.is_pointer())
            }
            OpCode::BRA(a, t) => {
                (600 + a.expect("BRA requires an address"))
                    ^ Self::get_pointer_toggle(t.is_pointer())
            }
            OpCode::BRZ(a, t) => {
                (700 + a.expect("BRZ requires an address"))
                    ^ Self::get_pointer_toggle(t.is_pointer())
            }
            OpCode::BRP(a, t) => {
                (800 + a.expect("BRP requires an address"))
                    ^ Self::get_pointer_toggle(t.is_pointer())
            }
            OpCode::INP(_, _) => 901,
            OpCode::OUT(_, _) => 902,
            OpCode::HLT(_, _) => 000,
            OpCode::COB(_, _) => 000,
            OpCode::DAT(a, t) => (a.unwrap_or(0)) ^ Self::get_pointer_toggle(t.is_pointer()),
            OpCode::SOUT(_, _) => 904,
            OpCode::POP(_, _) => 998,
            OpCode::PUSH(_, _) => 999,
        }
    }
}
