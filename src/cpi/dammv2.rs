use pinocchio::{
    account_info::AccountInfo, cpi::invoke, instruction::AccountMeta, instruction::Instruction,
    ProgramResult,
};
use crate::error::PinocchioCpiError;

const DAMMV2_INSTRUCTION_DATA: [u8; 24] = [
    // swap discriminator [0..8]
    248, 198, 158, 145, 225, 117, 135, 200,
    // amount_in placeholder [8..16] - 将被替换
    0, 0, 0, 0, 0, 0, 0, 0,
    // minimum_amount_out = 0 [16..24]
    0, 0, 0, 0, 0, 0, 0, 0,
];

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

    
    // 🚀 优化：预构建模板，只替换变量部分
    let mut instruction_data = DAMMV2_INSTRUCTION_DATA;
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());

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

pub fn execute_dammv2_swap_hop3(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    dammv2_accounts: &[AccountInfo],
    step: u8,
    is_wsol_token_a: bool,
) -> ProgramResult {
    match step {
        1 => {
            execute_dammv2_swap(trade_amount, header_accounts, dammv2_accounts, true, is_wsol_token_a)
        }
        2 => {
            execute_dammv2_swap_mid(trade_amount, header_accounts, dammv2_accounts, is_wsol_token_a)
        }
        3 => {
            execute_dammv2_swap_sell(trade_amount, header_accounts, dammv2_accounts, is_wsol_token_a)
        }
        _ => {
            Err(PinocchioCpiError::UnsupportedPoolType.into())
        }
    }
}

fn execute_dammv2_swap_mid(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    dammv2_accounts: &[AccountInfo],
    is_mid_zero_to_one: bool,
) -> ProgramResult {
    // 中间交换：Token1 -> Token2
    // 输入：header_accounts[8] (token1_account)
    // 输出：header_accounts[11] (token2_account)
    
    // is_mid_zero_to_one表示是否是token0->token1的方向
    let (token_a_mint, token_b_mint, token_a_program, token_b_program) = if is_mid_zero_to_one {
        // Token1是tokenA，Token2是tokenB (token0->token1)
        (&header_accounts[6], &header_accounts[9], &header_accounts[7], &header_accounts[10])
    } else {
        // Token1是tokenB，Token2是tokenA (token1->token0)
        (&header_accounts[9], &header_accounts[6], &header_accounts[10], &header_accounts[7])
    };

    let account_metas = [
        AccountMeta::new(dammv2_accounts[2].key(), false, false), // pool_authority
        AccountMeta::new(dammv2_accounts[3].key(), true, false),  // pool
        AccountMeta::new(header_accounts[8].key(), true, false),  // user_token_in (token1)
        AccountMeta::new(header_accounts[11].key(), true, false), // user_token_out (token2)
        AccountMeta::new(dammv2_accounts[4].key(), true, false),  // token_a_vault
        AccountMeta::new(dammv2_accounts[5].key(), true, false),  // token_b_vault
        AccountMeta::new(token_a_mint.key(), false, false),       // token_a_mint
        AccountMeta::new(token_b_mint.key(), false, false),       // token_b_mint
        AccountMeta::new(header_accounts[0].key(), true, true),   // payer
        AccountMeta::new(token_a_program.key(), false, false),    // token_a_program
        AccountMeta::new(token_b_program.key(), false, false),    // token_b_program
        AccountMeta::new(dammv2_accounts[0].key(), false, false), // referral_token_account
        AccountMeta::new(dammv2_accounts[1].key(), false, false), // event_authority
        AccountMeta::new(dammv2_accounts[0].key(), false, false), // program
    ];

    let mut instruction_data = DAMMV2_INSTRUCTION_DATA;
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());

    let swap_instruction = Instruction {
        program_id: dammv2_accounts[0].key(),
        accounts: &account_metas,
        data: &instruction_data,
    };

    let account_infos = [
        &dammv2_accounts[2],      // pool_authority
        &dammv2_accounts[3],      // pool
        &header_accounts[8],      // user_token_in (token1)
        &header_accounts[11],     // user_token_out (token2)
        &dammv2_accounts[4],      // token_a_vault
        &dammv2_accounts[5],      // token_b_vault
        token_a_mint,             // token_a_mint
        token_b_mint,             // token_b_mint
        &header_accounts[0],      // payer
        token_a_program,          // token_a_program
        token_b_program,          // token_b_program
        &dammv2_accounts[0],      // referral_token_account
        &dammv2_accounts[1],      // event_authority
        &dammv2_accounts[0],      // program
    ];

    invoke::<14>(&swap_instruction, &account_infos)
}

fn execute_dammv2_swap_sell(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    dammv2_accounts: &[AccountInfo],
    is_wsol_token_a: bool,
) -> ProgramResult {
    // 卖出交换：Token2 -> WSOL
    // 输入：header_accounts[11] (token2_account)
    // 输出：header_accounts[2] (wsol_account)
    
    // is_wsol_token_a表示WSOL是否为tokenA
    let (token_a_mint, token_b_mint, token_a_program, token_b_program) = if is_wsol_token_a {
        // WSOL是tokenA，Token2是tokenB
        (&header_accounts[1], &header_accounts[9], &header_accounts[3], &header_accounts[10])
    } else {
        // WSOL是tokenB，Token2是tokenA
        (&header_accounts[9], &header_accounts[1], &header_accounts[10], &header_accounts[3])
    };

    let account_metas = [
        AccountMeta::new(dammv2_accounts[2].key(), false, false), // pool_authority
        AccountMeta::new(dammv2_accounts[3].key(), true, false),  // pool
        AccountMeta::new(header_accounts[11].key(), true, false), // user_token_in (token2)
        AccountMeta::new(header_accounts[2].key(), true, false),  // user_token_out (wsol)
        AccountMeta::new(dammv2_accounts[4].key(), true, false),  // token_a_vault
        AccountMeta::new(dammv2_accounts[5].key(), true, false),  // token_b_vault
        AccountMeta::new(token_a_mint.key(), false, false),       // token_a_mint
        AccountMeta::new(token_b_mint.key(), false, false),       // token_b_mint
        AccountMeta::new(header_accounts[0].key(), true, true),   // payer
        AccountMeta::new(token_a_program.key(), false, false),    // token_a_program
        AccountMeta::new(token_b_program.key(), false, false),    // token_b_program
        AccountMeta::new(dammv2_accounts[0].key(), false, false), // referral_token_account
        AccountMeta::new(dammv2_accounts[1].key(), false, false), // event_authority
        AccountMeta::new(dammv2_accounts[0].key(), false, false), // program
    ];

    let mut instruction_data = DAMMV2_INSTRUCTION_DATA;
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());

    let swap_instruction = Instruction {
        program_id: dammv2_accounts[0].key(),
        accounts: &account_metas,
        data: &instruction_data,
    };

    let account_infos = [
        &dammv2_accounts[2],      // pool_authority
        &dammv2_accounts[3],      // pool
        &header_accounts[11],     // user_token_in (token2)
        &header_accounts[2],      // user_token_out (wsol)
        &dammv2_accounts[4],      // token_a_vault
        &dammv2_accounts[5],      // token_b_vault
        token_a_mint,             // token_a_mint
        token_b_mint,             // token_b_mint
        &header_accounts[0],      // payer
        token_a_program,          // token_a_program
        token_b_program,          // token_b_program
        &dammv2_accounts[0],      // referral_token_account
        &dammv2_accounts[1],      // event_authority
        &dammv2_accounts[0],      // program
    ];

    invoke::<14>(&swap_instruction, &account_infos)
}
