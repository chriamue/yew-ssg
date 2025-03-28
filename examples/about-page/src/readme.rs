use yew::prelude::*;

#[function_component(ReadmePage)]
pub fn readme_page() -> Html {
    let readme_content = include_str!("../../../README.md");

    let html_content = markdown::to_html(readme_content);

    html! {
        <div class="readme-container">
            <div class="markdown-body">
                { Html::from_html_unchecked(AttrValue::from(html_content)) }
            </div>
        </div>
    }
}
