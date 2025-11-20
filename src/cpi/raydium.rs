use pinocchio::{
    account_info::AccountInfo, 
    instruction::AccountMeta, 
    instruction::Instruction,
    cpi::invoke,
    ProgramResult,
};
use crate::error::PinocchioCpiError;

const RAYDIUM_INSTRUCTION_DATA: [u8; 17] = [
    // swap discriminator [0..1]
    9,
    // amount_in placeholder [1..9] - 将被替换
    0, 0, 0, 0, 0, 0, 0, 0,
    // minimum_amount_out = 0 [9..17]
    0, 0, 0, 0, 0, 0, 0, 0,
];

pub fn execute_raydium_swap(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    raydium_accounts: &[AccountInfo],
    is_buy: bool,
    _is_wsol_pool_0: bool,
) -> ProgramResult {
  
    let (
        input_token_account,
        output_token_account,
    ) = if is_buy {
        (
            &header_accounts[2],  // wsol_token_account
            &header_accounts[8],  // mint_token_account
        
        )
    } else {
        (
            &header_accounts[8],  // mint_token_account
            &header_accounts[2],  // wsol_token_account
        )
    };


    // 🚀 优化3: 预缓存频繁访问的账户，减少重复访问
    let user_owner = &header_accounts[0];
    let token_program = &header_accounts[7];
    let amm_account = &raydium_accounts[2];
    let authority = &raydium_accounts[1];
    let pool_coin = &raydium_accounts[3];
    let pool_pc = &raydium_accounts[4];

    // 🚀 优化4: 栈分配AccountMeta数组，避免Vec的堆分配
    let account_metas = [
        AccountMeta::readonly(token_program.key()),        // mint_token_program
        AccountMeta::writable(amm_account.key()),          // Amm Id
        AccountMeta::readonly(authority.key()),            // authority
        AccountMeta::writable(amm_account.key()),          // Amm Open Orders
        AccountMeta::writable(pool_coin.key()),            // Pool Coin Token Account
        AccountMeta::writable(pool_pc.key()),              // Pool Pc Token Account
        AccountMeta::writable(amm_account.key()),          // Serum Program Id
        AccountMeta::writable(amm_account.key()),          // Serum Market
        AccountMeta::writable(amm_account.key()),          // Serum Bids
        AccountMeta::writable(amm_account.key()),          // Serum Asks
        AccountMeta::writable(amm_account.key()),          // Serum Event Queue
        AccountMeta::writable(amm_account.key()),          // Serum Coin Vault Account
        AccountMeta::writable(amm_account.key()),          // Serum Pc Vault Account
        AccountMeta::writable(amm_account.key()),          // Serum Vault Signer
        AccountMeta::writable(input_token_account.key()),  // User Source Token Account
        AccountMeta::writable(output_token_account.key()), // User Dest Token Account
        AccountMeta::writable_signer(user_owner.key()),    // User Owner (signer)
    ];

    // 🚀 优化：预构建模板，只替换变量部分
    let mut instruction_data = RAYDIUM_INSTRUCTION_DATA;
      
    instruction_data[1..9].copy_from_slice(&trade_amount.to_le_bytes());

    // 🚀 优化5: 构建Pinocchio指令结构
    let swap_instruction = Instruction {
        program_id: raydium_accounts[0].key(),
        accounts: &account_metas,
        data: &instruction_data,
    };

    // 🚀 优化6: 栈分配AccountInfo数组，避免Vec分配
    let account_infos = [
        &header_accounts[7],   // mint_token_program
        amm_account,   // Amm Id
        &raydium_accounts[1],   // authority
        amm_account,   // Amm Open Orders
        &raydium_accounts[3],   // Pool Coin Token Account
        &raydium_accounts[4],   // Pool Pc Token Account
        amm_account,   // Serum Program Id
        amm_account,   // Serum Market
        amm_account,   // Serum Bids
        amm_account,   // Serum Asks
        amm_account,   // Serum Event Queue
        amm_account,   // Serum Coin Vault Account
        amm_account,   // Serum Pc Vault Account
        amm_account,   // Serum Vault Signer
        &input_token_account,   // User Source Token Account
        &output_token_account,   // User Dest Token Account
        &header_accounts[0],   // User Owner
    ];

    // 🚀 优化7: 使用Pinocchio高效CPI调用
    // 使用编译时常量指定账户数量，最大化性能
    invoke::<17>(&swap_instruction, &account_infos)
}

pub fn execute_raydium_swap_hop3(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    raydium_accounts: &[AccountInfo],
    step: u8,
    is_wsol_pool_0: bool,
) -> ProgramResult {
    match step {
        1 => {
            execute_raydium_swap(trade_amount, header_accounts, raydium_accounts, true, is_wsol_pool_0)
        }
        2 => {
            execute_raydium_swap_mid(trade_amount, header_accounts, raydium_accounts, is_wsol_pool_0)
        }
        3 => {
            execute_raydium_swap_sell(trade_amount, header_accounts, raydium_accounts, is_wsol_pool_0)
        }
        _ => {
            Err(PinocchioCpiError::UnsupportedPoolType.into())
        }
    }
}

