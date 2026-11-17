pub fn is_valid_asset_name(name: &str) -> bool {
    let Some((stem, extension)) = name.rsplit_once('.') else {
        return false;
    };

    let stem_valid = !stem.is_empty()
        && stem
            .split('-')
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit()));

    let extension_valid = !extension.is_empty()
        && !extension.contains('.')
        && extension
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());

    stem_valid && extension_valid
}
