#![allow(non_snake_case)]

mod app;
mod components;
mod routes;
mod state;

fn main() {
    dioxus::launch(app::App);
}
