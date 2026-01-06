// File: translator-server/src/templates/layout.rs

use maud::{html, Markup, DOCTYPE, PreEscaped};

pub fn base(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " - Translation Service" }
                style { (PreEscaped(STYLES)) }
            }
            body {
                header {
                    h1 { "Translation Service" }
                    nav {
                        a href="/" { "Home" }
                        " | "
                        a href="/ui/stats" { "Stats" }
                    }
                }
                main { (content) }
                footer {
                    p { "Built with Rust, Axum, and Web Components" }
                }
                script type="module" { (PreEscaped(COMPONENTS_JS)) }
            }
        }
    }
}

const STYLES: &str = r###"
:root {
    --color-primary: #0066cc;
    --color-success: #006600;
    --color-warning: #996600;
    --color-error: #660000;
    --spacing-md: 1rem;
    --radius: 4px;
}

* { box-sizing: border-box; }

body {
    font-family: system-ui, sans-serif;
    max-width: 800px;
    margin: 0 auto;
    padding: 1rem;
    line-height: 1.6;
}

header { border-bottom: 2px solid #333; padding-bottom: 1rem; margin-bottom: 2rem; }
nav a { color: var(--color-primary); }

form {
    background: #fff;
    padding: 1.5rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

label { display: block; margin-bottom: 0.5rem; font-weight: 600; }
textarea, input, select {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: var(--radius);
    margin-bottom: 1rem;
}
textarea { min-height: 120px; resize: vertical; }

button {
    background: var(--color-primary);
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: var(--radius);
    cursor: pointer;
}
button:hover:not(:disabled) { background: #0052a3; }
button:disabled { background: #ccc; cursor: not-allowed; }

.form-row { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }

.status-pending { color: var(--color-warning); background: #fff3cd; padding: 0.5rem 1rem; border-radius: var(--radius); }
.status-complete { color: var(--color-success); background: #d4edda; padding: 0.5rem 1rem; border-radius: var(--radius); }
.status-error { color: var(--color-error); background: #f8d7da; padding: 0.5rem 1rem; border-radius: var(--radius); }

.translation-result { background: #fff; padding: 1rem; border-radius: 8px; margin-top: 1rem; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
.translation-item { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; padding: 0.5rem 0; border-bottom: 1px solid #eee; }

.spinner {
    display: inline-block;
    width: 1rem; height: 1rem;
    border: 2px solid #ddd;
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

.btn-secondary { background: #6c757d; padding: 0.25rem 0.5rem; font-size: 0.875rem; }
"###;

const COMPONENTS_JS: &str = r###"
// TranslationForm: Handles form submission and polling
class TranslationForm extends HTMLFormElement {
    #pollInterval = null;

    connectedCallback() {
        this.addEventListener('submit', this);
    }

    disconnectedCallback() {
        this.removeEventListener('submit', this);
        if (this.#pollInterval) clearInterval(this.#pollInterval);
    }

    async handleEvent(e) {
        if (e.type === 'submit') {
            e.preventDefault();
            await this.#handleSubmit();
        }
    }

    async #handleSubmit() {
        const formData = new FormData(this);
        const texts = formData.get('texts').split('\n').map(l => l.trim()).filter(l => l);

        if (texts.length === 0) {
            this.#showStatus('error', { message: 'Enter at least one line' });
            return;
        }

        this.#setSubmitDisabled(true);
        this.#showStatus('pending', { message: 'Submitting...' });

        try {
            const response = await fetch('/translate', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    texts,
                    src_lang: formData.get('src_lang'),
                    targets: { result: formData.get('tgt_lang') }
                })
            });

            if (!response.ok) throw new Error((await response.json()).error);

            const { job_id } = await response.json();
            this.#showStatus('pending', { message: 'Translating...', jobId: job_id });
            this.#startPolling(job_id, texts);
        } catch (error) {
            this.#showStatus('error', { message: error.message });
            this.#setSubmitDisabled(false);
        }
    }

    #startPolling(jobId, sourceTexts) {
        let attempts = 0;
        this.#pollInterval = setInterval(async () => {
            if (++attempts > 120) {
                clearInterval(this.#pollInterval);
                this.#showStatus('error', { message: 'Timeout' });
                this.#setSubmitDisabled(false);
                return;
            }

            try {
                const status = await (await fetch(`/jobs/${jobId}`)).json();
                if (status.status === 'complete') {
                    clearInterval(this.#pollInterval);
                    this.#showStatus('complete', { sourceTexts, translations: status.translations });
                    this.#setSubmitDisabled(false);
                } else if (status.status === 'failed') {
                    clearInterval(this.#pollInterval);
                    this.#showStatus('error', { message: status.error });
                    this.#setSubmitDisabled(false);
                }
            } catch (e) { /* retry */ }
        }, 500);
    }

    #showStatus(status, detail) {
        this.dispatchEvent(new CustomEvent('translation-status', { bubbles: true, detail: { status, ...detail } }));
    }

    #setSubmitDisabled(disabled) {
        for (const el of this.elements) if (el.type === 'submit') el.disabled = disabled;
    }
}
customElements.define('translation-form', TranslationForm, { extends: 'form' });

// TranslationResult: Displays status and results
class TranslationResult extends HTMLElement {
    connectedCallback() {
        document.addEventListener('translation-status', this);
    }

    disconnectedCallback() {
        document.removeEventListener('translation-status', this);
    }

    handleEvent(e) {
        if (e.type === 'translation-status') {
            this.#render(e.detail);
        }
    }

    #render(detail) {
        this.innerHTML = '';
        const { status } = detail;

        if (status === 'pending') {
            this.innerHTML = `<div class="status-pending"><span class="spinner"></span> ${detail.message || 'Processing...'}</div>`;
        } else if (status === 'complete') {
            let html = '<div class="status-complete">Complete!</div>';
            for (const [lang, texts] of Object.entries(detail.translations)) {
                html += `<div class="translation-result"><h3>Translated: ${lang}</h3>`;
                detail.sourceTexts.forEach((src, i) => {
                    html += `<div class="translation-item"><div><strong>Original:</strong> ${src}</div><div><strong>Translated:</strong> ${texts[i]}</div></div>`;
                });
                html += '</div>';
            }
            this.innerHTML = html;
        } else if (status === 'error') {
            this.innerHTML = `<div class="status-error"><strong>Error:</strong> ${detail.message}</div>`;
        }
    }
}
customElements.define('translation-result', TranslationResult);
"###;