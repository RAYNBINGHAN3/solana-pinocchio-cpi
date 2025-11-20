use pinocchio::{
    account_info::AccountInfo, cpi::invoke, instruction::AccountMeta, instruction::Instruction,
    ProgramResult,
};
use crate::error::PinocchioCpiError;

const WHIRLPOOL_INSTRUCTION_DATA: [u8; 43] = [
    // swapV2 discriminator [0..8]
    43, 4, 237, 11, 26, 201, 30, 98,
    // amount placeholder [8..16] - 将被替换
    0, 0, 0, 0, 0, 0, 0, 0,
    // otherAmountThreshold = 0 [16..24]
    0, 0, 0, 0, 0, 0, 0, 0,
    // sqrtPriceLimit = 0 [24..40]
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    // amountSpecifiedIsInput = true [40]
    1,
    // aToB placeholder [41] - 将被替换
    0,
    // remainingAccountsInfo = None [42]
    0
];

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

    // 🚀 优化：预构建模板，只替换变量部分
    let mut instruction_data = WHIRLPOOL_INSTRUCTION_DATA;
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());
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

pub fn execute_whirlpool_swap_hop3(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    whirlpool_accounts: &[AccountInfo],
    step: u8,
    is_wsol_token_a: bool,
) -> ProgramResult {
    match step {
        1 => {
            execute_whirlpool_swap(trade_amount, header_accounts, whirlpool_accounts, true, is_wsol_token_a)
        }
        2 => {
            execute_whirlpool_swap_mid(trade_amount, header_accounts, whirlpool_accounts, is_wsol_token_a)
        }
        3 => {
            execute_whirlpool_swap_sell(trade_amount, header_accounts, whirlpool_accounts, is_wsol_token_a)
        }
        _ => {
            Err(PinocchioCpiError::UnsupportedPoolType.into())
        }
    }
}

fn execute_whirlpool_swap_mid(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    whirlpool_accounts: &[AccountInfo],
    is_mid_zero_to_one: bool,
) -> ProgramResult {
    // 中间交换：Token1 -> Token2
    // 输入：header_accounts[8] (token1_account)
    // 输出：header_accounts[11] (token2_account)
    
    let (token_program_a, token_program_b, token_account_a, token_account_b, mint_a, mint_b, a_to_b) = if is_mid_zero_to_one {
        // Token1是tokenA，Token2是tokenB (token0->token1)
        (
            &header_accounts[7],  // token1_program
            &header_accounts[10], // token2_program
            &header_accounts[8],  // token1_account
            &header_accounts[11], // token2_account
            &header_accounts[6],  // token1_mint
            &header_accounts[9],  // token2_mint
            true,
        )
    } else {
        // Token1是tokenB，Token2是tokenA (token1->token0)
        (
            &header_accounts[10], // token2_program
            &header_accounts[7],  // token1_program
            &header_accounts[11], // token2_account
            &header_accounts[8],  // token1_account
            &header_accounts[9],  // token2_mint
            &header_accounts[6],  // token1_mint
            false,
        )
    };

    let account_metas = [
        AccountMeta::new(token_program_a.key(), false, false),      // tokenProgramA
        AccountMeta::new(token_program_b.key(), false, false),      // tokenProgramB
        AccountMeta::new(header_accounts[5].key(), false, false),   // memoProgram
        AccountMeta::new(header_accounts[0].key(), true, true),     // tokenAuthority (payer)
        AccountMeta::new(whirlpool_accounts[1].key(), true, false), // whirlpool
        AccountMeta::new(mint_a.key(), false, false),               // tokenMintA
        AccountMeta::new(mint_b.key(), false, false),               // tokenMintB
        AccountMeta::new(token_account_a.key(), true, false),       // tokenOwnerAccountA
        AccountMeta::new(whirlpool_accounts[3].key(), true, false), // tokenVaultA
        AccountMeta::new(token_account_b.key(), true, false),       // tokenOwnerAccountB
        AccountMeta::new(whirlpool_accounts[4].key(), true, false), // tokenVaultB
        AccountMeta::new(whirlpool_accounts[5].key(), true, false), // tickArray0
        AccountMeta::new(whirlpool_accounts[6].key(), true, false), // tickArray1
        AccountMeta::new(whirlpool_accounts[7].key(), true, false), // tickArray2
        AccountMeta::new(whirlpool_accounts[2].key(), true, false), // oracle
    ];

    let mut instruction_data = WHIRLPOOL_INSTRUCTION_DATA;
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());
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

fn execute_whirlpool_swap_sell(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    whirlpool_accounts: &[AccountInfo],
    is_wsol_token_a: bool,
) -> ProgramResult {
    // 卖出交换：Token2 -> WSOL
    // 输入：header_accounts[11] (token2_account)
    // 输出：header_accounts[2] (wsol_account)
    
    let (token_program_a, token_program_b, token_account_a, token_account_b, mint_a, mint_b, a_to_b) = if is_wsol_token_a {
        // WSOL是tokenA，Token2是tokenB
        (
            &header_accounts[3],  // wsol_program
            &header_accounts[10], // token2_program
            &header_accounts[2],  // wsol_account
            &header_accounts[11], // token2_account
            &header_accounts[1],  // wsol_mint
            &header_accounts[9],  // token2_mint
            false, // Token2 -> WSOL (B to A)
        )
    } else {
        // WSOL是tokenB，Token2是tokenA
        (
            &header_accounts[10], // token2_program
            &header_accounts[3],  // wsol_program
            &header_accounts[11], // token2_account
            &header_accounts[2],  // wsol_account
            &header_accounts[9],  // token2_mint
            &header_accounts[1],  // wsol_mint
            true, // Token2 -> WSOL (A to B)
        )
    };

    let account_metas = [
        AccountMeta::new(token_program_a.key(), false, false),      // tokenProgramA
        AccountMeta::new(token_program_b.key(), false, false),      // tokenProgramB
        AccountMeta::new(header_accounts[5].key(), false, false),   // memoProgram
        AccountMeta::new(header_accounts[0].key(), true, true),     // tokenAuthority (payer)
        AccountMeta::new(whirlpool_accounts[1].key(), true, false), // whirlpool
        AccountMeta::new(mint_a.key(), false, false),               // tokenMintA
        AccountMeta::new(mint_b.key(), false, false),               // tokenMintB
        AccountMeta::new(token_account_a.key(), true, false),       // tokenOwnerAccountA
        AccountMeta::new(whirlpool_accounts[3].key(), true, false), // tokenVaultA
        AccountMeta::new(token_account_b.key(), true, false),       // tokenOwnerAccountB
        AccountMeta::new(whirlpool_accounts[4].key(), true, false), // tokenVaultB
        AccountMeta::new(whirlpool_accounts[5].key(), true, false), // tickArray0
        AccountMeta::new(whirlpool_accounts[6].key(), true, false), // tickArray1
        AccountMeta::new(whirlpool_accounts[7].key(), true, false), // tickArray2
        AccountMeta::new(whirlpool_accounts[2].key(), true, false), // oracle
    ];

    let mut instruction_data = WHIRLPOOL_INSTRUCTION_DATA;
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());
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
