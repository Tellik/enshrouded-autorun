use crate::ModManifest;
pub fn infer_api_contract(manifest: &mut ModManifest, source: &str) {
    let normalized: String = strip_lua_comments(source)
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    let assets = normalized.contains("runtime.require(\"game.assets.write\")")
        || normalized.contains("runtime.require('game.assets.write')")
        || [
            "game.assets.update_asset(",
            "game.assets.save_assets(",
            "game.assets.reset_assets(",
            "game.assets.create_resource(",
            "game.assets.create_content(",
        ]
        .iter()
        .any(|method| normalized.contains(method));
    let runtime = normalized.contains("runtime.require(\"runtime.lifecycle\")")
        || normalized.contains("runtime.require('runtime.lifecycle')")
        || normalized.contains("runtime.ecs.")
        || normalized.contains("runtime.world.")
        || normalized.contains("runtime.patch.")
        || normalized.contains("runtime.report_effect")
        || normalized.contains("on_load=")
        || normalized.contains("on_update=")
        || normalized.contains("on_unload=");
    // Capabilities are author-declared contract entries. Source inspection may
    // derive execution metadata, but it must never silently grant permissions.
    let apply_at = if runtime && !assets {
        "live"
    } else {
        "restart"
    };
    for setting in &mut manifest.settings {
        setting.apply_at = apply_at.into();
        setting.restart_required = apply_at != "live";
    }
}

fn strip_lua_comments(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    let mut quote = None;
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment = false;
    while let Some(character) = chars.next() {
        if line_comment {
            if character == '\n' {
                line_comment = false;
                output.push(character);
            }
            continue;
        }
        if block_comment {
            if character == ']' && chars.peek() == Some(&']') {
                chars.next();
                block_comment = false;
            }
            continue;
        }
        if let Some(delimiter) = quote {
            output.push(character);
            if escaped {
                escaped = false;
                continue;
            }
            if character == '\\' {
                escaped = true;
                continue;
            }
            if character == delimiter {
                quote = None;
            }
            continue;
        }
        if character == '\'' || character == '"' {
            quote = Some(character);
            output.push(character);
            continue;
        }
        if character == '-' && chars.peek() == Some(&'-') {
            chars.next();
            if chars.peek() == Some(&'[') {
                chars.next();
                if chars.peek() == Some(&'[') {
                    chars.next();
                    block_comment = true;
                    continue;
                }
                output.push_str("--[");
                continue;
            }
            line_comment = true;
            continue;
        }
        output.push(character);
    }
    output
}
