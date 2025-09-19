use crate::error::{PinocchioCpiError, PinocchioResult};
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
    pub sell: u8,
    pub is_wsol_pool_0_buy: bool,
    pub is_wsol_pool_0_sell: bool,
    pub is_simulate: bool,
    pub amount_in: u64,
    pub pump_base_amount_out: u64,
}

/// 🚀 超高效获取池账户数量 - 使用查找表避免重复匹配
#[inline(always)]
pub fn get_pool_info_by_num(buy: u8) -> usize {
    // 使用 get() 进行边界检查，避免 panic
    let buy_count = POOL_COUNTS.get(buy as usize).copied().unwrap_or(111);
    // let sell_count = POOL_COUNTS.get(sell as usize).copied().unwrap_or(111);

    buy_count
}

/// 🚀 高效解析指令数据
#[inline(always)]
pub fn parse_instruction_data(data: &[u8]) -> PinocchioResult<SwapParams> {
    // if data.len() < 12 {
    //     return Err(PinocchioCpiError::InstructionDataTooShort);
    // }

    let params = SwapParams {
        buy: data[0],
        sell: data[1],
        is_wsol_pool_0_buy: data[2] == 1,
        is_wsol_pool_0_sell: data[3] == 1,
        is_simulate: data[4] == 1,
        amount_in: u64::from_le_bytes(data[4..12].try_into().unwrap()),
        pump_base_amount_out: u64::from_le_bytes(data[12..20].try_into().unwrap()),
    };

    // if params.amount_in == 0 {
    //     return Err(PinocchioCpiError::InvalidTradeAmount);
    // }

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
    // Token 账户数据布局：amount 在偏移 64 字节处
    let data_len = token_account.data_len();
    // if data_len < 72 {
    //     return Err(PinocchioCpiError::InvalidTokenAccountData);
    // }

    let data = unsafe { core::slice::from_raw_parts(token_account.data_ptr(), data_len) };

    // 直接读取 8 字节的余额数据
    let balance_bytes: [u8; 8] = data[64..72]
        .try_into()
        .map_err(|_| PinocchioCpiError::InvalidTokenAccountData)?;

    Ok(u64::from_le_bytes(balance_bytes))
}
