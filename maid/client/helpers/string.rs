use std::path::Path;

// cache
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static STRING_CACHE: LazyLock<Mutex<HashMap<String, &'static str>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) fn path_to_str(path: &Path) -> &'static str {
    let string = path.to_string_lossy().into_owned();
    if let Some(cached) = STRING_CACHE.lock().unwrap().get(&string) {
        return cached;
    }

    let leaked = Box::leak(string.clone().into_boxed_str());
    STRING_CACHE.lock().unwrap().insert(string, leaked);
    leaked
}

pub(crate) fn trim_start_end(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}
