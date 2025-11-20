use pinocchio::{
    account_info::AccountInfo, cpi::invoke, instruction::AccountMeta, instruction::Instruction,
    ProgramResult,
};
use crate::error::PinocchioCpiError;

const CLMM_INSTRUCTION_DATA: [u8; 41] = [
    // swap_v2 discriminator [0..8]
    43, 4, 237, 11, 26, 201, 30, 98,
    // amount placeholder [8..16] - 将被替换
    0, 0, 0, 0, 0, 0, 0, 0,
    // other_amount_threshold = 0 [16..24]
    0, 0, 0, 0, 0, 0, 0, 0,
    // sqrt_price_limit = 0 [24..40]
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    // is_base_input = true [40]
    1,
];

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

    // 🚀 优化：预构建模板，只替换变量部分
    let mut instruction_data = CLMM_INSTRUCTION_DATA;
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());

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

pub fn execute_clmm_swap_hop3(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    clmm_accounts: &[AccountInfo],
    step: u8,
    is_wsol_token_0: bool,
) -> ProgramResult {
    match step {
        1 => {
            execute_clmm_swap(trade_amount, header_accounts, clmm_accounts, true, is_wsol_token_0)
        }
        2 => {
            execute_clmm_swap_mid(trade_amount, header_accounts, clmm_accounts, is_wsol_token_0)
        }
        3 => {
            execute_clmm_swap_sell(trade_amount, header_accounts, clmm_accounts, is_wsol_token_0)
        }
        _ => {
            Err(PinocchioCpiError::UnsupportedPoolType.into())
        }
    }
}

fn execute_clmm_swap_mid(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    clmm_accounts: &[AccountInfo],
    is_mid_zero_to_one: bool,
) -> ProgramResult {
    // 中间交换：Token1 -> Token2
    // 输入：header_accounts[8] (token1_account)
    // 输出：header_accounts[11] (token2_account)
    
    let (input_vault_index, output_vault_index, input_mint, output_mint) = if is_mid_zero_to_one {
        // Token1是token0，Token2是token1
        (5, 6, &header_accounts[6], &header_accounts[9])
    } else {
        // Token1是token1，Token2是token0
        (6, 5, &header_accounts[9], &header_accounts[6])
    };

    let account_metas = [
        AccountMeta::new(header_accounts[0].key(), true, true),   // payer (signer)
        AccountMeta::new(clmm_accounts[2].key(), false, false),  // amm_config (readonly)
        AccountMeta::new(clmm_accounts[1].key(), true, false),   // pool_state (writable)
        AccountMeta::new(header_accounts[8].key(), true, false), // input_token_account (token1)
        AccountMeta::new(header_accounts[11].key(), true, false), // output_token_account (token2)
        AccountMeta::new(clmm_accounts[input_vault_index].key(), true, false), // input_vault
        AccountMeta::new(clmm_accounts[output_vault_index].key(), true, false), // output_vault
        AccountMeta::new(clmm_accounts[3].key(), true, false),   // observation_state (writable)
        AccountMeta::new(header_accounts[3].key(), false, false), // token_program (readonly)
        AccountMeta::new(header_accounts[4].key(), false, false), // token_program_2022 (readonly)
        AccountMeta::new(header_accounts[5].key(), false, false), // memo_program (readonly)
        AccountMeta::new(input_mint.key(), false, false),        // input_vault_mint (readonly)
        AccountMeta::new(output_mint.key(), false, false),       // output_vault_mint (readonly)
        AccountMeta::new(clmm_accounts[4].key(), false, false),  // bitmap_extension (readonly)
        AccountMeta::new(clmm_accounts[7].key(), true, false),   // tick_array_minus_1 (writable)
        AccountMeta::new(clmm_accounts[8].key(), true, false),   // tick_array_0 (writable)
        AccountMeta::new(clmm_accounts[9].key(), true, false),   // tick_array_1 (writable)
    ];

    let mut instruction_data = CLMM_INSTRUCTION_DATA;
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());

    let swap_instruction = Instruction {
        program_id: clmm_accounts[0].key(),
        accounts: &account_metas,
        data: &instruction_data,
    };

    let base_accounts = [
        &header_accounts[0],                       // payer
        &clmm_accounts[2],                         // amm_config
        &clmm_accounts[1],                         // pool_state
        &header_accounts[8],                       // input_token_account (token1)
        &header_accounts[11],                      // output_token_account (token2)
        &clmm_accounts[input_vault_index],         // input_vault
        &clmm_accounts[output_vault_index],        // output_vault
        &clmm_accounts[3],                         // observation_state
        &header_accounts[3],                       // token_program
        &header_accounts[4],                       // token_program_2022
        &header_accounts[5],                       // memo_program
        input_mint,                                // input_vault_mint
        output_mint,                               // output_vault_mint
        &clmm_accounts[4],                         // bitmap_extension
        &clmm_accounts[7],                         // tick_array_minus_1
        &clmm_accounts[8],                         // tick_array_0
        &clmm_accounts[9],                         // tick_array_1
    ];

    invoke::<17>(&swap_instruction, &base_accounts)
}

