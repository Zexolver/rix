use std::fs;
use std::path::Path;
use crate::errors::RixError;
use crate::{parser, writer, Package};
use rnix::SyntaxKind;

pub fn add_package(upstream_dir: &Path, package: Package, wrapper: Option<String>) -> Result<(), RixError> {
    let file_path = upstream_dir.join(format!("{}.nix", package.group));
    
    if !file_path.exists() {
        fs::write(&file_path, writer::get_empty_group_template())?;
    }
    
    let mut content = fs::read_to_string(&file_path)?;
    let root_node = parser::parse_root_node(&content).map_err(RixError::ParseError)?;
    let list_node = parser::find_list_node(&root_node)
        .ok_or_else(|| RixError::ParseError("No list block [ ... ] found in target file".to_string()))?;

    // Check if package exists using the parser
    let packages = parser::extract_packages_from_list(&list_node);
    if packages.iter().any(|(name, _)| name == &package.name) {
        return Ok(());   
    }

    let description = package.description.unwrap_or_else(|| "Installed via Rix".to_string());
    
    // Inject the nixGL wrapper if a hardware lockfile specifies one
    let formatted_pkg = if let Some(ref w) = wrapper {
        format!("  (pkgs.writeShellScriptBin \"{}\" ''exec ${{pkgs.nixgl.{}}}/bin/{} ${{pkgs.{}}}/bin/{}'') # {}\n",
                package.name, w, w, package.name, package.name, description)
    } else {
        format!("  pkgs.{} # {}\n", package.name, description)
    };

    // TEXT RANGE SURGERY: Find the exact byte coordinate of the closing bracket ']'
    let last_token = list_node.last_token()
        .ok_or_else(|| RixError::ParseError("Could not find closing bracket of list".into()))?;
          
    let insert_index: usize = last_token.text_range().start().into();

    // Inject the string exactly before the closing bracket
    content.insert_str(insert_index, &formatted_pkg);

    // Safety net: validate AST before writing to disk
    writer::write_content_to_file(&file_path, &content)
}

pub fn remove_package_from_file(name: &str, file_path: &Path, _wrapper: Option<String>) -> Result<(), RixError> {
    let mut content = fs::read_to_string(file_path)?;
    let root_node = parser::parse_root_node(&content).map_err(RixError::ParseError)?;
    let list_node = parser::find_list_node(&root_node)
        .ok_or_else(|| RixError::ParseError("No list block [ ... ] found".to_string()))?;

    let mut target_range: Option<std::ops::Range<usize>> = None;

    for child in list_node.children() {
        let text = child.text().to_string().trim().to_string();
        
        // Smart matching: check if it's a standard package or a nixGL wrapped shell script
        let pkg_name = if text.starts_with("(pkgs.writeShellScriptBin") {
            let parts: Vec<&str> = text.split('"').collect();
            if parts.len() >= 3 {
                parts[1].to_string() // Extracts the package name from the quotes
            } else {
                text.clone()
            }
        } else {
            text.strip_prefix("pkgs.").unwrap_or(&text).to_string()
        };

        if pkg_name == name {
            let mut start_idx: usize = child.text_range().start().into();
            let mut end_idx: usize = child.text_range().end().into();

            // 1. Consume trailing whitespace and inline comments
            let mut current_sibling = child.next_sibling_or_token();
            while let Some(sibling) = current_sibling {
                match sibling {
                    rowan::NodeOrToken::Token(token) => {
                        if token.kind() == SyntaxKind::TOKEN_COMMENT {
                            end_idx = token.text_range().end().into();
                        } else if token.kind() == SyntaxKind::TOKEN_WHITESPACE {
                            end_idx = token.text_range().end().into();
                            if token.text().contains('\n') {
                                break; // Stop after capturing the line's trailing newline
                            }
                        } else {
                            break;
                        }
                        current_sibling = token.next_sibling_or_token();
                    }
                    rowan::NodeOrToken::Node(_) => break, // Stop if we hit another package
                }
            }

            // 2. Consume leading indentation (spaces only) so we don't leave empty blank lines
            let mut prev_sibling = child.prev_sibling_or_token();
            while let Some(sibling) = prev_sibling {
                match sibling {
                    rowan::NodeOrToken::Token(token) => {
                        if token.kind() == SyntaxKind::TOKEN_WHITESPACE {
                            let text = token.text();
                            if let Some(pos) = text.rfind('\n') {
                                // DEFENSIVE FIX: The whitespace token contains a newline bundled with spaces.
                                // We advance start_idx past the newline to consume ONLY the indentation spaces.
                                start_idx = usize::from(token.text_range().start()) + pos + 1;
                                break;
                            } else {
                                start_idx = token.text_range().start().into();
                                prev_sibling = token.prev_sibling_or_token();
                            }
                        } else {
                            break;
                        }
                    }
                    rowan::NodeOrToken::Node(_) => break,
                }
            }

            target_range = Some(start_idx..end_idx);
            break;
        }
    }

    if let Some(range) = target_range {
        // TEXT RANGE SURGERY: Slice out the exact bytes of the package, comment, and newline.
        content.replace_range(range, "");
        writer::write_content_to_file(file_path, &content)?;
    }

    Ok(())
}
