//! Various utilities components _to make your life a little bit easier_.
// mod checkbox;
// mod fragment;
// mod label;

macro_rules! modules {
    ($($module:ident, )*) => {
        $(
            mod $module;
            pub use $module::*;
        )*
    };
}

modules! {
    checkbox,
    sized_box,
    image,
    label,
}
