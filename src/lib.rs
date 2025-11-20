use pinocchio::cpi::set_return_data;
use pinocchio::{account_info::AccountInfo, entrypoint, pubkey::Pubkey, ProgramResult};
pub mod cpi;
pub mod error;
pub mod utils;

use error::PinocchioCpiError;

// 使用标准入口点
entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Some((first, tail)) = instruction_data.split_first() {
        match first {
            4 => {
                execute_direct_cpi(accounts, tail)?;
            }
            5 => {
                execute_direct_cpi_3hop(accounts, tail)?;
            }
            _ => {
                return Err(PinocchioCpiError::UnsupportedPoolType.into());
            }
        }
        
    }

    Ok(())
}

fn execute_direct_cpi(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let params = utils::parse_instruction_data(instruction_data, false)?;

    let buy_count = utils::validate_pool_types(params.buy)?;

    // 🚀 优化：使用更高效的账户分割
    let (header_accounts, pool_accounts) = accounts.split_at(9); // 改为9个header账户
    let (buy_accounts, sell_accounts) = pool_accounts.split_at(buy_count);

    let initial_wsol_balance = utils::get_token_balance(&header_accounts[2])?;

    execute_swap_optimized(
        params.buy,
        params.amount_in,
        header_accounts,
        buy_accounts,
        true,
        params.is_wsol_pool_0_buy,
        params.pump_base_amount_out,
    )?;

    let token_balance = utils::get_token_balance(&header_accounts[8])?;

    execute_swap_optimized(
        params.sell,
        token_balance,
        header_accounts,
        sell_accounts,
        false,
        params.is_wsol_pool_0_sell,
        params.pump_base_amount_out,
    )?;

    let final_wsol_balance = utils::get_token_balance(&header_accounts[2])?;

    if final_wsol_balance <= initial_wsol_balance + params.min_profit as u64 {
        return Err(PinocchioCpiError::ArbitrageFailed.into());
    }

    if params.is_simulate {
        let profit = final_wsol_balance - initial_wsol_balance;

        let mut return_data = [0u8; 8];
        return_data[0..8].copy_from_slice(&profit.to_le_bytes());

        // 🚀 返回利润数据给客户端
        set_return_data(&return_data);
    }

    Ok(())
}


fn execute_direct_cpi_3hop(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let params = utils::parse_instruction_data(instruction_data, true)?;

    let buy_count = utils::validate_pool_types(params.buy)?;
    let mid_count = utils::validate_pool_types(params.mid.unwrap())?;

    // 🚀 优化：使用更高效的账户分割
    let (header_accounts, pool_accounts) = accounts.split_at(12); // 改为12个header账户(3hop+mid的basemint的mint+ tokenprogram +tokenacc信息账户)
    
    // 优雅地分割三个pool的账户
    let (buy_accounts, remaining) = pool_accounts.split_at(buy_count);
    let (mid_accounts, sell_accounts) = remaining.split_at(mid_count);

    let initial_wsol_balance = utils::get_token_balance(&header_accounts[2])?;

    //buy_pool
    execute_swap_optimized_3hop(
        params.buy,
        params.amount_in,
        header_accounts,
        buy_accounts,
        1,
        params.is_wsol_pool_0_buy,
        params.pump_base_amount_out,
    )?;

    let token1_balance = utils::get_token_balance(&header_accounts[8])?;
    
    //mid_pool
    execute_swap_optimized_3hop(
        params.mid.unwrap(),
        token1_balance,
        header_accounts,
        mid_accounts,
        2,
        params.is_mid_zero_to_one.unwrap(),
        params.pump_base_amount_out,
    )?;

    let token2_balance = utils::get_token_balance(&header_accounts[11])?;

    execute_swap_optimized_3hop(
        params.sell,
        token2_balance,
        header_accounts,
        sell_accounts,
        3,
        params.is_wsol_pool_0_sell,
        params.pump_base_amount_out,
    )?;

    let final_wsol_balance = utils::get_token_balance(&header_accounts[2])?;

    if final_wsol_balance <= initial_wsol_balance + params.min_profit as u64 {
        return Err(PinocchioCpiError::ArbitrageFailed.into());
    }

    if params.is_simulate {
        let profit = final_wsol_balance - initial_wsol_balance;

        let mut return_data = [0u8; 8];
        return_data[0..8].copy_from_slice(&profit.to_le_bytes());

        // 🚀 返回利润数据给客户端
        set_return_data(&return_data);
    }

    Ok(())
}




