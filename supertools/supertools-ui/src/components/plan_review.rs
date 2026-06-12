use leptos::*;

#[component]
pub fn PlanReview(markdown: Signal<String>) -> impl IntoView {
    let html = move || {
        let md = markdown.get();
        if md.is_empty() {
            return "<p class='text-muted'>Loading plan content...</p>".to_string();
        }
        
        let parser = pulldown_cmark::Parser::new(&md);
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);
        html_output
    };

    view! {
        <div class="markdown-view" inner_html=html />
    }
}
