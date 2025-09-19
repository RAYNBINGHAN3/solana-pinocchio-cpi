use pinocchio::{
    account_info::AccountInfo, cpi::invoke, instruction::AccountMeta, instruction::Instruction,
    ProgramResult,
};

pub fn execute_clmm_swap(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    clmm_accounts: &[AccountInfo],
    is_buy: bool,
    is_wsol_token_0: bool, // WSOL是否为token_0
) -> ProgramResult {
    let (
        input_token_account,
        output_token_account,
        input_vault_index,
        output_vault_index,
        input_mint,
        output_mint,
    ) = if is_buy {
        // 买入：WSOL -> Token
        if is_wsol_token_0 {
            (
                &header_accounts[2], // wsol_token_account
                &header_accounts[8], // mint_token_account
                5, // token_vault_0_index
                6, // token_vault_1_index
                &header_accounts[1], // wsol_mint
                &header_accounts[6], // token_mint
            )
        } else {
            (
                &header_accounts[2], // wsol_token_account
                &header_accounts[8], // mint_token_account
                6, // token_vault_1_index
                5, // token_vault_0_index
                &header_accounts[1], // wsol_mint
                &header_accounts[6], // token_mint
            )
        }
    } else {
        // 卖出：Token -> WSOL
        if is_wsol_token_0 {
            (
                &header_accounts[8], // mint_token_account
                &header_accounts[2], // wsol_token_account
                6, // token_vault_1_index
                5, // token_vault_0_index
                &header_accounts[6], // token_mint
                &header_accounts[1], // wsol_mint
            )
        } else {
            (
                &header_accounts[8], // mint_token_account
                &header_accounts[2], // wsol_token_account
                5, // token_vault_0_index
                6, // token_vault_1_index
                &header_accounts[6], // token_mint
                &header_accounts[1], // wsol_mint
            )
        }
    };

    // 🚀 优化：栈分配账户列表 (17个账户)
    let account_metas = [
        AccountMeta::writable_signer(header_accounts[0].key()), // payer
        AccountMeta::readonly(clmm_accounts[2].key()),          // amm_config
        AccountMeta::writable(clmm_accounts[1].key()),          // pool_state
        AccountMeta::writable(input_token_account.key()),       // input_token_account
        AccountMeta::writable(output_token_account.key()),      // output_token_account
        AccountMeta::writable(clmm_accounts[input_vault_index].key()), // input_vault
        AccountMeta::writable(clmm_accounts[output_vault_index].key()), // output_vault
        AccountMeta::writable(clmm_accounts[3].key()),          // observation_state
        AccountMeta::readonly(header_accounts[3].key()),        // token_program
        AccountMeta::readonly(header_accounts[4].key()),        // token_program_2022
        AccountMeta::readonly(header_accounts[5].key()),        // memo_program
        AccountMeta::readonly(input_mint.key()),                // input_vault_mint
        AccountMeta::readonly(output_mint.key()),               // output_vault_mint
        AccountMeta::writable(clmm_accounts[4].key()),          // bitmap_extension
        AccountMeta::writable(clmm_accounts[7].key()),          // tick_array_minus_1
        AccountMeta::writable(clmm_accounts[8].key()),          // tick_array_0
        AccountMeta::writable(clmm_accounts[9].key()),          // tick_array_1
    ];

    // 🚀 优化：栈分配指令数据，预分配精确容量
    let mut instruction_data = [0u8; 41];
    // swap_v2 discriminator
    instruction_data[0..8].copy_from_slice(&[43, 4, 237, 11, 26, 201, 30, 98]);
    // amount
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());
    // other_amount_threshold = 0
    instruction_data[16..24].copy_from_slice(&0u64.to_le_bytes());
    // sqrt_price_limit = 0 (no limit)
    instruction_data[24..40].copy_from_slice(&0u128.to_le_bytes());
    // is_base_input = true
    instruction_data[40] = 1;

    let swap_instruction = Instruction {
        program_id: clmm_accounts[0].key(),
        accounts: &account_metas,
        data: &instruction_data,
    };

    // 构建AccountInfo数组 - 使用固定大小数组
    let base_accounts = [
        &header_accounts[0], // payer
        &clmm_accounts[2], // amm_config
        &clmm_accounts[1], // pool_state
        input_token_account, // input_token_account
        output_token_account, // output_token_account
        &clmm_accounts[input_vault_index], // input_vault
        &clmm_accounts[output_vault_index], // output_vault
        &clmm_accounts[3], // observation_state
        &header_accounts[3], // token_program
        &header_accounts[4], // token_program_2022
        &header_accounts[5], // memo_program
        input_mint, // input_vault_mint
        output_mint, // output_vault_mint
        &clmm_accounts[4], // bitmap_extension
        &clmm_accounts[7], // tick_array_minus_1 (固定添加)
        &clmm_accounts[8], // tick_array_0
        &clmm_accounts[9], // tick_array_1
    ];

    invoke::<17>(&swap_instruction, &base_accounts)
}

