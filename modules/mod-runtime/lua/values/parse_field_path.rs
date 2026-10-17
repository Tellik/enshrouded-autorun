#[derive(Clone, Copy)]
pub(super) enum FieldStep<'a> {
    Name(&'a str),
    Index(usize),
}

pub(super) fn field_path(mut path: &str) -> Result<Vec<FieldStep<'_>>, String> {
    if path.is_empty() || path.len() > 1024 {
        return Err("expected a nonempty field path of at most 1024 bytes".into());
    }
    let mut steps = Vec::new();
    while !path.is_empty() {
        if steps.len() >= 64 {
            return Err("field path exceeds 64 steps".into());
        }
        if let Some(tail) = path.strip_prefix('[') {
            let close = tail.find(']').ok_or("unterminated array index")?;
            let digits = &tail[..close];
            if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err("invalid array index".into());
            }
            let position = digits
                .parse::<usize>()
                .map_err(|_| "array index overflow")?;
            if position == 0 {
                return Err("array indices are one-based".into());
            }
            steps.push(FieldStep::Index(position - 1));
            path = &tail[close + 1..];
        } else {
            let end = path.find(['.', '[', ']']).unwrap_or(path.len());
            if end == 0 {
                return Err("expected a reflected field name".into());
            }
            steps.push(FieldStep::Name(&path[..end]));
            path = &path[end..];
        }
        if let Some(tail) = path.strip_prefix('.') {
            if tail.is_empty() || tail.starts_with(['.', '[', ']']) {
                return Err("expected a field name after '.'".into());
            }
            path = tail;
        } else if !path.is_empty() && !path.starts_with('[') {
            return Err("expected '.' or an array index in field path".into());
        }
    }
    Ok(steps)
}
