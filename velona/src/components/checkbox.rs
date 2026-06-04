use masonry::{
    core::{ArcStr, NewWidget},
    widgets::Checkbox,
};

use crate::widgets::checkbox::NewCheckboxExt;

/// Create a new reactive checkbox.
///
/// This function just call [`NewChecboxExt::new`], _since `<NewWidget<Checkbox> as NewCheckboxExt>::new` is too long_.
pub fn checkbox<Cf, Tf, T>(checked: Cf, text: Tf) -> NewWidget<Checkbox>
where
    Cf: Fn() -> bool + 'static,
    Tf: Fn() -> T + 'static,
    T: Into<ArcStr>,
{
    <NewWidget<Checkbox> as NewCheckboxExt>::new(checked, text)
}
