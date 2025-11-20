use pinocchio::{
    account_info::AccountInfo, 
    instruction::AccountMeta, 
    instruction::Instruction,
    cpi::invoke,
    ProgramResult,
};
use crate::error::PinocchioCpiError;

const CPMM_INSTRUCTION_DATA: [u8; 24] = [
    // discriminator [0..8]
    143, 190, 90, 218, 196, 30, 51, 222,
    // amount_in placeholder [8..16] - 将被替换
    0, 0, 0, 0, 0, 0, 0, 0,
    // minimum_amount_out = 0 [16..24]
    0, 0, 0, 0, 0, 0, 0, 0,
];

// const DISCRIMINATOR: [u8; 8] = [143, 190, 90, 218, 196, 30, 51, 222];
pub fn execute_cpmm_swap(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    cpmm_accounts: &[AccountInfo],
    is_buy: bool,
    is_wsol_pool_0: bool,
) -> ProgramResult {
  
    // 🚀 优化1: 确定vault配置，避免运行时分支
    let (wsol_vault, token_vault) = if is_wsol_pool_0 {
        (&cpmm_accounts[5], &cpmm_accounts[6])
    } else {
        (&cpmm_accounts[6], &cpmm_accounts[5])
    };

    // 🚀 优化2: 根据交易方向选择账户，编译时优化
    let (
        input_token_account,
        output_token_account,
        input_vault,
        output_vault,
        input_token_program,
        output_token_program,
        input_token_mint,
        output_token_mint,
    ) = if is_buy {
        (
            &header_accounts[2],  // wsol_token_account
            &header_accounts[8],  // mint_token_account
            wsol_vault,
            token_vault,
            &header_accounts[3],  // token_program
            &header_accounts[7],  // token_program_for_mint
            &header_accounts[1],  // wsol_mint
            &header_accounts[6],  // token_mint
        )
    } else {
        (
            &header_accounts[8],  // mint_token_account
            &header_accounts[2],  // wsol_token_account
            token_vault,
            wsol_vault,
            &header_accounts[7],  // token_program_for_mint
            &header_accounts[3],  // token_program
            &header_accounts[6],  // token_mint
            &header_accounts[1],  // wsol_mint
        )
    };

    // 🚀 优化3: 栈分配AccountMeta数组，避免Vec的堆分配
    let account_metas = [
        AccountMeta::new(header_accounts[0].key(), true, true),   // payer (signer)
        AccountMeta::new(cpmm_accounts[1].key(), false, false),   // authority (readonly)
        AccountMeta::new(cpmm_accounts[2].key(), false, false),   // amm_config (readonly)
        AccountMeta::new(cpmm_accounts[4].key(), true, false),    // pool_state (writable)
        AccountMeta::new(input_token_account.key(), true, false), // input_token_account (writable)
        AccountMeta::new(output_token_account.key(), true, false), // output_token_account (writable)
        AccountMeta::new(input_vault.key(), true, false),         // input_vault (writable)
        AccountMeta::new(output_vault.key(), true, false),        // output_vault (writable)
        AccountMeta::new(input_token_program.key(), false, false), // input_token_program (readonly)
        AccountMeta::new(output_token_program.key(), false, false), // output_token_program (readonly)
        AccountMeta::new(input_token_mint.key(), false, false),   // input_token_mint (readonly)
        AccountMeta::new(output_token_mint.key(), false, false),  // output_token_mint (readonly)
        AccountMeta::new(cpmm_accounts[3].key(), true, false),    // observation_state (writable)
    ];

    // 🚀 优化4: 栈分配指令数据，预分配精确容量
    // const INSTRUCTION_DATA_SIZE: usize = 24; // 8字节discriminator + 8字节amount_in + 8字节minimum_amount_out
    
    // 🚀 优化：预构建模板，只替换变量部分
    let mut instruction_data = CPMM_INSTRUCTION_DATA;
   
    // 只替换变量部分
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());

    // 🚀 优化5: 构建Pinocchio指令结构
    let swap_instruction = Instruction {
        program_id: cpmm_accounts[0].key(),
        accounts: &account_metas,
        data: &instruction_data,
    };

    // 🚀 优化6: 栈分配AccountInfo数组，避免Vec分配
    let account_infos = [
        &header_accounts[0],     // payer
        &cpmm_accounts[1],       // authority
        &cpmm_accounts[2],       // amm_config
        &cpmm_accounts[4],       // pool_state
        input_token_account,     // input_token_account
        output_token_account,    // output_token_account
        input_vault,             // input_vault
        output_vault,            // output_vault
        input_token_program,     // input_token_program
        output_token_program,    // output_token_program
        input_token_mint,        // input_token_mint
        output_token_mint,       // output_token_mint
        &cpmm_accounts[3],       // observation_state
    ];

    // 🚀 优化7: 使用Pinocchio高效CPI调用
    // 使用编译时常量指定账户数量，最大化性能
    invoke::<13>(&swap_instruction, &account_infos)
}



