#![allow(non_snake_case)]

mod app;
mod components;
mod mock_data;
mod routes;
mod state;
mod types;

fn main() {
    dioxus::launch(app::App);
}
