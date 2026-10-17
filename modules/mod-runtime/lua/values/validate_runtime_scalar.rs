use kfc::reflection::PrimitiveType;

/// Shared scalar policy for every runtime writer, including whole components.
pub(crate) fn validate_runtime_scalar(kind: PrimitiveType, bytes: &[u8]) -> Result<(), String> {
    let finite = match kind {
        PrimitiveType::Float32 => {
            f32::from_le_bytes(bytes.try_into().map_err(|_| "invalid float32 width")?).is_finite()
        }
        PrimitiveType::Float64 => {
            f64::from_le_bytes(bytes.try_into().map_err(|_| "invalid float64 width")?).is_finite()
        }
        _ => true,
    };
    if !finite {
        return Err("runtime field writes require finite floating-point values".into());
    }
    Ok(())
}
