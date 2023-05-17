use std::collections::HashMap;

/**
 * EVM From Scratch
 * Rust template
 *
 * To work on EVM From Scratch in Rust:
 *
 * - Install Rust: https://www.rust-lang.org/tools/install
 * - Edit `rust/lib.rs`
 * - Run `cd rust && cargo run` to run the tests
 *
 * Hint: most people who were trying to learn Rust and EVM at the same
 * gave up and switched to JavaScript, Python, or Go. If you are new
 * to Rust, implement EVM in another programming language first.
 */
use evm::evm;
use evm::EvmContext;
use evm::EvmData;
use evm::TxData;
use primitive_types::U256;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Evmtest {
    name: String,
    hint: String,
    code: Code,
    expect: Expect,
    block: Option<EvmContext>,
    tx: Option<TxData>,
}

#[derive(Debug, Deserialize)]
struct Code {
    asm: String,
    bin: String,
}

#[derive(Debug, Deserialize)]
struct Expect {
    stack: Option<Vec<String>>,
    success: bool,
    // #[serde(rename = "return")]
    // ret: Option<String>,
}

fn main() {
    let text = std::fs::read_to_string("../evm.json").unwrap();
    let data: Vec<Evmtest> = serde_json::from_str(&text).unwrap();

    let total = data.len();

    for (index, test) in data.iter().enumerate() {
        println!("Test {} of {}: {}", index + 1, total, test.name);

        let code: Vec<u8> = hex::decode(&test.code.bin).unwrap();

        let mut state: HashMap<String, String> = HashMap::new();
        let mut balances: HashMap<String, U256> = HashMap::new();

        balances.insert(
            "173983468828192506341714248598145129238407026077".to_string(),
            U256::from(256),
        );

        balances.insert(
            "0x1e79b045dc29eae9fdc69673c9dcd7c53e5e159d".to_string(),
            U256::from(512),
        );

        state.insert(
            "91343852333181432387730302044767688728495786666".to_string(),
            "2".to_string(),
        );

        state.insert(
            "91343852333181432387730302044767688728495787074".to_string(),
            "60426000526001601ff3".to_string(),
        );

        state.insert(
            "91343852333181432387730302044767688728495787075".to_string(),
            "3360005260206000f3".to_string(),
        );

        state.insert(
            "91343852333181432387730302044767688728495787076".to_string(),
            "60426000526001601ffd".to_string(),
        );

        state.insert(
            "1266634752353449195776526855020778617035141537245".to_string(),
            "30600055".to_string(),
        );

        state.insert(
            "0".to_string(),
            "0x1000000000000000000000000000000000000AAA".to_string(),
        );

        state.insert(
            "91343852333181432387730302044767688728495787080".to_string(),
            "6042600055".to_string(),
        );

        let mut evm_data = EvmData {
            context: test.block.clone(),
            tx_data: test.tx.clone(),
            state: state,
            balances: balances,
        };

        let result = evm(&code, &mut evm_data, true);

        let mut expected_stack: Vec<U256> = Vec::new();
        if let Some(ref stacks) = test.expect.stack {
            for value in stacks {
                expected_stack.push(U256::from_str_radix(value, 16).unwrap());
            }
        }

        let mut matching = result.stack.len() == expected_stack.len();
        if matching {
            for i in 0..result.stack.len() {
                if result.stack[i] != expected_stack[i] {
                    matching = false;
                    break;
                }
            }
        }

        matching = matching && result.success == test.expect.success;

        if !matching {
            println!("Instructions: \n{}\n", test.code.asm);

            println!("Expected success: {:?}", test.expect.success);
            println!("Expected stack: [");
            for v in expected_stack {
                println!("  {:#X},", v);
            }
            println!("]\n");

            println!("Actual success: {:?}", result.success);
            println!("Actual stack: [");
            for v in result.stack {
                println!("  {:#X},", v);
            }
            println!("]\n");

            println!("\nHint: {}\n", test.hint);
            println!("Progress: {}/{}\n\n", index, total);
            panic!("Test failed");
        }
        println!("PASS");
    }
    println!("Congratulations!");
}
