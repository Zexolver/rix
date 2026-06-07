use rnix::{SyntaxKind, SyntaxNode};
use rowan::ast::AstNode;

/// Helper method to recursively look for a list node (`NODE_LIST`)
/// NOTE: For simple, naked package group files, this correctly catches the main list.
pub fn find_list_node(node: &SyntaxNode) -> Option<SyntaxNode> {
    if node.kind() == SyntaxKind::NODE_LIST {
        return Some(node.clone());
    }
    for child in node.children() {
        if let Some(found) = find_list_node(&child) {
            return Some(found);
        }
    }
    None
}

/// Safely crawl the list AST node, extracting package names and inline comments
pub fn extract_packages_from_list(list_node: &SyntaxNode) -> Vec<(String, String)> {
    let mut items = Vec::new();
    
    // list_node.children() only evaluates actual element sub-nodes.
    // Every node present inside the list brackets represents an explicit package entry.
    for child in list_node.children() {
        let text = child.text().to_string().trim().to_string();
        
        // DEFENSIVE FIX: Normalize "pkgs." prefixes for internal matching logic, 
        // but fallback to the raw text string if it's a naked identifier or custom string.
        // This completely prevents Rix from erasing manually declared utilities.
        let pkg_name = text.strip_prefix("pkgs.").unwrap_or(&text).to_string();
        
        let mut current_sibling = child.next_sibling_or_token();
        let mut found_comment = String::from("Managed via Rix");

        while let Some(sibling) = current_sibling {
            match sibling {
                rowan::NodeOrToken::Node(_) => {
                    // We've hit the next actual package element node. Stop looking for comments.
                    break;
                }
                rowan::NodeOrToken::Token(token) => {
                    if token.kind() == SyntaxKind::TOKEN_COMMENT {
                        let cleaned = token.text().trim_start_matches('#').trim().to_string();
                        if !cleaned.is_empty() {
                            found_comment = cleaned;
                        }
                        break;
                    }
                    current_sibling = token.next_sibling_or_token();
                }
            }
        }
        items.push((pkg_name, found_comment));
    }
    items
}

/// Helper to extract the root syntax node cleanly from raw code
pub fn parse_root_node(content: &str) -> Result<SyntaxNode, String> {
    let parse = rnix::Root::parse(content);
    if !parse.errors().is_empty() {
        return Err(format!("{:?}", parse.errors()));
    }
    Ok(parse.tree().syntax().clone())
}

/// Normalizes raw HTTPS URLs into standard Nix Flake URIs
pub fn normalize_flake_uri(input: &str) -> String {
    let input = input.trim();
    
    if let Some(stripped) = input.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = stripped.trim_end_matches('/').split('/').collect();
        if parts.len() >= 2 {
            return format!("github:{}/{}", parts[0], parts[1]);
        }
    } else if let Some(stripped) = input.strip_prefix("https://gitlab.com/") {
        let parts: Vec<&str> = stripped.trim_end_matches('/').split('/').collect();
        if parts.len() >= 2 {
            return format!("gitlab:{}/{}", parts[0], parts[1]);
        }
    }
    
    // If it's already a valid flake URI (github:...) or local path, return as is
    input.to_string()
}

/// Infers a safe default flake input name from a URI
pub fn infer_flake_alias(uri: &str) -> String {
    // Split by '/' (for paths or repos) or ':' (for flake schemas)
    let last_segment = uri.split(&['/', ':'][..]).last().unwrap_or(uri);
    
    // Strip common suffixes that make for ugly variable names
    let cleaned = last_segment.trim_end_matches(".git");
    
    if cleaned.is_empty() {
        "custom-flake".to_string()
    } else {
        cleaned.to_string()
    }
}
