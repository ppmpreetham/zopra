use std::borrow::Cow;
use std::hash::{Hash, Hasher};

pub fn key_hash<T: Hash + ?Sized>(key: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

pub trait ClassInput {
    fn push_class(self, buf: &mut String);
}

fn push_text(buf: &mut String, value: &str) {
    if value.is_empty() {
        return;
    }
    if !buf.is_empty() {
        buf.push(' ');
    }
    buf.push_str(value);
}

impl ClassInput for &str {
    fn push_class(self, buf: &mut String) {
        push_text(buf, self);
    }
}
impl ClassInput for String {
    fn push_class(self, buf: &mut String) {
        push_text(buf, &self);
    }
}
impl ClassInput for &String {
    fn push_class(self, buf: &mut String) {
        push_text(buf, self);
    }
}
impl<'a> ClassInput for Cow<'a, str> {
    fn push_class(self, buf: &mut String) {
        push_text(buf, &self);
    }
}
impl<T: ClassInput> ClassInput for Option<T> {
    fn push_class(self, buf: &mut String) {
        if let Some(value) = self {
            value.push_class(buf);
        }
    }
}
impl ClassInput for () {
    fn push_class(self, _buf: &mut String) {}
}
impl ClassInput for (bool, &str) {
    fn push_class(self, buf: &mut String) {
        if self.0 {
            push_text(buf, self.1);
        }
    }
}

#[doc(hidden)]
pub use tw_merge;

#[macro_export]
macro_rules! cn {
    ($($expr:expr),* $(,)?) => {{
        use $crate::utils::ClassInput;
        let mut __classes = String::new();
        $($expr.push_class(&mut __classes);)*
        $crate::utils::tw_merge::tw_merge!(&__classes)
    }}
}
