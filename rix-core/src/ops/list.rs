use crate::errors::RixError;
use crate::{Package, parser, writer};
use rnix::SyntaxKind;
use std::fs;
use std::path::Path;

/// Helper to parse the raw alias name out of an external flake hack string
fn extract_alias(s: &str) -> Option<&str> {
    if let Some(idx) = s.find("__ext_flake or (") {
        let rest = &s[idx + "__ext_flake or (".len()..];
        if let Some(dot) = rest.find('.') {
            return Some(&rest[..dot]);
        }
    }
    None
}

pub fn add_package(
    upstream_dir: &Path,
    package: Package,
    wrapper: Option<String>,
) -> Result<(), RixError> {
    let file_path = upstream_dir.join(format!("{}.nix", package.group));

    if !file_path.exists() {
        fs::write(&file_path, writer::get_empty_group_template())?;
    }

    let mut content = fs::read_to_string(&file_path)?;
    let root_node = parser::parse_root_node(&content).map_err(RixError::ParseError)?;
    let list_node = parser::find_list_node(&root_node).ok_or_else(|| {
        RixError::ParseError("No list block [ ... ] found in target file".to_string())
    })?;

    // Check if package exists using the parser
    let packages = parser::extract_packages_from_list(&list_node);

    // Armored interception: Convert standard single-point resolution into a bulletproof fallback chain
    let mut pkg_name = package.name.clone();
    if pkg_name.contains("__ext_flake or (") {
        if let Some(start_idx) = pkg_name.find("__ext_flake or (") {
            let inner = &pkg_name[start_idx + "__ext_flake or (".len()..];
            if let Some(dot_idx) = inner.find(".packages.") {
                let alias = &inner[..dot_idx];
                pkg_name = format!(
                    "__ext_flake or ({0}.packages.${{pkgs.system}}.default or {0}.defaultPackage.${{pkgs.system}} or {0}.packages.${{pkgs.system}}.{0})",
                    alias
                );
            }
        }
    }

    if packages.iter().any(|(name, _)| {
        name == &pkg_name
            || (extract_alias(name).is_some() && extract_alias(name) == extract_alias(&pkg_name))
    }) {
        return Ok(());
    }

    let description = package
        .description
        .unwrap_or_else(|| "Installed via Rix".to_string());

    // DYNAMIC HARDWARE FIX: Wraps EVERY binary inside $out/bin instead of just {0}
    let formatted_pkg = if wrapper.is_some() {
        format!(
            "  (pkgs.symlinkJoin {{ name = \"{0}-rix-wrap\"; paths = [ pkgs.{0} ]; postBuild = ''\n    if [ -d $out/bin ]; then\n      for bin in $out/bin/*; do\n        filename=$(basename \"$bin\")\n        rm \"$bin\"\n        echo \"#!/bin/sh\" > \"$bin\"\n        echo \"exec ${{pkgs.nixgl.${{import ../../hardware-state.nix}}}}/bin/${{import ../../hardware-state.nix}} ${{pkgs.{0}}}/bin/$filename \\\"\\$@\\\"\" >> \"$bin\"\n        chmod +x \"$bin\"\n      done\n    fi\n  ''; }}) # {1}\n",
            pkg_name, description
        )
    } else {
        format!("  pkgs.{} # {}\n", pkg_name, description)
    };

    // TEXT RANGE SURGERY: Find the exact byte coordinate of the closing bracket ']'
    let last_token = list_node
        .last_token()
        .ok_or_else(|| RixError::ParseError("Could not find closing bracket of list".into()))?;

    let insert_index: usize = last_token.text_range().start().into();

    // Inject the string exactly before the closing bracket
    content.insert_str(insert_index, &formatted_pkg);

    // Safety net: validate AST before writing to disk
    writer::write_content_to_file(&file_path, &content)
}

pub fn remove_package_from_file(
    name: &str,
    file_path: &Path,
    _wrapper: Option<String>,
) -> Result<(), RixError> {
    let mut content = fs::read_to_string(file_path)?;
    let root_node = parser::parse_root_node(&content).map_err(RixError::ParseError)?;
    let list_node = parser::find_list_node(&root_node)
        .ok_or_else(|| RixError::ParseError("No list block [ ... ] found".to_string()))?;

    let mut target_range: Option<std::ops::Range<usize>> = None;

    for child in list_node.children() {
        let text = child.text().to_string().trim().to_string();

        // Smart matching: Support normal packages, old script wrappers, and new symlinkJoin wrappers
        let pkg_name = if text.starts_with("(pkgs.writeShellScriptBin") {
            let parts: Vec<&str> = text.split('"').collect();
            if parts.len() >= 3 {
                parts[1].to_string()
            } else {
                text.clone()
            }
        } else if text.starts_with("(pkgs.symlinkJoin") {
            // Extract the original package name from the "paths = [ pkgs.NAME ];" declaration
            if let Some(start) = text.find("paths = [ pkgs.") {
                let rest = &text[start + 15..];
                let end = rest.find(" ]").unwrap_or(rest.len());
                rest[..end].to_string()
            } else {
                text.clone()
            }
        } else {
            text.strip_prefix("pkgs.").unwrap_or(&text).to_string()
        };

        // Smart removal match: Safely maps simple names ("xplr") or full URLs back to our armored expressions
        let is_match = if pkg_name == name {
            true
        } else if let Some(alias) = extract_alias(&pkg_name) {
            alias == name || extract_alias(name) == Some(alias)
        } else {
            false
        };

        if is_match {
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
                                break;
                            }
                        } else {
                            break;
                        }
                        current_sibling = token.next_sibling_or_token();
                    }
                    rowan::NodeOrToken::Node(_) => break,
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
        content.replace_range(range, "");
        writer::write_content_to_file(file_path, &content)?;
    }

    Ok(())
}
