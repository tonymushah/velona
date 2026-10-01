// use arrayvec::ArrayVec;
use url::Url;
use velona_core::reactive::{
    computed::Memo,
    signal::ArcRwSignal,
    traits::{Read, Update},
};

use crate::location::LocationState;

#[derive(Debug, Clone)]
pub struct NavigationController {
    pub(crate) state: ArcRwSignal<LocationState>,
    // pub(crate) history: ArcRwSignal<History>,
}

// struct History {
//     stack: ArrayVec<LocationState, 16>,
//     index: Option<usize>,
// }

// impl History {
//     fn push(&mut self, state: LocationState) {
//         if self.stack.is_full() {
//             self.stack.remove(0);
//         } else {
//             if let Some(i) = self.index.as_mut() {
//                 *i += 1;
//             } else {
//                 self.index.replace(0);
//             }
//         }
//     }
//     fn replace(&mut self, state: LocationState) {
//         if let Some(index) = self.index {
//             if let Some(a) = self.stack.get_mut(index) {
//                 *a = state;
//             }
//         }
//     }
// }

impl NavigationController {
    #[doc(alias = "goto")]
    pub fn push(&self, path: &str) -> Result<(), url::ParseError> {
        if let Some(res) = self.state.write_only().try_maybe_update(|state| {
            if let Err(err_) = state.goto(path) {
                (false, Err(err_))
            } else {
                // {
                //     self.history.write().push(state.clone());
                // }
                (true, Ok(()))
            }
        }) {
            res
        } else {
            Ok(())
        }
    }

    pub fn current(&self) -> Memo<Url> {
        let location = self.state.read_only();
        Memo::new(move |_| location.read().url.clone())
    }
}
