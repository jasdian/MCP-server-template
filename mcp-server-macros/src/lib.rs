use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error, Visibility};

/// Attribute macro to automatically register MCP tools
///
/// Usage:
/// ```rust
/// #[mcp_tool]
/// pub struct MyTool;
///
/// impl McpTool for MyTool {
///     // ... trait implementation
/// }
/// ```
///
/// This macro:
/// 1. Validates the type is a public struct
/// 2. Generates a `ToolRegistration` trait implementation
/// 3. Submits the tool to the inventory for automatic collection
///
/// # Errors
///
/// - Applying to non-struct types (enums, unions) produces compile error
/// - Applying to private structs produces compile error
/// - Applying to generic structs produces compile error
#[proc_macro_attribute]
pub fn mcp_tool(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    match generate_tool_registration(&input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_tool_registration(input: &DeriveInput) -> Result<TokenStream, Error> {
    // Validate it's a struct
    match &input.data {
        Data::Struct(_) => {}
        Data::Enum(_) => {
            return Err(Error::new_spanned(
                input,
                "#[mcp_tool] cannot be applied to enums. Only structs implementing McpTool can be tools.",
            ));
        }
        Data::Union(_) => {
            return Err(Error::new_spanned(
                input,
                "#[mcp_tool] cannot be applied to unions. Only structs implementing McpTool can be tools.",
            ));
        }
    }

    // Validate it's public
    if !matches!(&input.vis, Visibility::Public(_)) {
        return Err(Error::new_spanned(
            &input.vis,
            "MCP tools must be public. Use `pub struct` instead of a private struct.",
        ));
    }

    // Validate no generics (can't box generic types)
    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &input.generics,
            "MCP tools cannot have generic parameters. Tools must be concrete types.",
        ));
    }

    let name = &input.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;

    // Generate the expanded code
    let expanded = quote! {
        // Preserve original attributes and visibility
        #(#attrs)*
        #vis struct #name;

        // Implement ToolRegistration trait for type safety
        impl crate::tools::ToolRegistration for #name {
            fn tool_instance() -> ::std::boxed::Box<dyn crate::tools::McpTool + Send + Sync> {
                ::std::boxed::Box::new(Self)
            }
        }

        // Submit to inventory for automatic collection
        ::inventory::submit! {
            crate::tools::ToolEntry {
                constructor: <#name as crate::tools::ToolRegistration>::tool_instance,
            }
        }
    };

    Ok(TokenStream::from(expanded))
}
