use pinocchio::{
    account_info::AccountInfo, 
    instruction::AccountMeta, 
    instruction::Instruction,
    cpi::invoke,
    ProgramResult,
};


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
    let mut instruction_data = [0u8; 24];
    
    // 复制discriminator
    instruction_data[0..8].copy_from_slice(&[143, 190, 90, 218, 196, 30, 51, 222]);
    // 复制amount_in
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());
    // minimum_amount_out = 0
    instruction_data[16..24].copy_from_slice(&0u64.to_le_bytes());

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


// / 执行CPMM交换 - 终极性能版本 (Unsafe)
// / 
// / ⚠️  WARNING: 此函数跳过借用检查以获得最低CU消耗
// / 只有在完全确定借用规则得到遵守时才使用
// / 
// / 性能优势:
// / 1. 跳过RefCell借用检查 (~50-100 CU节省)
// / 2. 零分配，全栈操作
// / 3. 直接syscall调用
// / 4. 编译时账户数量优化
// / 5. 栈分配指令数据

// pub unsafe fn execute_cpmm_swap_unchecked(
//     trade_amount: u64,
//     header_accounts: &[AccountInfo],
//     cpmm_accounts: &[AccountInfo],
//     is_buy: bool,
//     is_wsol_pool_0: bool,
// ) -> ProgramResult {
//     use pinocchio::{
//         cpi::invoke_signed_unchecked,
//         instruction::Account,
//     };

//     // 🚀 优化1: 确定vault配置，避免运行时分支
//     let (wsol_vault, token_vault) = if is_wsol_pool_0 {
//         (&cpmm_accounts[5], &cpmm_accounts[6])
//     } else {
//         (&cpmm_accounts[6], &cpmm_accounts[5])
//     };

//     // 🚀 优化2: 根据交易方向选择账户，编译时优化
//     let (
//         input_token_account,
//         output_token_account,
//         input_vault,
//         output_vault,
//         input_token_program,
//         output_token_program,
//         input_token_mint,
//         output_token_mint,
//     ) = if is_buy {
//         (
//             &header_accounts[2],  // wsol_token_account
//             &header_accounts[8],  // mint_token_account
//             wsol_vault,
//             token_vault,
//             &header_accounts[3],  // token_program
//             &header_accounts[7],  // token_program_for_mint
//             &header_accounts[1],  // wsol_mint
//             &header_accounts[6],  // token_mint
//         )
//     } else {
//         (
//             &header_accounts[8],  // mint_token_account
//             &header_accounts[2],  // wsol_token_account
//             token_vault,
//             wsol_vault,
//             &header_accounts[7],  // token_program_for_mint
//             &header_accounts[3],  // token_program
//             &header_accounts[6],  // token_mint
//             &header_accounts[1],  // wsol_mint
//         )
//     };

//     // 🚀 优化3: 栈分配AccountMeta数组
//     let account_metas = [
//         AccountMeta::new(header_accounts[0].key(), true, true),   // payer (signer)
//         AccountMeta::new(cpmm_accounts[1].key(), false, false),   // authority (readonly)
//         AccountMeta::new(cpmm_accounts[2].key(), false, false),   // amm_config (readonly)
//         AccountMeta::new(cpmm_accounts[4].key(), true, false),    // pool_state (writable)
//         AccountMeta::new(input_token_account.key(), true, false), // input_token_account (writable)
//         AccountMeta::new(output_token_account.key(), true, false), // output_token_account (writable)
//         AccountMeta::new(input_vault.key(), true, false),         // input_vault (writable)
//         AccountMeta::new(output_vault.key(), true, false),        // output_vault (writable)
//         AccountMeta::new(input_token_program.key(), false, false), // input_token_program (readonly)
//         AccountMeta::new(output_token_program.key(), false, false), // output_token_program (readonly)
//         AccountMeta::new(input_token_mint.key(), false, false),   // input_token_mint (readonly)
//         AccountMeta::new(output_token_mint.key(), false, false),  // output_token_mint (readonly)
//         AccountMeta::new(cpmm_accounts[3].key(), true, false),    // observation_state (writable)
//     ];

//     // 🚀 优化4: 栈分配指令数据
//     const INSTRUCTION_DATA_SIZE: usize = 24;
//     let mut instruction_data = [0u8; INSTRUCTION_DATA_SIZE];
    
//     instruction_data[0..8].copy_from_slice(&[143, 190, 90, 218, 196, 30, 51, 222]);
//     instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());
//     instruction_data[16..24].copy_from_slice(&0u64.to_le_bytes());

//     // 🚀 优化5: 构建Pinocchio指令结构
//     let swap_instruction = Instruction {
//         program_id: cpmm_accounts[0].key(),
//         accounts: &account_metas,
//         data: &instruction_data,
//     };

//     // 🚀 优化6: 栈分配Account数组，避免AccountInfo->Account的转换开销
//     let accounts = [
//         Account::from(&header_accounts[0]),     // payer
//         Account::from(&cpmm_accounts[1]),       // authority
//         Account::from(&cpmm_accounts[2]),       // amm_config
//         Account::from(&cpmm_accounts[4]),       // pool_state
//         Account::from(input_token_account),     // input_token_account
//         Account::from(output_token_account),    // output_token_account
//         Account::from(input_vault),             // input_vault
//         Account::from(output_vault),            // output_vault
//         Account::from(input_token_program),     // input_token_program
//         Account::from(output_token_program),    // output_token_program
//         Account::from(input_token_mint),        // input_token_mint
//         Account::from(output_token_mint),       // output_token_mint
//         Account::from(&cpmm_accounts[3]),       // observation_state
//     ];

//     // 🚀 优化7: 使用unsafe CPI调用，跳过所有借用检查
//     // 这是最高效的CPI调用方式，但需要确保借用安全
//     invoke_signed_unchecked(&swap_instruction, &accounts, &[]);
    
//     Ok(())
// }
