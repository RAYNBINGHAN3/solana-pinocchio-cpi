use pinocchio::{
    account_info::AccountInfo, cpi::invoke, instruction::AccountMeta, instruction::Instruction,
    ProgramResult
};

pub fn execute_pump_swap(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    pump_accounts: &[AccountInfo],
    is_buy: bool,
    is_wsol_base: bool, // WSOL是否为base mint
    pump_base_amount_out: u64,
) -> ProgramResult {
    // 根据WSOL位置确定base/quote mint和program
    let (
        base_mint,
        quote_mint,
        base_token_program,
        quote_token_program,
        user_base_token_account,
        user_quote_token_account,
    ) = if is_wsol_base {
        (
            &header_accounts[1], // wsol_mint
            &header_accounts[6], // token_mint
            &header_accounts[3], // token_program
            &header_accounts[7], // token_program_for_mint
            &header_accounts[2], // wsol_token_account
            &header_accounts[8], // mint_token_account
        )
    } else {
        (
            &header_accounts[6], // token_mint
            &header_accounts[1], // wsol_mint
            &header_accounts[7], // token_program_for_mint
            &header_accounts[3], // token_program
            &header_accounts[8], // mint_token_account
            &header_accounts[2], // wsol_token_account
        )
    };
    if is_buy {
        //wosl->token
        if is_wsol_base {
            pump_sell(
                trade_amount,
                header_accounts,
                pump_accounts,
                base_mint,
                quote_mint,
                user_base_token_account,
                user_quote_token_account,
                base_token_program,
                quote_token_program,
            )
        } else {
            // let trade_amount = pump_base_amount_out;
            pump_buy(
                pump_base_amount_out,
                header_accounts,
                pump_accounts,
                base_mint,
                quote_mint,
                user_base_token_account,
                user_quote_token_account,
                base_token_program,
                quote_token_program,
            )
        }
    } else {
        //token->wosl
        if is_wsol_base { // token->wsol 但是wsol是base情况走不通。不止base out amt 草泥马
            pump_buy(
                trade_amount,
                header_accounts,
                pump_accounts,
                base_mint,
                quote_mint,
                user_base_token_account,
                user_quote_token_account,
                base_token_program,
                quote_token_program,
            )
        } else {
            pump_sell(
                trade_amount,
                header_accounts,
                pump_accounts,
                base_mint,
                quote_mint,
                user_base_token_account,
                user_quote_token_account,
                base_token_program,
                quote_token_program,
            )
        }
    }
}

fn pump_buy(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    pump_accounts: &[AccountInfo],
    base_mint: &AccountInfo,
    quote_mint: &AccountInfo,
    user_base_token_account: &AccountInfo,
    user_quote_token_account: &AccountInfo,
    base_token_program: &AccountInfo,
    quote_token_program: &AccountInfo,
) -> ProgramResult {
    // 买入：使用buy指令 (23个账户)
    let account_metas = [
        AccountMeta::readonly(pump_accounts[1].key()), // pool
        AccountMeta::writable_signer(header_accounts[0].key()), // user (signer)
        AccountMeta::readonly(pump_accounts[2].key()), // global_config
        AccountMeta::readonly(base_mint.key()),        // base_mint
        AccountMeta::readonly(quote_mint.key()),       // quote_mint
        AccountMeta::writable(user_base_token_account.key()), // user_base_token_account
        AccountMeta::writable(user_quote_token_account.key()), // user_quote_token_account
        AccountMeta::writable(pump_accounts[12].key()), // pool_base_token_account (base_vault)
        AccountMeta::writable(pump_accounts[13].key()), // pool_quote_token_account (quote_vault)
        AccountMeta::readonly(pump_accounts[6].key()), // protocol_fee_recipient (pump_fee_wallet)
        AccountMeta::writable(pump_accounts[7].key()), // protocol_fee_recipient_token_account (pump_fee_wallet_ata)
        AccountMeta::readonly(base_token_program.key()), // base_token_program
        AccountMeta::readonly(quote_token_program.key()), // quote_token_program
        AccountMeta::readonly(pump_accounts[10].key()), // system_program
        AccountMeta::readonly(pump_accounts[11].key()), // associated_token_program
        AccountMeta::readonly(pump_accounts[3].key()), // event_authority
        AccountMeta::readonly(pump_accounts[0].key()), // program
        AccountMeta::writable(pump_accounts[4].key()), // coin_creator_vault_ata
        AccountMeta::readonly(pump_accounts[5].key()), // coin_creator_vault_authority
        AccountMeta::writable(pump_accounts[8].key()), // global_volume_accumulator
        AccountMeta::writable(pump_accounts[9].key()), // user_volume_accumulator
        AccountMeta::readonly(pump_accounts[14].key()), // fee_config
        AccountMeta::readonly(pump_accounts[15].key()), // fee_program
    ];
    let mut instruction_data = [0u8; 24];
    // buy discriminator
    instruction_data[0..8].copy_from_slice(&[102, 6, 61, 18, 1, 218, 235, 234]);
    // base_amount_out (要买多少token)
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());
    // max_quote_amount_in (最多花多少quote)
    instruction_data[16..24].copy_from_slice(&u64::MAX.to_le_bytes()); 

    let swap_instruction = Instruction {
        program_id: pump_accounts[0].key(),
        accounts: &account_metas,
        data: &instruction_data,
    };
    let account_infos = [
        &pump_accounts[1],        // pool
        &header_accounts[0],      // user
        &pump_accounts[2],        // global_config
        base_mint,                // base_mint
        quote_mint,               // quote_mint
        user_base_token_account,  // user_base_token_account
        user_quote_token_account, // user_quote_token_account
        &pump_accounts[12],       // base_vault
        &pump_accounts[13],       // quote_vault
        &pump_accounts[6],        // pump_fee_wallet
        &pump_accounts[7],        // pump_fee_wallet_ata
        base_token_program,       // base_token_program
        quote_token_program,      // quote_token_program
        &pump_accounts[10],       // system_program
        &pump_accounts[11],       // associated_token_program
        &pump_accounts[3],        // event_authority
        &pump_accounts[0],        // program
        &pump_accounts[4],        // coin_creator_vault_ata
        &pump_accounts[5],        // coin_creator_vault_authority
        &pump_accounts[8],        // global_vol_accumulator
        &pump_accounts[9],        // user_vol_accumulator
        &pump_accounts[14],       // fee_config
        &pump_accounts[15],       // fee_program
    ];
    invoke::<23>(&swap_instruction, &account_infos).map_err(|e| e.into())
}

