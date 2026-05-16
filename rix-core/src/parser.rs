use rnix::{SyntaxKind, SyntaxNode};
use rowan::ast::AstNode;

/// Helper method to recursively look for a list node (`NODE_LIST`)
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

/// Safely crawl the list AST node, extracting package names and their trailing inline comments
pub fn extract_packages_from_list(list_node: &SyntaxNode) -> Vec<(String, String)> {
    let mut items = Vec::new();
    
    for child in list_node.children() {
        if child.kind() == SyntaxKind::NODE_SELECT {
            let text = child.text().to_string().trim().to_string();
            if text.starts_with("pkgs.") {
                let mut current_sibling = child.next_sibling_or_token();
                let mut found_comment = String::from("Managed via Rix");

                while let Some(sibling) = current_sibling {
                    if sibling.kind() == SyntaxKind::NODE_SELECT {
                        break;
                    }
                    if sibling.kind() == SyntaxKind::TOKEN_COMMENT {
                        // Extract text directly from the token variant inside NodeOrToken
                        if let Some(token) = sibling.as_token() {
                            let cleaned = token.text().trim_start_matches('#').trim().to_string();
                            if !cleaned.is_empty() {
                                found_comment = cleaned;
                            }
                        }
                        break;
                    }
                    current_sibling = sibling.next_sibling_or_token();
                }
                items.push((text, found_comment));
            }
        }
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
