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

/// Helper method to safely crawl the list AST node and collect strings/comments
pub fn extract_packages_from_list(list_node: &SyntaxNode) -> Vec<(String, String)> {
    let mut items = Vec::new();
    
    for child in list_node.children() {
        if child.kind() == SyntaxKind::NODE_SELECT {
            let text = child.text().to_string().trim().to_string();
            if text.starts_with("pkgs.") {
                items.push((text, "System utility package".to_string()));
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
