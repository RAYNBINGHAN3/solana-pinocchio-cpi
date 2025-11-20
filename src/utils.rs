use crate::error::PinocchioResult;
use pinocchio::account_info::AccountInfo;

/// 每种池类型需要的账户数量
pub const CPMM_ACCOUNT_COUNT: usize = 7;
pub const DLMM_ACCOUNT_COUNT: usize = 9;
pub const DAMMV2_ACCOUNT_COUNT: usize = 6;
pub const PUMP_ACCOUNT_COUNT: usize = 16;
pub const RAYDIUM_ACCOUNT_COUNT: usize = 5;
pub const CLMM_ACCOUNT_COUNT: usize = 10;
pub const WHIRLPOOL_ACCOUNT_COUNT: usize = 8;

// 使用编译时常量数组，零运行时开销
const POOL_COUNTS: [usize; 7] = [
    CPMM_ACCOUNT_COUNT,      // 0
    DLMM_ACCOUNT_COUNT,      // 1
    DAMMV2_ACCOUNT_COUNT,    // 2
    PUMP_ACCOUNT_COUNT,      // 3
    RAYDIUM_ACCOUNT_COUNT,   // 4
    CLMM_ACCOUNT_COUNT,      // 5
    WHIRLPOOL_ACCOUNT_COUNT, // 6
];

/// 🚀 优化的指令数据解析结构
#[derive(Debug)]
pub struct SwapParams {
    pub buy: u8,
    pub mid: Option<u8>,
    pub sell: u8,
    pub is_wsol_pool_0_buy: bool,
    pub is_mid_zero_to_one: Option<bool>,
    pub is_wsol_pool_0_sell: bool,
    pub is_simulate: bool,
    pub amount_in: u64,
    pub pump_base_amount_out: u64,
    pub min_profit: u32,
}

/// 🚀 超高效获取池账户数量 - 直接索引访问，零边界检查
#[inline(always)]
pub fn get_pool_info_by_num(buy: u8) -> usize {
    // 🚀 优化：假设输入总是有效的，直接unsafe访问
    unsafe { *POOL_COUNTS.get_unchecked(buy as usize) }
}

/// 🚀 高效解析指令数据
#[inline(always)]
pub fn parse_instruction_data(data: &[u8], is_3hop: bool) -> PinocchioResult<SwapParams> {
 
    let params = if !is_3hop {
        SwapParams {
            buy: data[0],
            mid: None,
            sell: data[1],
            is_wsol_pool_0_buy: data[2] == 1,
            is_mid_zero_to_one: None,
            is_wsol_pool_0_sell: data[3] == 1,
            is_simulate: data[4] == 1,
            amount_in: u64::from_le_bytes(data[5..13].try_into().unwrap()),
            pump_base_amount_out: u64::from_le_bytes(data[13..21].try_into().unwrap()),
            min_profit: u32::from_le_bytes(data[21..25].try_into().unwrap()),
        }
    } else {
        SwapParams {
            buy: data[0],
            mid: Some(data[1]),
            sell: data[2],
            is_wsol_pool_0_buy: data[3] == 1,
            is_mid_zero_to_one: Some(data[4] == 1),
            is_wsol_pool_0_sell: data[5] == 1,
            is_simulate: data[6] == 1,
            amount_in: u64::from_le_bytes(data[7..15].try_into().unwrap()),
            pump_base_amount_out: u64::from_le_bytes(data[15..23].try_into().unwrap()),
            min_profit: u32::from_le_bytes(data[23..27].try_into().unwrap()),
        }
    };

    Ok(params)
}

/// 🚀 验证池类型并返回账户数量 - 一次调用获取两个值
#[inline(always)]
pub fn validate_pool_types(buy: u8) -> PinocchioResult<usize> {
    let buy_count = get_pool_info_by_num(buy);

    // if buy_count == 111 || sell_count == 111 {
    //     return Err(PinocchioCpiError::UnsupportedPoolType);
    // }

    Ok(buy_count)
}

/// 🚀 超高效获取 Token 余额 - 直接读取账户数据
#[inline(always)]
pub fn get_token_balance(token_account: &AccountInfo) -> PinocchioResult<u64> {
    // 🚀 优化：直接unsafe读取，避免slice操作和错误检查
    unsafe {
        let data_ptr = token_account.data_ptr().add(64);
        Ok(core::ptr::read_unaligned(data_ptr as *const u64))
    }
}