fn execute_clmm_swap_sell(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    clmm_accounts: &[AccountInfo],
    is_wsol_token_0: bool,
) -> ProgramResult {
    // 卖出交换：Token2 -> WSOL
    // 输入：header_accounts[11] (token2_account)
    // 输出：header_accounts[2] (wsol_account)
    
    let (input_vault_index, output_vault_index, input_mint, output_mint) = if is_wsol_token_0 {
        // WSOL是token0，Token2是token1
        (6, 5, &header_accounts[9], &header_accounts[1])
    } else {
        // WSOL是token1，Token2是token0
        (5, 6, &header_accounts[9], &header_accounts[1])
    };

    let account_metas = [
        AccountMeta::new(header_accounts[0].key(), true, true),   // payer (signer)
        AccountMeta::new(clmm_accounts[2].key(), false, false),  // amm_config (readonly)
        AccountMeta::new(clmm_accounts[1].key(), true, false),   // pool_state (writable)
        AccountMeta::new(header_accounts[11].key(), true, false), // input_token_account (token2)
        AccountMeta::new(header_accounts[2].key(), true, false), // output_token_account (wsol)
        AccountMeta::new(clmm_accounts[input_vault_index].key(), true, false), // input_vault
        AccountMeta::new(clmm_accounts[output_vault_index].key(), true, false), // output_vault
        AccountMeta::new(clmm_accounts[3].key(), true, false),   // observation_state (writable)
        AccountMeta::new(header_accounts[3].key(), false, false), // token_program (readonly)
        AccountMeta::new(header_accounts[4].key(), false, false), // token_program_2022 (readonly)
        AccountMeta::new(header_accounts[5].key(), false, false), // memo_program (readonly)
        AccountMeta::new(input_mint.key(), false, false),        // input_vault_mint (readonly)
        AccountMeta::new(output_mint.key(), false, false),       // output_vault_mint (readonly)
        AccountMeta::new(clmm_accounts[4].key(), false, false),  // bitmap_extension (readonly)
        AccountMeta::new(clmm_accounts[7].key(), true, false),   // tick_array_minus_1 (writable)
        AccountMeta::new(clmm_accounts[8].key(), true, false),   // tick_array_0 (writable)
        AccountMeta::new(clmm_accounts[9].key(), true, false),   // tick_array_1 (writable)
    ];

    let mut instruction_data = CLMM_INSTRUCTION_DATA;
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());

    let swap_instruction = Instruction {
        program_id: clmm_accounts[0].key(),
        accounts: &account_metas,
        data: &instruction_data,
    };

    let base_accounts = [
        &header_accounts[0],                       // payer
        &clmm_accounts[2],                         // amm_config
        &clmm_accounts[1],                         // pool_state
        &header_accounts[11],                      // input_token_account (token2)
        &header_accounts[2],                       // output_token_account (wsol)
        &clmm_accounts[input_vault_index],         // input_vault
        &clmm_accounts[output_vault_index],        // output_vault
        &clmm_accounts[3],                         // observation_state
        &header_accounts[3],                       // token_program
        &header_accounts[4],                       // token_program_2022
        &header_accounts[5],                       // memo_program
        input_mint,                                // input_vault_mint
        output_mint,                               // output_vault_mint
        &clmm_accounts[4],                         // bitmap_extension
        &clmm_accounts[7],                         // tick_array_minus_1
        &clmm_accounts[8],                         // tick_array_0
        &clmm_accounts[9],                         // tick_array_1
    ];

    invoke::<17>(&swap_instruction, &base_accounts)
}

