# AI Trading

An intelligent financial analysis and portfolio management system built with Rust, leveraging AI agents to make investment decisions based on Warren Buffett's principles.

## ⚠️ Important Disclaimer

**This project is for research and educational purposes only. It is not intended for actual trading or investment decisions.**

- **Not Financial Advice**: This system does not provide financial advice and should not be used as the sole basis for investment decisions.
- **No Performance Guarantees**: Past performance does not guarantee future results. AI predictions may be inaccurate.
- **Risk of Loss**: Trading and investing involve substantial risk of loss. You may lose some or all of your investment.
- **Consult Professionals**: Always consult with qualified financial advisors before making investment decisions.
- **Use at Your Own Risk**: The authors and contributors are not responsible for any financial losses incurred through the use of this system.

By using this software, you acknowledge that you understand these risks and agree to use it solely for educational and research purposes.

## Overview

AI Hedgefund is a sophisticated platform that combines financial data analysis with AI-powered decision making to evaluate stocks and manage investment portfolios. The system employs multiple specialized AI agents that work together to analyze financial metrics, assess risk, and make trading decisions.

## Key Features

- **Warren Buffett Agent**: Analyzes stocks based on Warren Buffett's investment principles:
  - Fundamental analysis (ROE, debt-to-equity, operating margin, current ratio)
  - Consistency analysis (earnings growth patterns)
  - Moat analysis (competitive advantages)
  - Management quality assessment
  - Intrinsic value calculation using DCF model

- **Risk Manager Agent**: Controls position sizing based on risk factors:
  - Enforces position limits (max 20% of portfolio per position)
  - Tracks current prices and available cash
  - Ensures proper risk management across the portfolio

- **Portfolio Manager Agent**: Makes final trading decisions:
  - Processes signals from analyst agents
  - Respects position limits and risk parameters
  - Supports long and short positions
  - Manages margin requirements
  - Provides reasoning for each trading decision

## Technical Architecture

### Backend (Rust)

- **Web Server**: Built with Actix-web framework
- **Async Runtime**: Powered by Tokio
- **Data Processing**: Uses Polars for efficient data manipulation
- **API Integration**: Connects to financial data providers

### AI Components

- **LLM Integration**: Supports multiple model providers:
  - Anthropic
  - Groq
  - DeepSeek
  - OpenAI

- **Agent Framework**: Modular design with specialized agents
  - Graph-based state management
  - Data caching and processing
  - Tool integration for financial analysis

## Project Structure

-   `AI_Hedgefund/`
    -   `Cargo.toml`  *# Rust project configuration*
    -   `.env`  *# Environment variables (not in repo)*
    -   `README.md`  *# Project documentation*
    -   `src/`
        -   `main.rs`  *# Application entry point*
        -   `app/`  *# Web application components*
            -   `mod.rs`  *# Module exports*
            -   `config.rs`  *# Configuration management*
            -   `factory.rs`  *# App factory*
            -   `controller/`  *# Request handlers*
                -   `mod.rs`
                -   `agent_controllers.rs`
            -   `models/`  *# Data models*
                -   `mod.rs`
            -   `routes/`  *# API routes*
                -   `mod.rs`
                -   `routes.rs`
            -   `services/`  *# Business logic services*
                -   `mod.rs`
                -   `service.rs`
                -   `agent_service.rs`
        -   `ai_agent/`  *# AI agent framework*
            -   `mod.rs`  *# Module exports*
            -   `agents/`  *# Specialized agents*
                -   `mod.rs`
                -   `warren_buffet.rs`  *# Value investing agent*
                -   `risk_manager.rs`  *# Risk management agent*
                -   `portfolio_manager.rs`  *# Trading decision agent*
            -   `data/`  *# Data processing and caching*
                -   `mod.rs`
                -   `data.rs`
                -   `models.rs`
                -   `cache.rs`
            -   `graph/`  *# Agent state management*
                -   `mod.rs`
                -   `graph.rs`
                -   `state.rs`
            -   `llm/`  *# LLM provider integrations*
                -   `mod.rs`
                -   `models.rs`
                -   `model_provider.rs`
                -   `groq.rs`


## Getting Started

### Prerequisites

- Rust (latest stable version)
- API keys for:
  - LLM providers (Anthropic, Groq, DeepSeek, OpenAI)
  - Financial data providers

### Environment Setup

1. Clone the repository
2. Create a `.env` file with the following variables

### Building and Running

```bash
# Build the project
cargo build

# Run the application
cargo run
```

## Usage

The AI Hedgefund system analyzes stocks and makes trading decisions through its API endpoints. The workflow typically involves:

1. Providing a list of ticker symbols to analyze
2. Setting date ranges for historical data analysis
3. Configuring portfolio parameters (cash, positions, margin requirements)
4. Receiving analysis and trading recommendations from the AI agents

## Acknowledgments
* This project is based on the ai_hedge_fund GitHub repository [text](https://github.com/virattt/ai-hedge-fund)
* Uses concepts from LangChain for agent orchestration and state management
* Implements investment strategies inspired by Warren Buffett's principles
* Built with Rust and modern AI technologies

## Future Improvements and Maintenance

### Performance Optimization

The current implementation experiences performance degradation when analyzing a large number of tickers. Planned optimizations include:

- **Parallel Processing**: Implement concurrent analysis of multiple tickers
- **Data Caching**: Enhance the caching system to reduce redundant API calls
- **Batch Processing**: Add support for processing tickers in batches
- **Database Integration**: Store historical analysis to reduce computation needs

### Agent Framework Enhancements

- **Agent Selection**: Allow users to select specific agents for analysis rather than running all agents
- **Custom Agent Configuration**: Enable customization of agent parameters
- **Agent Marketplace**: Support for pluggable third-party agents
- **Agent Orchestration**: Improve the coordination between agents for more efficient analysis

### User Experience

- **Progress Tracking**: Add real-time progress updates for long-running analyses
- **Result Caching**: Store and retrieve previous analyses to improve response times
- **Scheduled Analysis**: Support for periodic automated analysis of watchlists
- **Notification System**: Alert users when significant trading signals are detected

### Technical Debt

- Refactor the agent state management for better scalability
- Improve error handling and recovery mechanisms
- Enhance logging for better debugging and monitoring
- Implement comprehensive test coverage
