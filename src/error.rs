use {
    num_derive::FromPrimitive,
    pinocchio::program_error::{ProgramError, ToStr},
    thiserror::Error,
};


#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum PinocchioCpiError {
    // 0
    /// 指令数据长度不足
    #[error("Instruction data too short")]
    InstructionDataTooShort,
    
    // 1
    /// 交易金额为零或无效
    #[error("Invalid trade amount: amount cannot be zero")]
    InvalidTradeAmount,
    
    // 2
    /// 不支持的池类型
    #[error("Unsupported pool type")]
    UnsupportedPoolType,
    
    // 3
    /// 账户数量不足
    #[error("Not enough accounts provided")]
    NotEnoughAccounts,
    
    // 4
    /// Token 账户数据无效或损坏
    #[error("Invalid token account data")]
    InvalidTokenAccountData,
    
    // 5
    /// 套利失败：最终余额未增加
    #[error("Arbitrage failed: final balance not greater than initial")]
    ArbitrageFailed,
    
    // 6
    /// Token 余额不足
    #[error("Insufficient token balance")]
    InsufficientBalance,
    
    // 7
    /// 池配置错误
    #[error("Invalid pool configuration")]
    InvalidPoolConfiguration,
    
    // 8
    /// CPI 调用失败
    #[error("Cross-program invocation failed")]
    CpiCallFailed,
    
    // 9
    /// 账户所有者不匹配
    #[error("Account owner mismatch")]
    AccountOwnerMismatch,
    
    // 9
    /// 账户所有者不匹配
    #[error("Pump not supported in step 2")]
    PumpNotSupported,
}

/// 🚀 从自定义错误转换为 ProgramError
impl From<PinocchioCpiError> for ProgramError {
    fn from(e: PinocchioCpiError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

/// 🚀 从原始错误代码反序列化错误（用于调试和测试）
impl TryFrom<u32> for PinocchioCpiError {
    type Error = ProgramError;
    
    fn try_from(error: u32) -> Result<Self, Self::Error> {
        use num_traits::FromPrimitive;
        
        Self::from_u32(error).ok_or(ProgramError::InvalidArgument)
    }
}

/// 🚀 提供可读性强的错误信息（用于日志和调试）
impl ToStr for PinocchioCpiError {
    fn to_str<E>(&self) -> &'static str {
        match self {
            PinocchioCpiError::InstructionDataTooShort => {
                "Error: Instruction data too short - minimum 11 bytes required"
            }
            PinocchioCpiError::InvalidTradeAmount => {
                "Error: Invalid trade amount - amount must be greater than zero"
            }
            PinocchioCpiError::UnsupportedPoolType => {
                "Error: Unsupported pool type - valid types are 0-6"
            }
            PinocchioCpiError::NotEnoughAccounts => {
                "Error: Not enough accounts provided for the operation"
            }
            PinocchioCpiError::InvalidTokenAccountData => {
                "Error: Invalid token account data - account may be uninitialized or corrupted"
            }
            PinocchioCpiError::ArbitrageFailed => {
                "Error: Arbitrage failed - final WSOL balance not greater than initial balance"
            }
            PinocchioCpiError::InsufficientBalance => {
                "Error: Insufficient token balance for the operation"
            }
            PinocchioCpiError::InvalidPoolConfiguration => {
                "Error: Invalid pool configuration - check pool parameters"
            }
            PinocchioCpiError::CpiCallFailed => {
                "Error: Cross-program invocation failed - check target program and accounts"
            }
            PinocchioCpiError::AccountOwnerMismatch => {
                "Error: Account owner mismatch - account not owned by expected program"
            }
            PinocchioCpiError::PumpNotSupported => {
                "Error: Pump not supported in step 2"
            }
        }
    }
}

/// 🚀 便捷的结果类型别名
pub type PinocchioResult<T> = Result<T, PinocchioCpiError>;

