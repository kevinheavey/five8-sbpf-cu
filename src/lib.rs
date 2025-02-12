use {
    core::{
        fmt,
        str::{from_utf8, from_utf8_unchecked},
    },
    solana_account_info::AccountInfo,
    solana_program_error::ProgramResult,
    solana_pubkey::Pubkey,
};

solana_program_entrypoint::entrypoint!(process_instruction);

const MAX_BASE58_LEN: usize = 44;

struct WrapperBs58(pub Pubkey);

impl fmt::Display for WrapperBs58 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_bs58(f, &self.0)
    }
}

struct WrapperFive8(pub Pubkey);

impl fmt::Display for WrapperFive8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_five8(f, &self.0)
    }
}

fn write_bs58(f: &mut fmt::Formatter, p: &Pubkey) -> fmt::Result {
    let mut out = [0u8; MAX_BASE58_LEN];
    let out_slice: &mut [u8] = &mut out;
    // This will never fail because the only possible error is BufferTooSmall,
    // and we will never call it with too small a buffer.
    let len = bs58::encode(p.to_bytes()).onto(out_slice).unwrap();
    let as_str = from_utf8(&out[..len]).unwrap();
    f.write_str(as_str)
}

fn write_five8(f: &mut fmt::Formatter, p: &Pubkey) -> fmt::Result {
    let mut out = [0u8; MAX_BASE58_LEN];
    let len = five8::encode_32(&p.to_bytes(), &mut out) as usize;
    // any sequence of base58 chars is valid utf8
    let as_str = unsafe { from_utf8_unchecked(&out[..len]) };
    f.write_str(as_str)
}

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let first_byte = instruction_data[0];
    if first_byte == 0 {
        solana_msg::msg!("{}", WrapperBs58(*accounts[0].key));
    } else if first_byte == 1 {
        solana_msg::msg!("{}", WrapperFive8(*accounts[0].key));
    }
    Ok(())
}

#[test]
fn test_cu() {
    use litesvm::LiteSVM;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_pubkey::Pubkey;
    use solana_sdk::{message::Message, transaction::Transaction};
    let pid = Pubkey::new_unique();
    let program_bytes = include_bytes!("../target/deploy/five8_sbpf_cu.so");
    let mut svm = LiteSVM::new()
        .with_sigverify(false)
        .with_transaction_history(0);
    let payer_addr = Pubkey::new_unique();
    svm.airdrop(&payer_addr, 1000000000).unwrap();
    svm.add_program(pid, program_bytes);
    let to_log = Pubkey::new_unique();
    let ix_bs58 =
        Instruction::new_with_bytes(pid, &[0], vec![AccountMeta::new_readonly(to_log, false)]);
    let tx_bs58 = Transaction::new_unsigned(Message::new_with_blockhash(
        &[ix_bs58],
        Some(&payer_addr),
        &svm.latest_blockhash(),
    ));
    let res_bs58 = svm.send_transaction(tx_bs58).unwrap();
    println!("bs58: {}", res_bs58.compute_units_consumed);
    let ix_five8 =
        Instruction::new_with_bytes(pid, &[1], vec![AccountMeta::new_readonly(to_log, false)]);
    let tx_five8 = Transaction::new_unsigned(Message::new_with_blockhash(
        &[ix_five8],
        Some(&payer_addr),
        &svm.latest_blockhash(),
    ));
    let res_five8 = svm.send_transaction(tx_five8).unwrap();
    println!("five8: {}", res_five8.compute_units_consumed);
}
