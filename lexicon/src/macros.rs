#[macro_export]
// Macro to easily define an i18n resource.
macro_rules! r {
    (|$args:ident| $lit:literal) => {
        Some(GR::new(|$args| format!($lit)))
    };

    (|($($args:pat),*)| $lit:literal) => {
        Some(GR::new(|($($args),*)| format!($lit)))
    };
    ($lit:literal) => {
        Some(format!($lit))
    };
}

#[macro_export]
macro_rules! t {
    ($var:ident.$($access:tt).*) => {
        $var.r(|v| &v.$($access).*)
    };
}
