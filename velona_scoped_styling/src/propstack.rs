use velona_core::{
    NewWidgetBaseExt,
    masonry_core::core::{Property, PropertyStack, PropertyStackId, Selector},
    reactive::{
        effect::Effect,
        owner::on_cleanup,
        signal::{ArcReadSignal, arc_signal},
        traits::GetUntracked,
    },
    task::spawn_local_scoped_with_cancellation,
};

use crate::{ApplyToNewWidget, use_window_local};

#[derive(Debug)]
pub struct ScopedPropstack {
    id: ArcReadSignal<Option<PropertyStackId>>,
}

impl Default for ScopedPropstack {
    /// Create a new scoped property stack.
    ///
    /// The property stack will be removed [`on_cleanup`].
    fn default() -> Self {
        let window = use_window_local();
        let (id, set_id) = arc_signal(None);
        let data = Self { id: id.clone() };
        {
            let window = window.clone();
            spawn_local_scoped_with_cancellation(async move {
                match window.add_property_stack(PropertyStack::default()).await {
                    Ok(pid) => set_id(Some(pid)),
                    Err(err) => log::error!("Cannot create property stack: {err}"),
                }
            });
        }
        {
            let id = id.clone();
            let window = window.clone();
            on_cleanup(move || {
                if let Some(id) = id.get_untracked()
                    && let Err(err) = window.remove_property_stack(id)
                {
                    log::error!("cannot remove property stack {err}");
                }
            });
        }
        data
    }
}

impl ScopedPropstack {
    pub fn get_id(&self) -> ArcReadSignal<Option<PropertyStackId>> {
        self.id.clone()
    }
    // pub fn with_edit_mode(mut self, edit_mode: EditMode) -> Self {
    //     self.mode = edit_mode;
    //     self
    // }
    pub fn prop_opt<P, Pfn>(self, selector: Selector, prop: Pfn) -> Self
    where
        P: Property,
        Pfn: Fn(Option<P>) -> Option<P> + 'static,
    {
        let id = self.id.clone();
        let window = use_window_local();
        // let tree = use_window_render_root_ref()
        //     .expect("Cannot get the tree render root in the current context");
        Effect::new(move |old_property: Option<Option<P>>| -> Option<P> {
            let id = id()?;
            let old_property = old_property.flatten();
            let new_property = prop(old_property);
            // match mode {
            //     EditMode::Deferred => {
            derrefed_edit(&selector, &window, id, &new_property);
            //     }
            //     EditMode::Immediate => {
            //         immediate_edit(&selector, &tree, id, &new_property);
            //     }
            // };

            new_property
        });
        self
    }
    pub fn prop<P, Pfn>(self, selector: Selector, prop: Pfn) -> Self
    where
        P: Property,
        Pfn: Fn(Option<P>) -> P + 'static,
    {
        self.prop_opt(selector, move |old_prop| Some(prop(old_prop)))
    }
}

fn derrefed_edit<P>(
    selector: &Selector,
    window: &velona_core::window::WindowHandle,
    id: PropertyStackId,
    new_property: &Option<P>,
) where
    P: Property,
{
    let selector = selector.clone();
    let new_property = new_property.clone();
    if let Err(err) = window.edit_property_stack(id, move |edit| {
        match (edit.has_selector(&selector), new_property) {
            (true, None) => {
                edit.edit_last_selector_property_set(&selector, |e| {
                    e.remove::<P>();
                });
            }
            (true, Some(property)) => {
                edit.edit_last_selector_property_set(&selector, |e| {
                    e.insert(property);
                });
            }
            (false, None) => {
                // Nothing...
            }
            (false, Some(property)) => {
                edit.push(selector, property);
            }
        }
    }) {
        log::warn!("cannot change property stack value {err}");
    }
}

// fn immediate_edit<P>(
//     selector: &Selector,
//     tree: &velona_core::render_root::WindowRenderRootRef,
//     id: PropertyStackId,
//     new_property: &Option<P>,
// ) where
//     P: Property,
// {
//     let new_property = new_property.clone();
//     let selector = selector.clone();
//     let res = tree.use_inner_render_root_mut(|inner| {
//         inner.tree.edit_property_stack(id, |edit| {
//             match (edit.has_selector(&selector), new_property) {
//                 (true, None) => {
//                     edit.edit_last_selector_property_set(&selector, |e| {
//                         e.remove::<P>();
//                     });
//                 }
//                 (true, Some(property)) => {
//                     edit.edit_last_selector_property_set(&selector, |e| {
//                         e.insert(property);
//                     });
//                 }
//                 (false, None) => {
//                     // Nothing...
//                 }
//                 (false, Some(property)) => {
//                     edit.push(selector, property);
//                 }
//             }
//         });
//     });
//     if res.is_none() {
//         log::warn!("cannot change property stack value");
//     }
// }

impl ApplyToNewWidget for ScopedPropstack {
    fn apply_to_widget<W>(
        &self,
        new_widget: velona_core::masonry_core::core::NewWidget<W>,
    ) -> velona_core::masonry_core::core::NewWidget<W>
    where
        W: velona_core::masonry_core::core::Widget + ?Sized,
    {
        let id = self.get_id();
        new_widget.use_reactive_widget_erased_mut(move |mut this| {
            if let Some(id) = id() {
                this.ctx.set_property_stack(id);
            }
        })
    }
}