fn pump_sell(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    pump_accounts: &[AccountInfo],
    base_mint: &AccountInfo,
    quote_mint: &AccountInfo,
    user_base_token_account: &AccountInfo,
    user_quote_token_account: &AccountInfo,
    base_token_program: &AccountInfo,
    quote_token_program: &AccountInfo,
) -> ProgramResult {
    // 卖出：使用sell指令 (21个账户)
    let account_metas = [
        AccountMeta::readonly(pump_accounts[1].key()), // pool
        AccountMeta::writable_signer(header_accounts[0].key()), // user (signer)
        AccountMeta::readonly(pump_accounts[2].key()), // global_config
        AccountMeta::readonly(base_mint.key()),        // base_mint
        AccountMeta::readonly(quote_mint.key()),       // quote_mint
        AccountMeta::writable(user_base_token_account.key()), // user_base_token_account
        AccountMeta::writable(user_quote_token_account.key()), // user_quote_token_account
        AccountMeta::writable(pump_accounts[12].key()), // pool_base_token_account
        AccountMeta::writable(pump_accounts[13].key()), // pool_quote_token_account
        AccountMeta::readonly(pump_accounts[6].key()), // protocol_fee_recipient
        AccountMeta::writable(pump_accounts[7].key()), // protocol_fee_recipient_token_account
        AccountMeta::readonly(base_token_program.key()), // base_token_program
        AccountMeta::readonly(quote_token_program.key()), // quote_token_program
        AccountMeta::readonly(pump_accounts[10].key()), // system_program
        AccountMeta::readonly(pump_accounts[11].key()), // associated_token_program
        AccountMeta::readonly(pump_accounts[3].key()), // event_authority
        AccountMeta::readonly(pump_accounts[0].key()), // program
        AccountMeta::writable(pump_accounts[4].key()), // coin_creator_vault_ata
        AccountMeta::readonly(pump_accounts[5].key()), // coin_creator_vault_authority
        AccountMeta::readonly(pump_accounts[14].key()), // fee_config
        AccountMeta::readonly(pump_accounts[15].key()), // fee_program
    ];

    let mut instruction_data = [0u8; 24];
    // sell discriminator
    instruction_data[0..8].copy_from_slice(&[51, 230, 133, 164, 1, 127, 131, 173]);
    // base_amount_in (要卖多少token)
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());
    // min_quote_amount_out (最少得到多少quote)
    instruction_data[16..24].copy_from_slice(&0u64.to_le_bytes());

    let swap_instruction = Instruction {
        program_id: pump_accounts[0].key(),
        accounts: &account_metas,
        data: &instruction_data,
    };

    let account_infos = [
        &pump_accounts[1],        // pool
        &header_accounts[0],      // user
        &pump_accounts[2],        // global_config
        base_mint,                // base_mint
        quote_mint,               // quote_mint
        user_base_token_account,  // user_base_token_account
        user_quote_token_account, // user_quote_token_account
        &pump_accounts[12],       // base_vault
        &pump_accounts[13],       // quote_vault
        &pump_accounts[6],        // pump_fee_wallet
        &pump_accounts[7],        // pump_fee_wallet_ata
        base_token_program,       // base_token_program
        quote_token_program,      // quote_token_program
        &pump_accounts[10],       // system_program
        &pump_accounts[11],       // associated_token_program
        &pump_accounts[3],        // event_authority
        &pump_accounts[0],        // program
        &pump_accounts[4],        // coin_creator_vault_ata
        &pump_accounts[5],        // coin_creator_vault_authority
        &pump_accounts[14],       // fee_config
        &pump_accounts[15],       // fee_program
    ];

    invoke::<21>(&swap_instruction, &account_infos).map_err(|e| e.into())
}
