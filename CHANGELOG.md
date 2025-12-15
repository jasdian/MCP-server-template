# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2024-12-15

### Added

- **Automatic Tool Registration System** - Major feature implementation
  - Created `mcp-server-macros` procedural macro crate
  - Implemented `#[mcp_tool]` attribute macro for automatic tool registration
  - Integrated `inventory` crate for compile-time tool collection
  - Zero-cost abstraction with link-time initialization
  - Compile-time validations:
    - Tools must be public structs
    - No generic type parameters allowed
    - Automatic duplicate name detection at startup
  - Converted project to Cargo workspace structure
  - Tools now auto-register with just `#[mcp_tool]` attribute

### Changed

- **Tool Registration Workflow** - Breaking change
  - Removed manual `register_tool()` function
  - `initialize_all_tools()` now uses inventory-based auto-discovery
  - Updated README with new tool creation workflow
  - Simplified tool addition to 3 steps:
    1. Create tool with `#[mcp_tool]` attribute
    2. Add module declaration
    3. Done - no manual registration needed

### Removed

- **Manual Registration Code**
  - Removed `register_tool<T>()` public function from `tools/mod.rs`
  - Removed 5 manual registration tests from test suite
  - Cleaned up unused imports in test files

### Technical Details

- **New Dependencies:**
  - `inventory = "0.3"` - Distributed collection system for compile-time registration
  - `mcp-server-macros` - Internal proc-macro crate
    - `syn = "2.0"` with full features
    - `quote = "1.0"`
    - `proc-macro2 = "1.0"`

- **Architecture:**
  - Workspace with 2 members: main crate + macro crate
  - Procedural macro generates `ToolRegistration` trait implementations
  - Inventory collects all `#[mcp_tool]` annotated tools at link-time
  - Auto-discovery happens before `main()` runs

- **Testing:**
  - All 157 tests passing
  - Maintained backward compatibility for existing tools
  - Added compile-time safety checks via macro validations

## [0.1.0] - Initial Release

### Added

- MCP (Model Context Protocol) server implementation
- Authentication system with API key support
- External API key management per user
- Comprehensive test suite (160+ tests)
- Property-based testing with proptest
- JSON-RPC 2.0 protocol support
- Tool validation system with JSON Schema
- CORS support
- Health check endpoint
- Example tool: `get_current_time`
- Full documentation in README.md

### Features

- Bearer token authentication
- User-specific external credentials
- Type-safe tool parameter validation
- Async tool execution
- Comprehensive error handling
- Integration tests with axum-test
- Environment-based configuration

[Unreleased]: https://github.com/yourusername/mcp-server/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/yourusername/mcp-server/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/yourusername/mcp-server/releases/tag/v0.1.0
