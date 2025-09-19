use pinocchio::{
    account_info::AccountInfo, 
    instruction::AccountMeta, 
    instruction::Instruction, 
    cpi::invoke,
    ProgramResult,
};


pub fn execute_dlmm_swap(
    trade_amount: u64,
    header_accounts: &[AccountInfo],
    dlmm_accounts: &[AccountInfo],
    is_buy: bool,
    is_wsol_x_mint: bool,
) -> ProgramResult {
 
    let (token_x_mint, token_y_mint, token_x_program, token_y_program) = if is_wsol_x_mint {
        (&header_accounts[1], &header_accounts[6], &header_accounts[3], &header_accounts[7])
    } else {
        (&header_accounts[6], &header_accounts[1], &header_accounts[7], &header_accounts[3])
    };


    let (
        user_token_in,
        user_token_out,
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

    let dlmm_program_id = &dlmm_accounts[0];
    // 🚀 优化3: 栈分配AccountMeta数组，避免Vec的堆分配
    let account_metas = [
        AccountMeta::new(dlmm_accounts[3].key(), true, false),   // pool_state(writable)
        AccountMeta::new(dlmm_program_id.key(), false, false),   // bin_array_bitmap_extension(readonly)

        AccountMeta::new(dlmm_accounts[4].key(), true, false),   // reserve_x(writable)
        AccountMeta::new(dlmm_accounts[5].key(), true, false),   // reserve_y(writable)
        AccountMeta::new(user_token_in.key(), true, false),   // user_token_in(writable)
        AccountMeta::new(user_token_out.key(), true, false),   // user_token_out(writable)
        AccountMeta::new(token_x_mint.key(), false, false),   // token_x_mint(readonly)
        AccountMeta::new(token_y_mint.key(), false, false),   // token_y_mint(readonly)
        AccountMeta::new(dlmm_accounts[2].key(), true, false),   // oracle(writable)
        AccountMeta::new(dlmm_program_id.key(), false, false),   // host_fee_in(readonly)
        AccountMeta::new(header_accounts[0].key(), true, true),   // payer (signer)
        AccountMeta::new(token_x_program.key(), false, false),   // token_x_program(readonly)
        AccountMeta::new(token_y_program.key(), false, false),   // token_y_program(readonly)
        AccountMeta::new(header_accounts[5].key(), false, false),   // memo_program(readonly)
        AccountMeta::new(dlmm_accounts[1].key(), false, false),   // event_authority(readonly)
        AccountMeta::new(dlmm_program_id.key(), false, false),   // program id(readonly)

        AccountMeta::new(dlmm_accounts[6].key(), true, false),   // bin_array_minus_1(writable)
        AccountMeta::new(dlmm_accounts[7].key(), true, false),   // bin_array_0(writable)
        AccountMeta::new(dlmm_accounts[8].key(), true, false),   // bin_array_1(writable)
    ];

    // 🚀 优化4: 栈分配指令数据，预分配精确容量
    let mut instruction_data = [0u8; 28];
    
    // 复制discriminator
    instruction_data[0..8].copy_from_slice(&[65, 75, 63, 76, 235, 91, 91, 136]);
    // 复制amount_in
    instruction_data[8..16].copy_from_slice(&trade_amount.to_le_bytes());
    // minimum_amount_out = 0
    instruction_data[16..24].copy_from_slice(&0u64.to_le_bytes());
    // remaining_accounts_info: RemainingAccountsInfo { slices: Vec<RemainingAccountsSlice> }
    // 空的 slices 向量序列化为: [长度(4字节) = 0]
    instruction_data[24..28].copy_from_slice(&0u32.to_le_bytes()); // slices.len() = 0


    // 🚀 优化5: 构建Pinocchio指令结构
    let swap_instruction = Instruction {
        program_id: dlmm_accounts[0].key(),
        accounts: &account_metas,
        data: &instruction_data,
    };

    // 🚀 优化6: 栈分配AccountInfo数组，避免Vec分配
    let account_infos = [
        &dlmm_accounts[3],     // 
        dlmm_program_id,       // bin_array_bitmap_extension
        &dlmm_accounts[4],       // reserve_x
        &dlmm_accounts[5],       // reserve_y
        user_token_in,     // user_token_in
        user_token_out,    // output_token_account
        token_x_mint,             // token_x_mint
        token_y_mint,            // token_y_mint
        &dlmm_accounts[2],     // oracle
        dlmm_program_id,       // host_fee_in
        &header_accounts[0],       // payer
        token_x_program,        // token_x_program
        token_y_program,       // token_y_program
        &header_accounts[5],       // memo_program
        &dlmm_accounts[1],       // event_authority
        dlmm_program_id,       // program
        &dlmm_accounts[6],       // bin_array_minus_1
        &dlmm_accounts[7],       // bin_array_0
        &dlmm_accounts[8],       // bin_array_1
    ];

 
    // 使用编译时常量指定账户数量，最大化性能
    invoke::<19>(&swap_instruction, &account_infos).map_err(|e| e.into())
}