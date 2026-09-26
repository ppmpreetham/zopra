pub trait ClassInput {
    fn into_class_string(self) -> Option<String>;
}

impl ClassInput for &str {
    fn into_class_string(self) -> Option<String> {
        if self.is_empty() { None } else { Some(self.to_string()) }
    }
}

impl ClassInput for String {
    fn into_class_string(self) -> Option<String> {
        if self.is_empty() { None } else { Some(self) }
    }
}

impl<T: ClassInput> ClassInput for Option<T> {
    fn into_class_string(self) -> Option<String> {
        self.and_then(|t| t.into_class_string())
    }
}

#[doc(hidden)]
pub use tw_merge;

#[macro_export]
macro_rules! cn {
    ($($expr:expr),* $(,)?) => {
        {
            use $crate::utils::ClassInput;
            let mut __classes = String::new();
            $(
                if let Some(__c) = $expr.into_class_string() {
                    if !__classes.is_empty() {
                        __classes.push(' ');
                    }
                    __classes.push_str(&__c);
                }
            )*
            $crate::utils::tw_merge::tw_merge!(&__classes)
        }
    }
}
