// File: translator-server/src/templates/components.rs

use maud::{html, Markup};

pub fn translation_form() -> Markup {
    html! {
        form is="translation-form" {
            div {
                label for="texts" { "Text to translate (one per line):" }
                textarea id="texts" name="texts" placeholder="Hello\nGoodbye" required {}
            }

            div class="form-row" {
                div {
                    label for="src_lang" { "Source:" }
                    select id="src_lang" name="src_lang" {
                        option value="eng_Latn" selected { "English" }
                        option value="fra_Latn" { "French" }
                        option value="deu_Latn" { "German" }
                        option value="spa_Latn" { "Spanish" }
                    }
                }
                div {
                    label for="tgt_lang" { "Target:" }
                    select id="tgt_lang" name="tgt_lang" {
                        option value="fra_Latn" selected { "French" }
                        option value="eng_Latn" { "English" }
                        option value="deu_Latn" { "German" }
                        option value="spa_Latn" { "Spanish" }
                    }
                }
            }

            button type="submit" { "Translate" }
        }
    }
}

pub fn translation_result() -> Markup {
    html! {
        translation-result {}
    }
}