/// Preprocessing utilities for multiline statement merging

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

        // Check if this line contains an arrow call but doesn't start with ->
        if line.contains("->") && !trimmed.starts_with("->") {
            // This is the start of a potential multiline arrow call
            let mut merged = line.to_string();
            i += 1;

            // Merge all following lines that start with ->
            while i < lines.len() {
                let next_line = lines[i];
                let next_trimmed = next_line.trim();

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

        result.push(line.to_string());
        i += 1;
    }

    result.join("\n")
}
