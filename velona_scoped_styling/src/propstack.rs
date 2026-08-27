use velona_core::{
    NewWidgetBaseExt,
    masonry_core::core::{Property, PropertyStack, PropertyStackId, Selector},
    reactive::{
        effect::Effect,
        owner::on_cleanup,
        signal::{ArcReadSignal, ArcWriteSignal, arc_signal},
        traits::{GetUntracked, Update},
    },
    task::spawn_local_scoped_with_cancellation,
};

use crate::{ApplyToNewWidget, PropertyStackUtils, use_window_local};

#[derive(Debug)]
pub struct ScopedPropstack {
    id: ArcReadSignal<Option<PropertyStackId>>,
    stack: ArcWriteSignal<PropertyStack>,
}

impl Default for ScopedPropstack {
    /// Create an empty scoped property stack.
    ///
    /// The property stack will be removed [`on_cleanup`].
    fn default() -> Self {
        Self::new(PropertyStack::default())
    }
}

impl ScopedPropstack {
    /// Create a new scoped property stack.
    ///
    /// The property stack will be removed [`on_cleanup`].
    pub fn new(property_stack: PropertyStack) -> Self {
        let window = use_window_local();
        let (id, set_id) = arc_signal(None);
        let (property_stack, set_property_stack) = arc_signal(property_stack);
        let data = Self {
            id: id.clone(),
            stack: set_property_stack,
        };
        {
            let window = window.clone();
            let property_stack = property_stack.clone();
            spawn_local_scoped_with_cancellation(async move {
                match window
                    .add_property_stack(property_stack.get_untracked())
                    .await
                {
                    Ok(pid) => set_id(Some(pid)),
                    Err(err) => log::error!("Cannot create property stack: {err}"),
                }
            });
        }
        {
            let window = window.clone();
            let id = id.clone();
            Effect::new(move || {
                let property_stack = property_stack();
                let id = id();
                if let Some(property_stack_id) = id {
                    let res = window.replace_property_stack(property_stack_id, property_stack);
                    if let Err(err) = res {
                        log::error!("{err}");
                    }
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
        let stack = self.stack.clone();
        // let tree = use_window_render_root_ref()
        //     .expect("Cannot get the tree render root in the current context");
        Effect::new(move |old_property: Option<Option<P>>| -> Option<P> {
            let old_property = old_property.flatten();
            let new_property = prop(old_property);
            let selector = selector.clone();
            stack.update(|stack| {
                match (stack.has_selector(&selector), &new_property) {
                    (true, None) => {
                        if let Some(set) = stack.get_last_selector_property_set_mut(&selector) {
                            set.remove::<P>();
                        } else {
                            unreachable!("The property set is already in stack");
                        }
                    }
                    (true, Some(property)) => {
                        if let Some(set) = stack.get_last_selector_property_set_mut(&selector) {
                            set.insert(property.clone());
                        } else {
                            unreachable!("The property set is already in stack");
                        }
                    }
                    (false, None) => {
                        // Nothing...
                    }
                    (false, Some(property)) => {
                        stack.push_layer(selector, property.clone());
                    }
                }
            });

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
