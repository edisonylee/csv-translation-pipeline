// File: translator-server/src/templates/pages.rs

use maud::{html, Markup};
use super::{layout, components};

pub fn home() -> Markup {
    let content = html! {
        h2 { "Translate Text" }
        (components::translation_form())
        div id="result" { (components::translation_result()) }
    };
    layout::base("Home", content)
}

pub fn stats(translation_count: u64, job_count: u64, redis_connected: bool) -> Markup {
    let content = html! {
        h2 { "Statistics" }
        div class="translation-result" {
            table {
                tr { td { "Cached Translations:" } td { (translation_count) } }
                tr { td { "Active Jobs:" } td { (job_count) } }
                tr {
                    td { "Redis:" }
                    td {
                        @if redis_connected { "Connected" } @else { "Disconnected" }
                    }
                }
            }
        }
    };
    layout::base("Stats", content)
}