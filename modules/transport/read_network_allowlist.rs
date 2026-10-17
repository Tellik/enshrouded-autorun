pub fn read_network_allowlist(config: &serde_json::Value) -> (Vec<String>, bool) {
    let raw = config
        .pointer("/modules/network/allowedClientSteamIds")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let restrict = !raw.trim().is_empty();
    let ids = raw
        .split([',', ';'])
        .map(str::trim)
        .filter(|peer| {
            (16..=20).contains(&peer.len())
                && peer.bytes().all(|byte| byte.is_ascii_digit())
                && peer.parse::<u64>().is_ok_and(|id| id != 0)
        })
        .map(str::to_owned)
        .collect();
    (ids, restrict)
}
