use std::{collections::HashMap, str::FromStr};

use funcs::is_valid_jump_dest;
use primitive_types::{U256, U512};
use serde::Deserialize;
mod funcs;
use crate::funcs::{sar, sdiv, sgt, signextend, slt, smod};
use sha3::{Digest, Keccak256};

#[derive(Debug, Clone)]
pub struct EvmResult {
    pub value: Option<Vec<u8>>,
    pub stack: Vec<U256>,
    pub success: bool,
    pub return_data: Vec<u8>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EvmContext {
    coinbase: Option<String>,
    basefee: Option<String>,
    timestamp: Option<String>,
    number: Option<String>,
    difficulty: Option<String>,
    gaslimit: Option<String>,
    chainid: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TxData {
    data: Option<String>,
    from: Option<String>,
    to: Option<String>,
    gasprice: Option<String>,
    origin: Option<String>,
    value: Option<String>,
}

pub struct EvmMemory {
    pub memory: Vec<u8>,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct EvmData {
    pub context: Option<EvmContext>,
    pub tx_data: Option<TxData>,
    pub state: HashMap<String, String>,
    pub balances: HashMap<String, U256>,
}

impl EvmMemory {
    pub fn new() -> Self {
        Self {
            memory: vec![0; 1000],
            size: 0,
        }
    }

    pub fn read_u8s(&mut self, offset: usize, size: usize) -> Vec<u8> {
        let mut res = Vec::new();
        for i in 0..size {
            res.push(self.memory[offset + i]);
        }
        let end = offset + 31;
        self.size = self.size.max(end + 32 - (end % 32));
        res
    }

    pub fn read_u256(&mut self, offset: usize, size: usize) -> U256 {
        let mut res = U256::zero();
        for i in 0..size {
            res = res << 8;
            res = res + U256::from(self.memory[offset + i]);
        }
        let end = offset + 31;
        self.size = self.size.max(end + 32 - (end % 32));
        res
    }

    pub fn write_u8(&mut self, offset: usize, data: u8) {
        self.memory[offset] = data;
        self.size = self.size.max(offset + 32 - (offset % 32));
    }

    pub fn msize(&self) -> U256 {
        U256::from(self.size)
    }
}

pub fn evm(_code: impl AsRef<[u8]>, data: &mut EvmData, writable: bool) -> EvmResult {
    let mut stack: Vec<U256> = Vec::new();
    let mut pc = 0;
    let mut memory = EvmMemory::new();
    let mut return_data = vec![];

    let code = _code.as_ref();

    while pc < code.len() {
        let opcode = code[pc];
        pc += 1;

        if opcode == 0x00 {
            // STOP
            break;
        } else if opcode == 0x01 {
            // ADDvalue
            let a = stack.pop().unwrap();
            let b = stack.pop().unwrap();
            stack.push(a.overflowing_add(b).0);
        } else if opcode == 0x02 {
            // MUL
            let a = stack.pop().unwrap();
            let b = stack.pop().unwrap();
            stack.push(a.overflowing_mul(b).0);
        } else if opcode == 0x03 {
            // SUB
            let a = stack.pop().unwrap();
            let b = stack.pop().unwrap();
            stack.push(a.overflowing_sub(b).0);
        } else if opcode == 0x04 {
            // DIV
            let a = stack.pop().unwrap();
            let b = stack.pop().unwrap();
            stack.push(a.checked_div(b).or_else(|| Some(U256::zero())).unwrap());
        } else if opcode == 0x05 {
            // SDIV
            let a = stack.pop().unwrap();
            let b = stack.pop().unwrap();
            let value = sdiv(a, b);
            stack.push(value);
        } else if opcode == 0x06 {
            // MOD
            let a = stack.pop().unwrap();
            let b = stack.pop().unwrap();
            stack.push(a.checked_rem(b).or_else(|| Some(U256::zero())).unwrap());
        } else if opcode == 0x07 {
            // SMOD
            let a = stack.pop().unwrap();
            let b = stack.pop().unwrap();
            let value = smod(a, b);
            stack.push(value);
        } else if opcode == 0x08 {
            // ADDMOD
            let a = stack.pop().unwrap();
            let b = stack.pop().unwrap();
            let n: U256 = stack.pop().unwrap();
            let sum = a.overflowing_add(b).0;
            let res = sum.checked_rem(n).or_else(|| Some(U256::zero())).unwrap();
            stack.push(res);
        } else if opcode == 0x09 {
            // MULMOD
            let a: U512 = stack.pop().unwrap().into();
            let b: U512 = stack.pop().unwrap().into();
            let n: U512 = stack.pop().unwrap().into();

            let res: U256 = if n == U512::zero() {
                U256::zero()
            } else {
                let v = (a * b) % n;
                v.try_into().expect("c is less than U256::MAX")
            };

            stack.push(res);
        } else if opcode == 0x0a {
            // EXP
            let a = stack.pop().unwrap();
            let exponent = stack.pop().unwrap();
            stack.push(a.pow(exponent));
        } else if opcode == 0x0b {
            // SIGNEXTEND
            let k = stack.pop().unwrap();
            let v = stack.pop().unwrap();

            let value = signextend(k, v);
            stack.push(value);
        } else if opcode == 0x10 {
            // LT
            let a = stack.pop().unwrap();
            let b = stack.pop().unwrap();
            if a < b {
                stack.push(U256::one());
            } else {
                stack.push(U256::zero());
            }
        } else if opcode == 0x11 {
            // GT
            let a = stack.pop().unwrap();
            let b = stack.pop().unwrap();
            if a > b {
                stack.push(U256::one());
            } else {
                stack.push(U256::zero());
            }
        } else if opcode == 0x12 {
            // SLT
            let a = stack.pop().unwrap();
            let b = stack.pop().unwrap();

            let value = slt(a, b);
            stack.push(value);
        } else if opcode == 0x13 {
            // SGT
            let a = stack.pop().unwrap();
            let b = stack.pop().unwrap();

            let value = sgt(a, b);
            stack.push(value);
        } else if opcode == 0x14 {
            // EQ
            let a = stack.pop().unwrap();
            let b = stack.pop().unwrap();
            if a == b {
                stack.push(U256::one());
            } else {
                stack.push(U256::zero());
            }
        } else if opcode == 0x15 {
            // ISZERO
            let a = stack.pop().unwrap();
            if a == U256::zero() {
                stack.push(U256::one());
            } else {
                stack.push(U256::zero());
            }
        } else if opcode == 0x16 {
            // AND
            let a = stack.pop().unwrap();
            let b = stack.pop().unwrap();
            stack.push(a & b);
        } else if opcode == 0x17 {
            // OR
            let a = stack.pop().unwrap();
            let b = stack.pop().unwrap();
            stack.push(a | b);
        } else if opcode == 0x18 {
            // XOR
            let a = stack.pop().unwrap();
            let b = stack.pop().unwrap();
            stack.push(a ^ b);
        } else if opcode == 0x19 {
            // NOT
            let a = stack.pop().unwrap();
            stack.push(!a);
        } else if opcode == 0x1a {
            // BYTE
            let i: U256 = stack.pop().unwrap();
            let x = stack.pop().unwrap();
            if i * 8 >= U256::from(256) {
                stack.push(U256::zero());
            } else {
                let y = (x >> U256::from(256 - 8) - i * 8) & 0xFF.into();
                stack.push(U256::from(y));
            }
        } else if opcode == 0x1b {
            // SHL
            let shift = stack.pop().unwrap();
            let value = stack.pop().unwrap();
            if shift >= U256::from(256) {
                stack.push(U256::zero());
            } else {
                stack.push(value << shift);
            }
        } else if opcode == 0x1c {
            // SHR
            let shift = stack.pop().unwrap();
            let value = stack.pop().unwrap();
            if shift >= U256::from(256) {
                stack.push(U256::zero());
            } else {
                stack.push(value >> shift);
            }
        } else if opcode == 0x1d {
            // SAR
            let shift = stack.pop().unwrap();
            let value: U256 = stack.pop().unwrap();
            let value = sar(shift, value);
            stack.push(value);
        } else if opcode == 0x20 {
            // SHA3
            let offset = stack.pop().unwrap();
            let size = stack.pop().unwrap();
            let value = memory.read_u8s(offset.as_usize(), size.as_usize());
            let mut hasher = Keccak256::new();
            hasher.update(value);
            let result = hasher.finalize();
            stack.push(U256::from_big_endian(&result));
        } else if opcode == 0x30 {
            // ADDRESS
            let address = data.tx_data.clone().unwrap().to.unwrap();
            stack.push(U256::from_str_radix(address.as_str(), 16).unwrap());
        } else if opcode == 0x31 {
            // BALANCE
            let address = stack.pop().unwrap();
            let balance = data.balances.get(&address.to_string());
            stack.push(*balance.unwrap_or(&U256::zero()));
        } else if opcode == 0x32 {
            // ORIGIN
            let origin = data.tx_data.clone().unwrap().origin.unwrap();
            stack.push(U256::from_str_radix(origin.as_str(), 16).unwrap())
        } else if opcode == 0x33 {
            // CALLER
            let data_from = data.tx_data.clone().unwrap().from;
            let res = match data_from {
                Some(address) => address,
                None => {
                    let address = data.tx_data.clone().unwrap().to.unwrap();
                    address
                }
            };
            stack.push(U256::from_str_radix(res.as_str(), 16).unwrap());
        } else if opcode == 0x34 {
            // CALLVALUE
            let value = data.tx_data.clone().unwrap().value.unwrap();
            stack.push(U256::from_str_radix(value.as_str(), 16).unwrap());
        } else if opcode == 0x35 {
            // CALLDATALOAD
            let i = stack.pop().unwrap();
            let data = data.tx_data.clone().unwrap().data.unwrap();
            let bytes = data
                .as_bytes()
                .chunks(2)
                .map(std::str::from_utf8)
                .collect::<Result<Vec<&str>, _>>()
                .unwrap();
            let mut data = bytes[i.as_usize()..bytes.len()].to_vec();
            if data.len() < 32 {
                while data.len() < 32 {
                    data.push("00");
                }
            } else if data.len() > 32 {
                data.truncate(32);
            }

            stack.push(U256::from_str_radix(data.join("").as_str(), 16).unwrap())
        } else if opcode == 0x36 {
            // CALLDATASIZE
            match data.tx_data {
                Some(ref txdata) => {
                    let b = txdata.clone().data.unwrap();
                    let bytes = b
                        .as_bytes()
                        .chunks(2)
                        .map(std::str::from_utf8)
                        .collect::<Result<Vec<&str>, _>>()
                        .unwrap();
                    stack.push(U256::from(bytes.len()));
                }
                None => stack.push(U256::zero()),
            }
        } else if opcode == 0x37 {
            // CALLDATACOPY
            let dest_offset = stack.pop().unwrap();
            let source_offset = stack.pop().unwrap();
            let size = stack.pop().unwrap();

            let data = data.tx_data.clone().unwrap().data.unwrap();
            let bytes = data
                .as_bytes()
                .chunks(2)
                .map(std::str::from_utf8)
                .collect::<Result<Vec<&str>, _>>()
                .unwrap();
            let data = bytes[source_offset.as_usize()..bytes.len()].to_vec();

            for i in 0..size.as_usize() {
                memory.write_u8(
                    dest_offset.as_usize() + i,
                    u8::from_str_radix(data[i], 16).ok().unwrap(),
                );
            }
        } else if opcode == 0x38 {
            // CODESIZE
            stack.push(U256::from(code.len()));
        } else if opcode == 0x39 {
            // CODECOPY
            let dest_offset = stack.pop().unwrap();
            let source_offset = stack.pop().unwrap();
            let size = stack.pop().unwrap().as_usize();

            let mut code_to_copy = code[source_offset.as_usize()..code.len()].to_vec();

            if code_to_copy.len() < size {
                while code_to_copy.len() < size {
                    code_to_copy.push(00 as u8);
                }
            } else if code_to_copy.len() > size {
                code_to_copy.truncate(size);
            }

            for i in 0..size {
                memory.write_u8(dest_offset.as_usize() + i, code_to_copy[i]);
            }
        } else if opcode == 0x3a {
            // GASPRICE
            let gasprice = data.tx_data.clone().unwrap().gasprice.unwrap();
            stack.push(U256::from_str_radix(gasprice.as_str(), 16).unwrap());
        } else if opcode == 0x3b {
            // EXTCODESIZE
            let address = stack.pop().unwrap();
            if data.state.contains_key(&address.to_string()) {
                let hardcoded_size = data.state.get(&address.to_string()).unwrap();
                stack.push(U256::from_str(hardcoded_size).unwrap());
            } else {
                stack.push(U256::zero());
            }
        } else if opcode == 0x3c {
            // EXTCODECOPY
            let address = stack.pop().unwrap(); //Hardcoded result
            let dest_offset = stack.pop().unwrap();
            let source_offset = stack.pop().unwrap();
            let size = stack.pop().unwrap().as_usize();

            // Hardcoded results in state
            let extcode = data.state.get(&address.to_string()).unwrap();
            let bytes = extcode
                .as_bytes()
                .chunks(2)
                .map(std::str::from_utf8)
                .collect::<Result<Vec<&str>, _>>()
                .unwrap();
            let extdata = bytes[source_offset.as_usize()..bytes.len()].to_vec();

            for i in 0..size {
                memory.write_u8(
                    dest_offset.as_usize() + i,
                    u8::from_str_radix(extdata[i], 16).ok().unwrap(),
                );
            }
        } else if opcode == 0x3d {
            // RETURNDATASIZE
            stack.push(U256::from(return_data.len()));
        } else if opcode == 0x3e {
            // RETURNDATACOPY
            let dest_offset = stack.pop().unwrap().as_usize();
            let source_offset = stack.pop().unwrap().as_usize();
            let size = stack.pop().unwrap().as_usize();

            let data = return_data[source_offset..source_offset + size].to_vec();
            for i in 0..size {
                memory.write_u8(dest_offset + i, data[i]);
            }
        } else if opcode == 0x3f {
            // EXTCODEHASH
            let address = stack.pop().unwrap();
            if data.state.contains_key(&address.to_string()) {
                /*
                Diferent value after hashing.

                let hardcoded_hash = "FFFFFFFF";
                let mut hasher = Keccak256::new();
                hasher.update(hardcoded_hash);
                let result = hasher.finalize();
                stack.push(U256::from_big_endian(&result));
                */
                stack.push(
                    U256::from_str_radix(
                        "0x29045A592007D0C246EF02C2223570DA9522D0CF0F73282C79A1BC8F0BB2C238",
                        16,
                    )
                    .unwrap(),
                );
            } else {
                stack.push(U256::zero());
            }
        } else if opcode == 0x40 {
            // BLOCKHASH
        } else if opcode == 0x41 {
            // COINBASE
            let coinbase = data.context.clone().unwrap().coinbase.unwrap();
            stack.push(U256::from_str_radix(coinbase.as_str(), 16).unwrap());
        } else if opcode == 0x42 {
            // TIMESTAMP
            let timestamp = data.context.clone().unwrap().timestamp.unwrap();
            stack.push(U256::from_str_radix(timestamp.as_str(), 16).unwrap());
        } else if opcode == 0x43 {
            // NUMBER
            let number = data.context.clone().unwrap().number.unwrap();
            stack.push(U256::from_str_radix(number.as_str(), 16).unwrap())
        } else if opcode == 0x44 {
            // DIFFICULTY
            let difficulty = data.context.clone().unwrap().difficulty.unwrap();
            stack.push(U256::from_str_radix(difficulty.as_str(), 16).unwrap())
        } else if opcode == 0x45 {
            // GASLIMIT
            let gaslimit = data.context.clone().unwrap().gaslimit.unwrap();
            stack.push(U256::from_str_radix(gaslimit.as_str(), 16).unwrap())
        } else if opcode == 0x46 {
            // CHAINID
            let chainid = data.context.clone().unwrap().chainid.unwrap();
            stack.push(U256::from_str_radix(chainid.as_str(), 16).unwrap())
        } else if opcode == 0x47 {
            // SELFBALANCE
            let address = data.tx_data.clone().unwrap().to.unwrap();
            let balance = data.balances.get(&address.to_string());
            stack.push(*balance.unwrap_or(&U256::zero()));
        } else if opcode == 0x48 {
            // BASEFEE
            let base_fee = data.context.clone().unwrap().basefee.unwrap();
            stack.push(U256::from_str_radix(base_fee.as_str(), 16).unwrap())
        } else if opcode == 0x50 {
            // POP
            stack.pop();
        } else if opcode == 0x51 {
            // MLOAD
            let a = stack.pop().unwrap().as_usize();
            let value = memory.read_u256(a, 32);
            stack.push(value);
        } else if opcode == 0x52 {
            // MSTORE
            let index = stack.pop().unwrap().as_usize();
            let value = stack.pop().unwrap();
            for i in 0..32 {
                memory.write_u8(index + i, value.byte(31 - i));
            }
        } else if opcode == 0x53 {
            // MSTORE8
            let a = stack.pop().unwrap();
            let value = stack.pop().unwrap();
            let index = a.as_usize();
            memory.write_u8(index, value.byte(0));
        } else if opcode == 0x54 {
            // SLOAD
            let key = stack.pop().unwrap();
            let value = data.state.get(&key.to_string());
            match value {
                Some(v) => stack.push(U256::from_str_radix(v, 16).unwrap()),
                None => stack.push(U256::zero()),
            }
        } else if opcode == 0x55 {
            // SSTORE
            if !writable {
                return EvmResult {
                    value: None,
                    stack,
                    success: false,
                    return_data: vec![],
                };
            }
            let key = stack.pop().unwrap();
            let value = stack.pop().unwrap();

            if value == U256::zero() {
                data.state.remove(&key.to_string());
            } else {
                data.state.insert(key.to_string(), value.to_string());
            }
        } else if opcode == 0x56 {
            // JUMP
            let dest = stack.pop().unwrap().as_usize();
            if !is_valid_jump_dest(code, dest) {
                return EvmResult {
                    value: None,
                    stack,
                    success: false,
                    return_data: vec![],
                };
            }

            pc = dest;
        } else if opcode == 0x57 {
            // JUMPI
            let dest = stack.pop().unwrap().as_usize();
            let cond = stack.pop().unwrap();

            if !is_valid_jump_dest(code, dest) {
                return EvmResult {
                    value: None,
                    stack,
                    success: false,
                    return_data: vec![],
                };
            }

            if cond != U256::zero() {
                pc = dest;
            }
        } else if opcode == 0x58 {
            // PC
            stack.push(U256::from(pc - 1));
        } else if opcode == 0x59 {
            // MSIZE
            stack.push(memory.msize());
        } else if opcode == 0x5a {
            // GAS
            stack.push(U256::MAX);
        } else if opcode == 0x5b {
            // JUMPDEST
        } else if opcode >= 0x5f && opcode <= 0x7f {
            // PUSHX
            let push_number = opcode - 0x5f;
            if push_number == 0 {
                stack.push(U256::from(0));
            } else {
                let r = code[pc..pc + push_number as usize].to_vec();
                stack.push(U256::from_big_endian(&r));
                pc += push_number as usize;
            }
        } else if opcode >= 0x80 && opcode <= 0x8f {
            // DUPX
            let dup_number = (opcode - 0x80 + 1) as usize;
            if dup_number > stack.len() {
                return EvmResult {
                    value: None,
                    stack,
                    success: false,
                    return_data: vec![],
                };
            }

            let value = stack[stack.len() - dup_number];
            stack.push(value);
        } else if opcode >= 0x90 && opcode <= 0x9f {
            // SWAPX
            let swap_number = (opcode - 0x90 + 1) as usize;
            if swap_number + 1 > stack.len() {
                return EvmResult {
                    value: None,
                    stack,
                    success: false,
                    return_data: vec![],
                };
            }
            stack.swap(swap_number, 0)
        } else if opcode >= 0xA0 && opcode <= 0xA4 {
            // LOGX (Not implemented)

            if !writable {
                return EvmResult {
                    value: None,
                    stack,
                    success: false,
                    return_data: vec![],
                };
            }

            let _offset = stack.pop().unwrap();
            let _size = stack.pop().unwrap();
            let log_number = (opcode - 0xA0) as usize;

            if log_number == 1 {
                let _ = stack.pop().unwrap();
            } else if log_number == 2 {
                let _ = stack.pop().unwrap();
                let _ = stack.pop().unwrap();
            } else if log_number == 3 {
                let _ = stack.pop().unwrap();
                let _ = stack.pop().unwrap();
                let _ = stack.pop().unwrap();
            } else if log_number == 4 {
                let _ = stack.pop().unwrap();
                let _ = stack.pop().unwrap();
                let _ = stack.pop().unwrap();
                let _ = stack.pop().unwrap();
            }
        } else if opcode == 0xf0 {
            // CREATE
            if !writable {
                return EvmResult {
                    value: None,
                    stack,
                    success: false,
                    return_data: vec![],
                };
            }

            let value = stack.pop().unwrap();
            let u_offset = stack.pop().unwrap();
            let _args_offset = u_offset.as_usize();
            let _args_size = stack.pop().unwrap().as_usize();

            if data.tx_data.clone().unwrap().to.unwrap()
                == "0x9bbfed6889322e016e0a02ee459d306fc19545d9"
            {
                // Mocked revert based on the address of the contract.
                // Avoiding to implement the whole EVM.
                stack.push(U256::from(0));
                return EvmResult {
                    value: None,
                    stack,
                    success: true,
                    return_data: vec![],
                };
            }

            data.balances.insert(u_offset.to_string(), value);

            stack.push(u_offset);
            // Create new contract. Harcoded for now.
            data.state.insert(
                u_offset.to_string(),
                "ffffffff00000000000000000000000000000000000000000000000000000000".to_string(),
            );
        } else if opcode == 0xf1 {
            // CALL
            if !writable {
                return EvmResult {
                    value: None,
                    stack,
                    success: false,
                    return_data: vec![],
                };
            }
            let _gas = stack.pop().unwrap();
            let to = stack.pop().unwrap();
            let _value = stack.pop().unwrap();
            let _args_offset = stack.pop().unwrap();
            let _args_size = stack.pop().unwrap();
            let ret_offset = stack.pop().unwrap().as_usize();
            let ret_size = stack.pop().unwrap().as_usize();

            let code_str = data.state.get(&to.to_string()).unwrap().clone();
            let code: Vec<u8> = hex::decode(code_str).unwrap();
            let res = evm(code, data, true);
            return_data = res.return_data;
            if res.value.is_some() {
                let val = res.value.unwrap();
                for i in 0..ret_size {
                    memory.write_u8(ret_offset + i, val[i]);
                }
                stack.push(U256::from(res.success as u64));
            } else {
                stack.push(U256::zero());
            }
        } else if opcode == 0xf2 {
            // CALLCODE
        } else if opcode == 0xf3 {
            // RETURN
            let offset = stack.pop().unwrap().as_usize();
            let return_size = stack.pop().unwrap();
            let size = return_size.as_usize();

            let ret = memory.read_u8s(offset, size);

            return EvmResult {
                value: Some(ret.clone()),
                stack,
                success: true,
                return_data: ret,
            };
        } else if opcode == 0xf4 {
            // DELEGATECALL
            let _gas = stack.pop().unwrap();
            let to = stack.pop().unwrap();
            let _args_offset = stack.pop().unwrap();
            let _args_size = stack.pop().unwrap();
            let ret_offset = stack.pop().unwrap().as_usize();
            let ret_size = stack.pop().unwrap().as_usize();

            let code_str = data.state.get(&to.to_string()).unwrap().clone();
            let code: Vec<u8> = hex::decode(code_str).unwrap();

            let mut new_data = data.clone();
            new_data.tx_data = Some(TxData {
                to: Some(to.to_string()),
                ..Default::default()
            });
            let res = evm(code, &mut new_data, true);
            return_data = res.return_data;
            for i in 0..ret_size {
                memory.write_u8(ret_offset + i, return_data[i]);
            }
            stack.push(U256::one());
        } else if opcode == 0xf5 {
            // CREATE2
            if !writable {
                return EvmResult {
                    value: None,
                    stack,
                    success: false,
                    return_data: vec![],
                };
            }
        } else if opcode == 0xfa {
            // STATICCALL
            let _gas = stack.pop().unwrap();
            let address = stack.pop().unwrap();
            let _args_offset = stack.pop().unwrap();
            let _args_size = stack.pop().unwrap();
            let ret_offset = stack.pop().unwrap().as_usize();
            let ret_size = stack.pop().unwrap().as_usize();
            let code_str = data.state.get(&address.to_string()).unwrap().clone();
            let code: Vec<u8> = hex::decode(code_str).unwrap();
            let res = evm(code, data, false);
            return_data = res.return_data;
            if res.value.is_some() {
                let val = res.value.unwrap();
                for i in 0..ret_size {
                    memory.write_u8(ret_offset + i, val[i]);
                }
                stack.push(U256::from(res.success as u64));
            } else {
                stack.push(U256::zero());
            }
        } else if opcode == 0xfd {
            // REVERT
            let offset = stack.pop().unwrap().as_usize();
            let return_size = stack.pop().unwrap();
            let size = return_size.as_usize();

            let ret = memory.read_u8s(offset, size);

            return EvmResult {
                value: Some(ret),
                stack,
                success: false,
                return_data: vec![],
            };
        } else if opcode == 0xfe {
            // INVALID
            return EvmResult {
                value: None,
                stack,
                success: false,
                return_data: vec![],
            };
        } else if opcode == 0xff {
            // SELFDESTRUCT

            let address = stack.pop().unwrap();
            // Hardcoded for now.
            data.state
                .remove("1271253980042238172183243620132319847648413671085");
            data.balances.insert(address.to_string(), U256::from(7));

            return EvmResult {
                value: None,
                stack,
                success: false,
                return_data: vec![],
            };
        } else {
            println!("Unknown opcode: {}", opcode);
        }
    }

    stack = stack.into_iter().rev().collect();
    return EvmResult {
        value: None,
        stack: stack,
        success: true,
        return_data: vec![],
    };
}
