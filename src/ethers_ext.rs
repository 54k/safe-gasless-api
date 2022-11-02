use eth_encode_packed::abi::encode_packed;
use eth_encode_packed::ethabi::ethereum_types::{Address, U256};
use eth_encode_packed::SolidityDataType;
use ethers::abi::{AbiEncode, Token};
use ethers::utils::keccak256;
use log::debug;

struct Sdt<'a>(SolidityDataType<'a>);

impl<'a> TryFrom<&'a Token> for Sdt<'a> {
    type Error = String;

    fn try_from(token: &'a Token) -> Result<Self, Self::Error> {
        match token {
            Token::Address(val) => Ok(Sdt(SolidityDataType::Address(Address::from(val.as_fixed_bytes())))),
            Token::Bytes(val) => Ok(Sdt(SolidityDataType::Bytes(val.as_slice()))),
            Token::Uint(val) => Ok(Sdt(SolidityDataType::Number(U256::from(val.encode().as_slice())))),
            Token::Int(val) => Ok(Sdt(SolidityDataType::Number(U256::from(val.encode().as_slice())))),
            Token::String(val) => Ok(Sdt(SolidityDataType::String(val))),
            Token::Bool(val) => Ok(Sdt(SolidityDataType::Bool(*val))),
            _ => Err(format!("Unexpected token passed {:?}", token))
        }
    }
}

fn convert_to_sdt(tokens: &[Token]) -> Vec<SolidityDataType> {
    tokens.iter()
        .map(Sdt::try_from)
        .map(|sdt| sdt.unwrap().0)
        .collect()
}

pub(crate) fn solidity_keccak256(tokens: &[Token]) -> [u8; 32] {
    let tokens = convert_to_sdt(tokens);
    let (packed, hash) = encode_packed(tokens.as_slice());
    debug!("Packed hash: {}", hash);
    keccak256(packed)
}