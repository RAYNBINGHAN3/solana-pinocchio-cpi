use pinocchio::{
    account_info::AccountInfo, 
    instruction::AccountMeta, 
    instruction::Instruction,
    cpi::invoke,
    ProgramResult,
};


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

 
    let mut instruction_data = [0u8; 17];
    
    // 复制discriminator
    instruction_data[0..1].copy_from_slice(&[9]);
    // 复制amount_in
    instruction_data[1..9].copy_from_slice(&trade_amount.to_le_bytes());
    // minimum_amount_out = 0
    instruction_data[9..17].copy_from_slice(&0u64.to_le_bytes());

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
