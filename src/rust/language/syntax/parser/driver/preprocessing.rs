//! Preprocessing utilities for multiline statement merging
/// Preprocess source to merge ALL multiline statements with braces
/// Handles: synth, bind, pattern, let with map/array, emit, etc.
/// Example:
///   bind myMidi -> myBassline {
///       velocity: 80,
///       bpm: 150
///   }
/// becomes:
///   bind myMidi -> myBassline { velocity: 80, bpm: 150 }
pub fn preprocess_multiline_braces(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("#") {
            result.push(line.to_string());
            i += 1;
            continue;
        }

        // Check if this line contains an opening brace
        if line.contains('{') {
            // Count braces to see if it's complete on one line
            let open_braces = line.matches('{').count();
            let close_braces = line.matches('}').count();

            if open_braces > close_braces {
                // Multiline detected - collect all lines until braces are balanced
                let mut merged = line.to_string();
                let mut brace_depth = open_braces - close_braces;
                i += 1;

                while i < lines.len() && brace_depth > 0 {
                    let next_line = lines[i];
                    let next_trimmed = next_line.trim();

                    // Skip comments inside multiline blocks
                    if next_trimmed.starts_with("//") || next_trimmed.starts_with("#") {
                        i += 1;
                        continue;
                    }

                    // Remove inline comments before merging
                    let clean_line = if let Some(comment_pos) = next_line.find("//") {
                        &next_line[..comment_pos]
                    } else if let Some(comment_pos) = next_line.find("#") {
                        &next_line[..comment_pos]
                    } else {
                        next_line
                    };

                    let clean_trimmed = clean_line.trim();
                    if !clean_trimmed.is_empty() {
                        // Add space before appending (unless it's a closing brace)
                        if !clean_trimmed.starts_with('}') && !merged.ends_with('{') {
                            merged.push(' ');
                        } else if clean_trimmed.starts_with('}') {
                            merged.push(' ');
                        }
                        merged.push_str(clean_trimmed);
                    }

                    // Update brace depth (use original line for brace counting)
                    brace_depth += next_line.matches('{').count();
                    brace_depth -= next_line.matches('}').count();

                    i += 1;
                }

                result.push(merged);
                continue; // Don't increment i again
            }
        }

        result.push(line.to_string());
        i += 1;
    }

    result.join("\n")
}

//

