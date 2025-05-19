use parking_lot::Mutex;
use std::{collections::HashSet, sync::LazyLock};

#[derive(Default)]
struct StringPool {
    pool: HashSet<&'static str>,
}

impl StringPool {
    /// 驻留字符串
    fn intern(&mut self, s: &str) -> &'static str {
        if let Some(&interned) = self.pool.get(s) {
            interned
        } else {
            let leaked: &'static str = Box::leak(Box::from(s));
            self.pool.insert(leaked);
            leaked
        }
    }
}

static STRING_POOL: LazyLock<Mutex<StringPool>> =
    LazyLock::new(|| Mutex::new(StringPool::default()));

pub fn intern_string<S: AsRef<str>>(s: S) -> &'static str {
    STRING_POOL.lock().intern(s.as_ref())
}
