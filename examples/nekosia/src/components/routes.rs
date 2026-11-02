use velona::widgets::View;
use velona_router::{Route, Router};

use crate::components::image_of_the_day;

pub fn routes() -> impl View {
    Router::default()
        .route(Route::index(image_of_the_day::image_of_the_day))
        .into_view()
}
