use pinocchio::{
    account_info::AccountInfo, cpi::invoke, instruction::AccountMeta, instruction::Instruction,
    ProgramResult,
};

pub fn execute_whirlpool_swap(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    whirlpool_accounts: &[AccountInfo],
    is_buy: bool,
    is_wsol_token_a: bool, // WSOL是否为token_a
) -> ProgramResult {
    let (
        token_program_a,
        token_program_b,
        token_account_a,
        token_account_b,
        mint_a,
        mint_b,
        a_to_b,
    ) = if is_buy {
        // 买入：WSOL -> Token
        if is_wsol_token_a {
            (
                &header_accounts[3], // token_program
                &header_accounts[7], // token_program_for_mint
                &header_accounts[2], // wsol_token_account
                &header_accounts[8], // mint_token_account
                &header_accounts[1], // wsol_mint
                &header_accounts[6], // token_mint
                true, // a_to_b = true (WSOL is token A)
            )
        } else {
            (
                &header_accounts[7], // token_program_for_mint
                &header_accounts[3], // token_program
                &header_accounts[8], // mint_token_account
                &header_accounts[2], // wsol_token_account
                &header_accounts[6], // token_mint
                &header_accounts[1], // wsol_mint
                false, // a_to_b = false (WSOL is token B)
            )
        }
    } else {
        // 卖出：Token -> WSOL
        if is_wsol_token_a {
            (
                &header_accounts[3], // token_program
                &header_accounts[7], // token_program_for_mint
                &header_accounts[2], // wsol_token_account
                &header_accounts[8], // mint_token_account
                &header_accounts[1], // wsol_mint
                &header_accounts[6], // token_mint
                false, // a_to_b = false (Token B -> WSOL A)
            )
        } else {
            (
                &header_accounts[7], // token_program_for_mint
                &header_accounts[3], // token_program
                &header_accounts[8], // mint_token_account
                &header_accounts[2], // wsol_token_account
                &header_accounts[6], // token_mint
                &header_accounts[1], // wsol_mint
                true, // a_to_b = true (Token A -> WSOL B)
            )
        }
    };

    // 构建账户列表 (15个账户)
    let account_metas = [
        AccountMeta::readonly(token_program_a.key()),       // tokenProgramA
        AccountMeta::readonly(token_program_b.key()),       // tokenProgramB
        AccountMeta::readonly(header_accounts[5].key()),    // memoProgram
        AccountMeta::writable_signer(header_accounts[0].key()), // tokenAuthority (payer)
        AccountMeta::writable(whirlpool_accounts[1].key()), // whirlpool
        AccountMeta::readonly(mint_a.key()),                // tokenMintA
        AccountMeta::readonly(mint_b.key()),                // tokenMintB
        AccountMeta::writable(token_account_a.key()),       // tokenOwnerAccountA
        AccountMeta::writable(whirlpool_accounts[3].key()), // tokenVaultA
        AccountMeta::writable(token_account_b.key()),       // tokenOwnerAccountB
        AccountMeta::writable(whirlpool_accounts[4].key()), // tokenVaultB
        AccountMeta::writable(whirlpool_accounts[5].key()), // tickArray0
        AccountMeta::writable(whirlpool_accounts[6].key()), // tickArray1
        AccountMeta::writable(whirlpool_accounts[7].key()), // tickArray2
        AccountMeta::writable(whirlpool_accounts[2].key()), // oracle
    ];

    // 构建指令数据 - swapV2指令
    let mut instruction_data = [0u8; 42];
    // swapV2 discriminator
    instruction_data[0..8].copy_from_slice(&[43, 4, 237, 11, 26, 201, 30, 98]);
    // amount
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());
    // otherAmountThreshold = 0
    instruction_data[16..24].copy_from_slice(&0u64.to_le_bytes());
    // sqrtPriceLimit = 0 (no limit)
    instruction_data[24..40].copy_from_slice(&0u128.to_le_bytes());
    // amountSpecifiedIsInput = true
    instruction_data[40] = 1;
    // aToB
    instruction_data[41] = if a_to_b { 1 } else { 0 };

    let swap_instruction = Instruction {
        program_id: whirlpool_accounts[0].key(),
        accounts: &account_metas,
        data: &instruction_data,
    };

    let account_infos = [
        token_program_a,              // tokenProgramA
        token_program_b,              // tokenProgramB
        &header_accounts[5],          // memoProgram
        &header_accounts[0],          // tokenAuthority (payer)
        &whirlpool_accounts[1],       // whirlpool
        mint_a,                       // tokenMintA
        mint_b,                       // tokenMintB
        token_account_a,              // tokenOwnerAccountA
        &whirlpool_accounts[3],       // tokenVaultA
        token_account_b,              // tokenOwnerAccountB
        &whirlpool_accounts[4],       // tokenVaultB
        &whirlpool_accounts[5],       // tickArray0
        &whirlpool_accounts[6],       // tickArray1
        &whirlpool_accounts[7],       // tickArray2
        &whirlpool_accounts[2],       // oracle
    ];

    invoke::<15>(&swap_instruction, &account_infos)
}