pub fn execute_cpmm_swap_hop3(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    cpmm_accounts: &[AccountInfo],
    step: u8,
    is_wsol_pool_0: bool,
) -> ProgramResult {
    match step {
        1 => {
            return execute_cpmm_swap(trade_amount, header_accounts, cpmm_accounts, true, is_wsol_pool_0)
        }
        2 => {
            return execute_cpmm_swap_mid(trade_amount, header_accounts, cpmm_accounts, is_wsol_pool_0)
        }
        3 => {
            return execute_cpmm_swap_sell(trade_amount, header_accounts, cpmm_accounts, is_wsol_pool_0)
        }
        _ => {
            return Err(PinocchioCpiError::UnsupportedPoolType.into());
        }
    }
    
}


fn execute_cpmm_swap_mid(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    cpmm_accounts: &[AccountInfo],
    is_mid_zero_to_one: bool,
) -> ProgramResult {

    // 中间交换：Token1 -> Token2
    // 输入：header_accounts[8] (token1_account)
    // 输出：header_accounts[11] (token2_account)
    
    let (input_vault, output_vault) = if is_mid_zero_to_one {
        (&cpmm_accounts[5], &cpmm_accounts[6])
    } else {
        (&cpmm_accounts[6], &cpmm_accounts[5])
    };

    let account_metas = [
        AccountMeta::new(header_accounts[0].key(), true, true),   // payer
        AccountMeta::new(cpmm_accounts[1].key(), false, false),   // authority
        AccountMeta::new(cpmm_accounts[2].key(), false, false),   // amm_config
        AccountMeta::new(cpmm_accounts[4].key(), true, false),    // pool_state
        AccountMeta::new(header_accounts[8].key(), true, false),  // input_token_account (token1)
        AccountMeta::new(header_accounts[11].key(), true, false), // output_token_account (token2)
        AccountMeta::new(input_vault.key(), true, false),         // input_vault
        AccountMeta::new(output_vault.key(), true, false),        // output_vault
        AccountMeta::new(header_accounts[7].key(), false, false), // token1_program
        AccountMeta::new(header_accounts[10].key(), false, false), // token2_program
        AccountMeta::new(header_accounts[6].key(), false, false), // token1_mint
        AccountMeta::new(header_accounts[9].key(), false, false), // token2_mint
        AccountMeta::new(cpmm_accounts[3].key(), true, false),    // observation_state
    ];

    let mut instruction_data = CPMM_INSTRUCTION_DATA;
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());

    let swap_instruction = Instruction {
        program_id: cpmm_accounts[0].key(),
        accounts: &account_metas,
        data: &instruction_data,
    };

    let account_infos = [
        &header_accounts[0], 
        &cpmm_accounts[1], 
        &cpmm_accounts[2], 
        &cpmm_accounts[4],
        &header_accounts[8], 
        &header_accounts[11], 
        input_vault, 
        output_vault,
        &header_accounts[7],
        &header_accounts[10], 
        &header_accounts[6], 
        &header_accounts[9],
        &cpmm_accounts[3],
    ];

    invoke::<13>(&swap_instruction, &account_infos)
}


fn execute_cpmm_swap_sell(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    cpmm_accounts: &[AccountInfo],
    is_base_mint_on_0: bool,
) -> ProgramResult {
    // 卖出交换：Token2 -> WSOL
    // 输入：header_accounts[11] (token2_account)
    // 输出：header_accounts[2] (wsol_account)
    
    let (input_vault, output_vault) = if is_base_mint_on_0 {
        (&cpmm_accounts[6], &cpmm_accounts[5])  
    } else {
        (&cpmm_accounts[5], &cpmm_accounts[6])  
    };

    let account_metas = [
        AccountMeta::new(header_accounts[0].key(), true, true),   // payer
        AccountMeta::new(cpmm_accounts[1].key(), false, false),   // authority
        AccountMeta::new(cpmm_accounts[2].key(), false, false),   // amm_config
        AccountMeta::new(cpmm_accounts[4].key(), true, false),    // pool_state
        AccountMeta::new(header_accounts[11].key(), true, false), // input_token_account (token2)
        AccountMeta::new(header_accounts[2].key(), true, false),  // output_token_account (wsol)
        AccountMeta::new(input_vault.key(), true, false),         // input_vault
        AccountMeta::new(output_vault.key(), true, false),        // output_vault
        AccountMeta::new(header_accounts[10].key(), false, false), // token2_program
        AccountMeta::new(header_accounts[3].key(), false, false), // wsol_program
        AccountMeta::new(header_accounts[9].key(), false, false), // token2_mint
        AccountMeta::new(header_accounts[1].key(), false, false), // wsol_mint
        AccountMeta::new(cpmm_accounts[3].key(), true, false),    // observation_state
    ];

    let mut instruction_data = CPMM_INSTRUCTION_DATA;
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());

    let swap_instruction = Instruction {
        program_id: cpmm_accounts[0].key(),
        accounts: &account_metas,
        data: &instruction_data,
    };

    let account_infos = [
        &header_accounts[0], 
        &cpmm_accounts[1], 
        &cpmm_accounts[2], 
        &cpmm_accounts[4],
        &header_accounts[11], 
        &header_accounts[2], 
        input_vault, 
        output_vault,
        &header_accounts[10], 
        &header_accounts[3], 
        &header_accounts[9], 
        &header_accounts[1],
        &cpmm_accounts[3],
    ];

    invoke::<13>(&swap_instruction, &account_infos)
}