/// Preprocess source to merge multiline arrow calls
/// Example:
///   target -> method1(arg)
///           -> method2(arg)
/// becomes:
///   target -> method1(arg) -> method2(arg)
pub fn preprocess_multiline_arrow_calls(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Case A: line contains an arrow call and is not itself a continuation (doesn't start with ->)
        if line.contains("->") && !trimmed.starts_with("->") {
            // This is the start of a potential multiline arrow call
            let mut merged = line.to_string();
            i += 1;

            // Merge all following lines that start with -> (allow interleaved comments/blank lines)
            while i < lines.len() {
                let next_line = lines[i];
                let next_trimmed = next_line.trim();

                // Skip comment or empty lines between arrow continuations
                if next_trimmed.is_empty()
                    || next_trimmed.starts_with("//")
                    || next_trimmed.starts_with("#")
                {
                    i += 1;
                    continue;
                }

                if next_trimmed.starts_with("->") {
                    // Append this line to the merged line
                    merged.push(' ');
                    merged.push_str(next_trimmed);
                    i += 1;
                } else {
                    // Not a continuation line, stop merging
                    break;
                }
            }

            result.push(merged);
            continue; // Don't increment i again
        }

        // Case B: support definitions that set a trigger (e.g. `let t = .bank.kick`) followed by
        // continuation lines that start with `-> effect(...)`. In this case the initial line does
        // not contain '->' but we should treat following '->' lines as part of the same statement.
        // Detect a trigger assignment by checking for '=' and a '.' beginning the RHS.
        if !trimmed.starts_with("->") && line.contains('=') {
            if let Some(rhs) = line.splitn(2, '=').nth(1) {
                if rhs.trim_start().starts_with('.') {
                    // This is a trigger assignment; merge following -> lines
                    let mut merged = line.to_string();
                    let mut j = i + 1;
                    let mut merged_any = false;
                    while j < lines.len() {
                        let next_line = lines[j];
                        let next_trimmed = next_line.trim();

                        // Skip comments/blank lines
                        if next_trimmed.is_empty()
                            || next_trimmed.starts_with("//")
                            || next_trimmed.starts_with("#")
                        {
                            j += 1;
                            continue;
                        }

                        if next_trimmed.starts_with("->") {
                            merged.push(' ');
                            merged.push_str(next_trimmed);
                            merged_any = true;
                            j += 1;
                        } else {
                            break;
                        }
                    }

                    if merged_any {
                        result.push(merged);
                        i = j;
                        continue;
                    }
                    // If no continuation lines, fall through to default handling
                }
            }
        }

        // Case B2: support bare trigger lines like `.bank.kick` followed by indented `->` lines
        // Example:
        //   .myBank.kick
        //       -> speed(1.0)
        //       -> reverse(true)
        // In this case merge the following `->` continuation lines into the trigger line so
        // parse_trigger_line can pick up effects in a single statement.
        if !trimmed.starts_with("->") && trimmed.starts_with('.') && !line.contains("->") {
            let mut merged = line.to_string();
            let mut j = i + 1;
            let mut merged_any = false;
            while j < lines.len() {
                let next_line = lines[j];
                let next_trimmed = next_line.trim();

                // Skip comments/blank lines
                if next_trimmed.is_empty()
                    || next_trimmed.starts_with("//")
                    || next_trimmed.starts_with("#")
                {
                    j += 1;
                    continue;
                }

                if next_trimmed.starts_with("->") {
                    merged.push(' ');
                    merged.push_str(next_trimmed);
                    merged_any = true;
                    j += 1;
                } else {
                    break;
                }
            }

            if merged_any {
                result.push(merged);
                i = j;
                continue;
            }
            // If no continuation lines, fall through
        }

        // Case C: support synth declarations split across lines, e.g.
        //   let mySynth = synth saw
        //       -> lfo(...)
        // If the RHS of an assignment starts with 'synth', merge following '->' continuation lines
        if !trimmed.starts_with("->") && line.contains('=') {
            if let Some(rhs) = line.splitn(2, '=').nth(1) {
                if rhs.trim_start().starts_with("synth") {
                    // This is a synth assignment; merge following -> lines
                    let mut merged = line.to_string();
                    let mut j = i + 1;
                    let mut merged_any = false;
                    while j < lines.len() {
                        let next_line = lines[j];
                        let next_trimmed = next_line.trim();

                        // Skip comments/blank lines
                        if next_trimmed.is_empty()
                            || next_trimmed.starts_with("//")
                            || next_trimmed.starts_with("#")
                        {
                            j += 1;
                            continue;
                        }

                        if next_trimmed.starts_with("->") {
                            merged.push(' ');
                            merged.push_str(next_trimmed);
                            merged_any = true;
                            j += 1;
                        } else {
                            break;
                        }
                    }

                    if merged_any {
                        result.push(merged);
                        i = j;
                        continue;
                    }
                    // If no continuation lines, fall through to default handling
                }
            }
        }

        // Case D: support bare identifier lines followed by indented '->' continuations
        // Example:
        //   mySynth
        //       -> note(C4)
        //       -> duration(2000)
        // Merge into: `mySynth -> note(C4) -> duration(2000)`
        if !trimmed.starts_with("->") && !line.contains("->") {
            // If the line is a single identifier-like token, merge following -> lines
            // Skip reserved keywords (let, var, const, for, if, group, etc.) to avoid
            // accidentally merging declarations with continuations.
            let token = trimmed.split_whitespace().next().unwrap_or("");
            let reserved = [
                "let", "var", "const", "for", "if", "group", "spawn", "on", "automate", "bind",
                "call", "emit", "bank",
            ];
            if !token.is_empty()
                && !reserved.contains(&token)
                && token
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
            {
                let mut merged = line.to_string();
                let mut j = i + 1;
                let mut merged_any = false;
                while j < lines.len() {
                    let next_line = lines[j];
                    let next_trimmed = next_line.trim();

                    if next_trimmed.is_empty()
                        || next_trimmed.starts_with("//")
                        || next_trimmed.starts_with("#")
                    {
                        j += 1;
                        continue;
                    }

                    if next_trimmed.starts_with("->") {
                        merged.push(' ');
                        merged.push_str(next_trimmed);
                        merged_any = true;
                        j += 1;
                    } else {
                        break;
                    }
                }

                if merged_any {
                    // If the previous pushed line was a `let <name> = synth ...` declaration for
                    // the same identifier, attach the chain to that let line instead of
                    // leaving a separate bare target line.
                    // Look backwards for the nearest non-empty previous pushed line and
                    // attempt to attach the chain to a `let <token> =` synth declaration.
                    // Determine the first method name in the chain (the call after the first '->')
                    let mut first_method_name = String::new();
                    if let Some(second_part) = merged.split("->").nth(1) {
                        let second_part = second_part.trim();
                        if let Some(paren_idx) = second_part.find('(') {
                            first_method_name = second_part[..paren_idx].trim().to_string();
                        } else {
                            first_method_name = second_part.to_string();
                        }
                    }

                    // Allowed synth-parameter method names which should be attached to the synth
                    // declaration when used under a bare identifier. Other methods (like `note` or
                    // `duration`) are runtime actions and should remain as separate ArrowCall
                    // statements targeting the identifier.
                    let synth_param_methods =
                        ["type", "adsr", "lfo", "filter", "filters", "options"];

                    let _let_pattern1 = format!("let {} =", token);
                    let _let_pattern2 = format!("let {}=", token);
                    let mut attached = false;

                    // Only attempt to attach the chain to a previous `let` if the chain
                    // begins with a synth-parameter method.
                    if synth_param_methods.contains(&first_method_name.as_str()) {
                        if !result.is_empty() {
                            // Find index of last non-empty previous line
                            let mut k = result.len();
                            while k > 0 {
                                k -= 1;
                                let candidate = result[k].trim();
                                if candidate.is_empty() {
                                    // skip empty/comment lines
                                    continue;
                                }

                                // Be more permissive: accept any previous line that looks like a let assignment
                                // for this token (contains 'let', the token, and an '='). This is robust to
                                // spacing variations like 'let name =', 'let name=' or additional text.
                                let candidate_line = result[k].to_lowercase();
                                if candidate_line.contains("let")
                                    && candidate_line.contains(&token.to_lowercase())
                                    && candidate_line.contains('=')
                                {
                                    // Attach chain part to this candidate line
                                    if let Some(pos) = merged.find(token) {
                                        let chain_part = merged[pos + token.len()..].trim_start();
                                        if !chain_part.is_empty() {
                                            result[k].push(' ');
                                            result[k].push_str(chain_part);
                                        }
                                        i = j;
                                        attached = true;
                                    }
                                }

                                break; // stop after first non-empty line
                            }
                        }
                    }

                    if attached {
                        continue;
                    }

                    // If the first method is a synth-param, we attempted to attach above. If not,
                    // keep the merged bare identifier arrow call as its own statement.
                    if !synth_param_methods.contains(&first_method_name.as_str()) {
                        result.push(merged);
                        i = j;
                        continue;
                    }

                    // If we reach here but not attached (e.g., no matching let found), fall through
                    // and push the merged chain as its own statement.
                    result.push(merged);
                    i = j;
                    continue;
                }
            }
        }

        // Fallback: if this is a let assignment (synth or otherwise) and the subsequent
        // non-empty lines start with '->', merge them. This is a more permissive rule to
        // handle cases where continuation lines were not merged earlier (indentation/formatting).
        if trimmed.starts_with("let ") && !trimmed.starts_with("->") && line.contains('=') {
            // Peek ahead to see if next meaningful line starts with '->'
            let mut j = i + 1;
            let mut found = false;
            while j < lines.len() {
                let nl = lines[j];
                let nt = nl.trim();
                if nt.is_empty() || nt.starts_with("//") || nt.starts_with("#") {
                    j += 1;
                    continue;
                }
                if nt.starts_with("->") {
                    found = true;
                }
                break;
            }

            if found {
                let mut merged = line.to_string();
                i += 1;
                let mut merged_any = false;
                while i < lines.len() {
                    let next_line = lines[i];
                    let next_trimmed = next_line.trim();

                    if next_trimmed.is_empty()
                        || next_trimmed.starts_with("//")
                        || next_trimmed.starts_with("#")
                    {
                        i += 1;
                        continue;
                    }

                    if next_trimmed.starts_with("->") {
                        merged.push(' ');
                        merged.push_str(next_trimmed);
                        merged_any = true;
                        i += 1;
                    } else {
                        break;
                    }
                }

                if merged_any {
                    result.push(merged);
                    continue;
                }
            }
        }

        result.push(line.to_string());
        i += 1;
    }

    result.join("\n")
}
