use pinocchio::{
    account_info::AccountInfo, cpi::invoke, instruction::AccountMeta, instruction::Instruction,
    ProgramResult,
};

pub fn execute_dammv2_swap(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    dammv2_accounts: &[AccountInfo],
    is_buy: bool,
    is_wsol_token_a: bool, // WSOL是否为token_a
) -> ProgramResult {
    let (
        token_a_mint,
        token_b_mint,
        token_a_program,
        token_b_program,

    ) = if is_wsol_token_a {
        (
            &header_accounts[1], // wsol_mint
            &header_accounts[6], // token_mint
            &header_accounts[3], // token_program
            &header_accounts[7], // token_program_for_mint
        )
    } else {
        (
            &header_accounts[6], // token_mint
            &header_accounts[1], // wsol_mint
            &header_accounts[7], // token_program_for_mint
            &header_accounts[3], // token_program
        )
    };

    let (user_token_in, user_token_out) = if is_buy {
        (
            &header_accounts[2], // wsol_token_account
            &header_accounts[8], // mint_token_account
        )
    } else {
        (
            &header_accounts[8], // mint_token_account
            &header_accounts[2], // wsol_token_account
        )
    };

    // 构建账户列表 (14个账户)
    let account_metas = [
        AccountMeta::readonly(dammv2_accounts[2].key()),     // pool_authority
        AccountMeta::writable(dammv2_accounts[3].key()),    // pool
        AccountMeta::writable(user_token_in.key()),         // user_token_in
        AccountMeta::writable(user_token_out.key()),        // user_token_out
        AccountMeta::writable(dammv2_accounts[4].key()),    // token_a_vault
        AccountMeta::writable(dammv2_accounts[5].key()),    // token_b_vault
        AccountMeta::readonly(token_a_mint.key()),          // token_a_mint
        AccountMeta::readonly(token_b_mint.key()),          // token_b_mint
        AccountMeta::writable_signer(header_accounts[0].key()), // payer (signer)
        AccountMeta::readonly(token_a_program.key()),       // token_a_program
        AccountMeta::readonly(token_b_program.key()),       // token_b_program
        AccountMeta::readonly(dammv2_accounts[0].key()),    // referral_token_account (用program_id占位)
        AccountMeta::readonly(dammv2_accounts[1].key()),    // event_authority
        AccountMeta::readonly(dammv2_accounts[0].key()),    // program
    ];

    let mut instruction_data = [0u8; 24];
    // swap discriminator
    instruction_data[0..8].copy_from_slice(&[248, 198, 158, 145, 225, 117, 135, 200]);
    // amount_in
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());
    // minimum_amount_out = 0
    instruction_data[16..24].copy_from_slice(&0u64.to_le_bytes());

    let swap_instruction = Instruction {
        program_id: dammv2_accounts[0].key(),
        accounts: &account_metas,
        data: &instruction_data,
    };

    let account_infos = [
        &dammv2_accounts[2],  // pool_authority
        &dammv2_accounts[3],  // pool
        user_token_in,        // user_token_in
        user_token_out,       // user_token_out
        &dammv2_accounts[4],  // token_a_vault
        &dammv2_accounts[5],  // token_b_vault
        token_a_mint,         // token_a_mint
        token_b_mint,         // token_b_mint
        &header_accounts[0],  // payer
        token_a_program,      // token_a_program
        token_b_program,      // token_b_program
        &dammv2_accounts[0],  // referral_token_account
        &dammv2_accounts[1],  // event_authority
        &dammv2_accounts[0],  // program
    ];

    invoke::<14>(&swap_instruction, &account_infos)
}
