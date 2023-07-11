#[macro_export]
macro_rules! wo {
    ($struct_name:ident $($field:ident : $value:expr),* $(,)?) => {
        $struct_name {
            $($field: Some($value)),*
        }
    };
}

#[macro_export]
macro_rules! r {
    ($lit:literal) => {
        format!($lit)
    };
}

#[macro_export]
macro_rules! t {
    ($var:ident.$($access:tt).*) => {
        $var.r(|v| &v.$($access).*)
    };
}