fn execute_raydium_swap_mid(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    raydium_accounts: &[AccountInfo],
    _is_mid_zero_to_one: bool,
) -> ProgramResult {
    // 中间交换：Token1 -> Token2
    // 输入：header_accounts[8] (token1_account)
    // 输出：header_accounts[11] (token2_account)
    
    let user_owner = &header_accounts[0];
    let token_program = &header_accounts[7];
    let amm_account = &raydium_accounts[2];
    let authority = &raydium_accounts[1];
    let pool_coin = &raydium_accounts[3];
    let pool_pc = &raydium_accounts[4];

    let account_metas = [
        AccountMeta::readonly(token_program.key()),                    // mint_token_program
        AccountMeta::writable(amm_account.key()),                      // Amm Id
        AccountMeta::readonly(authority.key()),                        // authority
        AccountMeta::writable(amm_account.key()),                      // Amm Open Orders
        AccountMeta::writable(pool_coin.key()),                        // Pool Coin Token Account
        AccountMeta::writable(pool_pc.key()),                          // Pool Pc Token Account
        AccountMeta::writable(amm_account.key()),                      // Serum Program Id
        AccountMeta::writable(amm_account.key()),                      // Serum Market
        AccountMeta::writable(amm_account.key()),                      // Serum Bids
        AccountMeta::writable(amm_account.key()),                      // Serum Asks
        AccountMeta::writable(amm_account.key()),                      // Serum Event Queue
        AccountMeta::writable(amm_account.key()),                      // Serum Coin Vault Account
        AccountMeta::writable(amm_account.key()),                      // Serum Pc Vault Account
        AccountMeta::writable(amm_account.key()),                      // Serum Vault Signer
        AccountMeta::writable(header_accounts[8].key()),               // User Source Token Account (token1)
        AccountMeta::writable(header_accounts[11].key()),              // User Dest Token Account (token2)
        AccountMeta::writable_signer(user_owner.key()),                // User Owner (signer)
    ];

    let mut instruction_data = RAYDIUM_INSTRUCTION_DATA;
    instruction_data[1..9].copy_from_slice(&trade_amount.to_le_bytes());

    let swap_instruction = Instruction {
        program_id: raydium_accounts[0].key(),
        accounts: &account_metas,
        data: &instruction_data,
    };

    let account_infos = [
        &header_accounts[7],      // mint_token_program
        amm_account,              // Amm Id
        &raydium_accounts[1],     // authority
        amm_account,              // Amm Open Orders
        &raydium_accounts[3],     // Pool Coin Token Account
        &raydium_accounts[4],     // Pool Pc Token Account
        amm_account,              // Serum Program Id
        amm_account,              // Serum Market
        amm_account,              // Serum Bids
        amm_account,              // Serum Asks
        amm_account,              // Serum Event Queue
        amm_account,              // Serum Coin Vault Account
        amm_account,              // Serum Pc Vault Account
        amm_account,              // Serum Vault Signer
        &header_accounts[8],      // User Source Token Account (token1)
        &header_accounts[11],     // User Dest Token Account (token2)
        &header_accounts[0],      // User Owner
    ];

    invoke::<17>(&swap_instruction, &account_infos)
}

fn execute_raydium_swap_sell(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    raydium_accounts: &[AccountInfo],
    _is_wsol_pool_0: bool,
) -> ProgramResult {
    // 卖出交换：Token2 -> WSOL
    // 输入：header_accounts[11] (token2_account)
    // 输出：header_accounts[2] (wsol_account)
    
    let user_owner = &header_accounts[0];
    let token_program = &header_accounts[10];
    let amm_account = &raydium_accounts[2];
    let authority = &raydium_accounts[1];
    let pool_coin = &raydium_accounts[3];
    let pool_pc = &raydium_accounts[4];

    let account_metas = [
        AccountMeta::readonly(token_program.key()),                    // mint_token_program
        AccountMeta::writable(amm_account.key()),                      // Amm Id
        AccountMeta::readonly(authority.key()),                        // authority
        AccountMeta::writable(amm_account.key()),                      // Amm Open Orders
        AccountMeta::writable(pool_coin.key()),                        // Pool Coin Token Account
        AccountMeta::writable(pool_pc.key()),                          // Pool Pc Token Account
        AccountMeta::writable(amm_account.key()),                      // Serum Program Id
        AccountMeta::writable(amm_account.key()),                      // Serum Market
        AccountMeta::writable(amm_account.key()),                      // Serum Bids
        AccountMeta::writable(amm_account.key()),                      // Serum Asks
        AccountMeta::writable(amm_account.key()),                      // Serum Event Queue
        AccountMeta::writable(amm_account.key()),                      // Serum Coin Vault Account
        AccountMeta::writable(amm_account.key()),                      // Serum Pc Vault Account
        AccountMeta::writable(amm_account.key()),                      // Serum Vault Signer
        AccountMeta::writable(header_accounts[11].key()),              // User Source Token Account (token2)
        AccountMeta::writable(header_accounts[2].key()),               // User Dest Token Account (wsol)
        AccountMeta::writable_signer(user_owner.key()),                // User Owner (signer)
    ];

    let mut instruction_data = RAYDIUM_INSTRUCTION_DATA;
    instruction_data[1..9].copy_from_slice(&trade_amount.to_le_bytes());

    let swap_instruction = Instruction {
        program_id: raydium_accounts[0].key(),
        accounts: &account_metas,
        data: &instruction_data,
    };

    let account_infos = [
        &header_accounts[7],      // mint_token_program
        amm_account,              // Amm Id
        &raydium_accounts[1],     // authority
        amm_account,              // Amm Open Orders
        &raydium_accounts[3],     // Pool Coin Token Account
        &raydium_accounts[4],     // Pool Pc Token Account
        amm_account,              // Serum Program Id
        amm_account,              // Serum Market
        amm_account,              // Serum Bids
        amm_account,              // Serum Asks
        amm_account,              // Serum Event Queue
        amm_account,              // Serum Coin Vault Account
        amm_account,              // Serum Pc Vault Account
        amm_account,              // Serum Vault Signer
        &header_accounts[11],     // User Source Token Account (token2)
        &header_accounts[2],      // User Dest Token Account (wsol)
        &header_accounts[0],      // User Owner
    ];

    invoke::<17>(&swap_instruction, &account_infos)
}
