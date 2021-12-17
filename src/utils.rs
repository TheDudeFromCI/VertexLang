#[macro_export]
macro_rules! return_if {
    ($c:expr, $v:expr) => {
        if $c {
            return $v;
        }
    };
}

#[macro_export]
macro_rules! continue_if {
    ($c:expr) => {
        if $c {
            continue;
        }
    };
}
