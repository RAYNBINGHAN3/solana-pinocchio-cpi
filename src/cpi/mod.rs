//! # CPI 模块 - Pinocchio 高效实现
//! 
//! 本模块包含各种 DEX 的高效 CPI 调用实现，全部基于 Pinocchio 框架
//! 以实现最低的计算单元消耗和内存使用。

pub mod cpmm;
pub mod dlmm;
pub mod raydium;
pub mod pump;
pub mod dammv2;
pub mod clmm;
pub mod whirlpool;