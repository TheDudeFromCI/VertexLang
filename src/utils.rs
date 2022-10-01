#[allow(unused_macros)]

macro_rules! str_vec {
    ( $vec:expr ) => {{
        if !$vec.is_empty() {
            format!(
                "[ {} ]",
                $vec.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(", ")
            )
        } else {
            String::from("[]")
        }
    }};
}

#[allow(unused_imports)]
pub(crate) use str_vec;
