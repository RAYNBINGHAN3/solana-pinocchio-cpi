[EN](#en) | [ZH-CN](#zh-cn)

---
<img width="1739" height="709" alt="image" src="https://github.com/user-attachments/assets/45238a54-0e4c-4f6e-834d-7c8671b3e9d3" />
---

<a name="en"></a>

# solana-pinocchio-cpi (English)

> **⚠️ Important: This project is a high-performance Solana CPI execution program, designed exclusively for on-chain DEX trade execution. All complex computations (e.g., arbitrage pathfinding, optimal amount calculation) must be performed off-chain.**

---

## Project Overview

`solana-pinocchio-cpi` is a Solana program developed with the [`Pinocchio`](https://github.com/anza-xyz/pinocchio) framework. It provides an extremely low-Units and ultra-efficient CPI (Cross-Program Invocation) layer for interacting with various DEXs.

This project strictly adheres to the **"Off-Chain Calculation, On-Chain Execution"** principle. It does not contain any arbitrage logic or price calculation itself. Instead, it serves as a highly efficient execution engine that receives pre-computed transaction instructions from an off-chain client and executes them optimally on-chain.

### Core Features
- **Ultra-High Performance**: Built on `Pinocchio`, it bypasses the Anchor framework's abstractions to achieve near-native Solana program performance and minimal Units overhead.
- **Pure CPI Execution**: Focuses solely on executing `swap` instructions, with no on-chain state, price oracles, or complex computational logic.
- **Multi-DEX Protocol Support**: Includes built-in CPI adapters for several major DEX protocols, such as:
  - Raydium (CPMM, CLMM)
  - Orca (Whirlpool)
  - Meteora (DLMM, DAMM)
  - Pump.fun
- **Flexible Trading Paths**: Supports 2-hop and 3-hop transaction routes, allowing clients to construct complex arbitrage strategies.
- **Off-Chain Dependency**: Strictly requires the client to perform all calculations off-chain, including finding arbitrage opportunities, determining the trade path, and calculating optimal input amounts and slippage.

## Design Philosophy

In the world of Solana MEV and arbitrage, the cost (gas and compute units) and latency of on-chain computations are critical bottlenecks that limit profitability. The design philosophy of `solana-pinocchio-cpi` is to move all work that can be done asynchronously or in advance to an off-chain environment.

**Workflow:**
1. **Off-Chain Client**: Responsible for monitoring on-chain state, analyzing market conditions, and discovering arbitrage opportunities.
2. **Off-Chain Calculation**: The client computes the full arbitrage path (e.g., WSOL -> USDC -> BONK -> WSOL), the precise input/output amounts for each `swap`, and all required account information.
3. **On-Chain Execution**: The client bundles the results into a single transaction instruction and calls the `solana-pinocchio-cpi` program.
4. **`solana-pinocchio-cpi`**: The program parses the instruction and executes a sequence of CPI calls to the target DEXs in the most efficient manner to complete the arbitrage.

This design significantly reduces transaction complexity and failure rates while maximizing execution speed.

## Directory Structure

- `src/lib.rs`: The program's entry point, responsible for parsing instructions and dispatching them to the appropriate `swap` executors.
- `src/cpi/`: Contains all CPI logic for interacting with specific DEX protocols. Each file corresponds to a DEX or pool type.
- `src/utils.rs`: Utility functions for parsing instruction data and account info.
- `src/error.rs`: Custom error types.

## How to Use

This project is not intended for beginners. It is a low-level tool that must be integrated with a sophisticated off-chain computation engine (Bot).

```rust
// This is a conceptual example of how an off-chain client would use this program.
// In a real-world scenario, you would build and send a complete transaction.

// 1. Calculate the optimal path and amount off-chain
let optimal_path = find_best_arbitrage_path();
let optimal_amount_in = calculate_optimal_amount();

// 2. Prepare all necessary account metadata
let accounts = prepare_accounts_for_path(&optimal_path);

// 3. Construct the instruction data
let instruction_data = encode_instruction_data(&optimal_path, optimal_amount_in);

// 4. Send the transaction
send_transaction_with_instruction(
    MY_PINOCCHIO_PROGRAM_ID,
    accounts,
    instruction_data
);
```

---
<br>

<a name="zh-cn"></a>

# solana-pinocchio-cpi (简体中文)

> **⚠️ 重要提示：本项目是一个超高性能的 Solana CPI 调用程序，专为链上 DEX 交易执行而设计。所有复杂的计算（如套利路径发现、最优金额计算）都应在链下完成。**

---

## 项目简介

`solana-pinocchio-cpi` 是一个基于 [`Pinocchio`](https://github.com/anza-xyz/pinocchio) 框架开发的 Solana 程序，旨在提供一个极低 Units 消耗和超高执行效率的 CPI（跨程序调用）层，用于与各种 DEX 进行交互。

本项目严格遵循 **“链下计算，链上执行”** 的原则。它本身不包含任何套利逻辑或价格计算，而是作为一个高效的执行引擎，接收链下客户端计算好的交易指令，并以最优化的方式在链上执行。

### 核心特性
- **超高性能**: 基于 `Pinocchio` 构建，移除了 Anchor 框架的抽象层，实现了接近原生 Solana 程序的性能和极低的 Units 开销。
- **纯粹的 CPI 执行**: 专注于执行 `swap` 操作，不包含任何链上状态、价格预言机或复杂的计算逻辑。
- **多 DEX 协议支持**: 内置了对多种主流 DEX 协议的 CPI 调用适配，包括：
  - Raydium (CPMM, CLMM)
  - Orca (Whirlpool)
  - Meteora (DLMM, DAMM)
  - Pump.fun
- **灵活的交易路径**: 支持 2-hop 和 3-hop 交易路径，允许客户端构建复杂的套利组合。
- **链下依赖**: 强制要求客户端在链下完成所有计算，包括寻找套利机会、确定交易路径、计算最优输入金额和滑点等。

## 设计哲学

在 Solana MEV 和套利领域，链上计算的成本（Gas 和计算单元）和延迟是限制盈利能力的关键瓶颈。`solana-pinocchio-cpi` 的设计哲学是将所有可以异步和预先完成的工作移至链下。

**工作流程:**
1. **链下客户端**: 负责监控链上状态，分析市场行情，发现套利机会。
2. **链下计算**: 客户端计算出完整的套利路径（例如，WSOL -> USDC -> BONK -> WSOL）、每个 `swap` 的精确输入/输出金额以及所需的账户信息。
3. **链上执行**: 客户端将计算结果打包成一条交易指令，调用 `solana-pinocchio-cpi` 程序。
4. **`solana-pinocchio-cpi`**: 程序解析指令，并以最高效的方式连续调用目标 DEX 的 CPI 接口，完成整个套利过程。

这种设计极大地降低了交易的复杂性和失败率，并最大限度地提高了执行速度。

## 目录结构

- `src/lib.rs`: 程序入口，负责解析指令并分发到不同的 `swap` 执行器。
- `src/cpi/`: 包含了所有与具体 DEX 协议交互的 CPI 调用逻辑。每个文件对应一个 DEX 或池类型。
- `src/utils.rs`: 用于解析指令数据和账户信息的辅助函数。
- `src/error.rs`: 自定义错误类型。

## 如何使用

本项目不适合初学者直接使用。它是一个底层工具，需要与一个成熟的链下计算引擎（Bot）集成。

```rust
// 这是一个链下客户端调用本程序的概念性示例
// 实际使用时，你需要构建一个完整的交易并发送到链上

// 1. 链下计算出最优路径和金额
let optimal_path = find_best_arbitrage_path();
let optimal_amount_in = calculate_optimal_amount();

// 2. 准备所有需要的账户信息
let accounts = prepare_accounts_for_path(&optimal_path);

// 3. 构建指令数据
let instruction_data = encode_instruction_data(&optimal_path, optimal_amount_in);

// 4. 发送交易
send_transaction_with_instruction(
    MY_PINOCCHIO_PROGRAM_ID,
    accounts,
    instruction_data
);
```