#[inline(always)]
fn execute_swap_optimized(
    pool_type: u8,
    amount_in: u64,
    header_accounts: &[AccountInfo],
    pool_accounts: &[AccountInfo],
    is_buy: bool,
    is_wsol_pool_0: bool,
    pump_base_amount_out: u64,
) -> ProgramResult {
    match pool_type {
        0 => cpi::cpmm::execute_cpmm_swap(
            amount_in,
            header_accounts,
            pool_accounts,
            is_buy,
            is_wsol_pool_0,
        ),
        1 => cpi::dlmm::execute_dlmm_swap(
            amount_in,
            header_accounts,
            pool_accounts,
            is_buy,
            is_wsol_pool_0,
        ),
        2 => cpi::dammv2::execute_dammv2_swap(
            amount_in,
            header_accounts,
            pool_accounts,
            is_buy,
            is_wsol_pool_0,
        ),
        3 => cpi::pump::execute_pump_swap(
            amount_in,
            header_accounts,
            pool_accounts,
            is_buy,
            is_wsol_pool_0,
            pump_base_amount_out,
        ),
        4 => cpi::raydium::execute_raydium_swap(
            amount_in,
            header_accounts,
            pool_accounts,
            is_buy,
            is_wsol_pool_0,
        ),
        5 => cpi::clmm::execute_clmm_swap(
            amount_in,
            header_accounts,
            pool_accounts,
            is_buy,
            is_wsol_pool_0,
        ),
        6 => cpi::whirlpool::execute_whirlpool_swap(
            amount_in,
            header_accounts,
            pool_accounts,
            is_buy,
            is_wsol_pool_0,
        ),
        _ => Err(PinocchioCpiError::UnsupportedPoolType.into()),
    }
}
 

 
#[inline(always)]
fn execute_swap_optimized_3hop(
    pool_type: u8,
    amount_in: u64,
    header_accounts: &[AccountInfo],
    pool_accounts: &[AccountInfo],
    step: u8,
    is_wsol_pool_0: bool,
    pump_base_amount_out: u64,
) -> ProgramResult {
    match pool_type {
        0 => cpi::cpmm::execute_cpmm_swap_hop3(
            amount_in,
            header_accounts,
            pool_accounts,
            step,
            is_wsol_pool_0
        ),
        1 => cpi::dlmm::execute_dlmm_swap_hop3(
            amount_in,
            header_accounts,
            pool_accounts,
            step,
            is_wsol_pool_0,
        ),
        2 => cpi::dammv2::execute_dammv2_swap_hop3(
            amount_in,
            header_accounts,
            pool_accounts,
            step,
            is_wsol_pool_0,
        ),
        3 => cpi::pump::execute_pump_swap_hop3(
            amount_in,
            header_accounts,
            pool_accounts,
            step, 
            is_wsol_pool_0,
            pump_base_amount_out,
        ),
        4 => cpi::raydium::execute_raydium_swap_hop3(
            amount_in,
            header_accounts,
            pool_accounts,
            step,
            is_wsol_pool_0,
        ),
        5 => cpi::clmm::execute_clmm_swap_hop3(
            amount_in,
            header_accounts,
            pool_accounts,
            step,
            is_wsol_pool_0,
        ),
        6 => cpi::whirlpool::execute_whirlpool_swap_hop3(
            amount_in,
            header_accounts,
            pool_accounts,
            step,
            is_wsol_pool_0,
        ),
        _ => Err(PinocchioCpiError::UnsupportedPoolType.into()),
    }
}
