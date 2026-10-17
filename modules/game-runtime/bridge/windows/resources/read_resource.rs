use crate::windows::native_abi::RuntimeResourceId;
use crate::windows::provider::provider;

fn read_resource_buffer(
    mut fetch: impl FnMut(&mut [u8], &mut usize) -> bool,
) -> Result<Vec<u8>, String> {
    const LIMIT: usize = 256 * 1024 * 1024;
    let mut bytes = vec![0u8; 16 * 1024];
    for attempt in 0..3 {
        let mut actual = 0;
        let success = fetch(&mut bytes, &mut actual);
        if actual > LIMIT {
            return Err("live ResourceService resource exceeded the 256 MiB safety limit".into());
        }
        if success {
            if actual > bytes.len() {
                return Err("native resource read exceeded its output capacity".into());
            }
            bytes.truncate(actual);
            return Ok(bytes);
        }
        if actual <= bytes.len() {
            return Err(
                "live ResourceService resource is unavailable or changed during retrieval".into(),
            );
        }
        if attempt == 2 {
            break;
        }
        bytes.resize(actual, 0);
    }
    Err("live ResourceService resource size kept changing during retrieval".into())
}

pub fn resource_read(resource: &kfc::guid::ResourceId) -> Result<Vec<u8>, String> {
    let _span =
        shroudforge_runtime_diagnostics::operation_timings::span("native.assets.read_resource");
    use kfc::container::StaticHash;
    let fetch = provider()
        .and_then(|value| value.resource_read)
        .ok_or("native provider lacks the ResourceService read extension")?;
    let native = RuntimeResourceId::from(resource);
    let hash = resource.static_hash();
    read_resource_buffer(|bytes, actual| unsafe {
        fetch(&native, hash, bytes.as_mut_ptr(), bytes.len(), actual)
    })
}

#[cfg(test)]
mod resource_buffer_tests {
    use super::read_resource_buffer;

    #[test]
    fn reads_small_snapshots_without_a_size_only_probe_and_retries_larger_owned_buffers() {
        let mut calls = 0;
        let small = read_resource_buffer(|buffer, actual| {
            calls += 1;
            assert!(!buffer.is_empty());
            buffer[..4].copy_from_slice(&[1, 2, 3, 4]);
            *actual = 4;
            true
        })
        .unwrap();
        assert_eq!(small, [1, 2, 3, 4]);
        assert_eq!(calls, 1);
        calls = 0;
        let large = read_resource_buffer(|buffer, actual| {
            calls += 1;
            *actual = 20 * 1024;
            if buffer.len() < *actual {
                return false;
            }
            buffer.fill(42);
            true
        })
        .unwrap();
        assert_eq!(calls, 2);
        assert_eq!(large, vec![42; 20 * 1024]);
    }

    #[test]
    fn never_publishes_failed_or_incomplete_resource_buffers() {
        assert!(
            read_resource_buffer(|buffer, actual| {
                buffer[0] = 99;
                *actual = 4;
                false
            })
            .is_err()
        );
        assert!(
            read_resource_buffer(|buffer, actual| {
                *actual = buffer.len() + 1;
                true
            })
            .is_err()
        );
        assert!(
            read_resource_buffer(|_, actual| {
                *actual = 256 * 1024 * 1024 + 1;
                false
            })
            .unwrap_err()
            .contains("safety limit")
        );
        let mut calls = 0;
        assert!(
            read_resource_buffer(|buffer, actual| {
                calls += 1;
                *actual = buffer.len() + 1;
                false
            })
            .unwrap_err()
            .contains("kept changing")
        );
        assert_eq!(calls, 3);
    }
}
