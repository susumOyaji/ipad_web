use wasm_bindgen::prelude::*;
use web_sys::{Request, RequestInit, RequestMode, Response};
use scraper::{Html, Selector, ElementRef};
use serde::{Serialize};
use wasm_bindgen_futures::JsFuture;
use js_sys::{Array,Reflect};
use serde_wasm_bindgen::to_value;



#[derive(Serialize, Debug)]
pub struct Article {
    title: String,
    urls: Vec<String>,
}

#[wasm_bindgen]
pub async fn fetch_movie_keywords() -> Result<JsValue, JsValue> {
    let url = "https://yoshizo.hatenablog.com/entry/microsoft-rewards-search-keyword-list/#movie";
    
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);
    
    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| JsValue::from_str(&format!("リクエストの作成に失敗しました: {:?}", e)))?;

    let window = web_sys::window().unwrap();
    let response_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| JsValue::from_str(&format!("ページの取得に失敗しました: {:?}", e)))?;
    
    let response: Response = response_value.dyn_into().unwrap();
    let text_value = JsFuture::from(response.text().unwrap())
        .await
        .map_err(|e| JsValue::from_str(&format!("レスポンスの読み取りに失敗しました: {:?}", e)))?;
    
    let text = text_value.as_string().unwrap();
    let document = Html::parse_document(&text);

    let title_b_selector = Selector::parse("b").unwrap();
    //let ul_selector = Selector::parse("ul").unwrap();
    let li_selector = Selector::parse("li").unwrap();
    let b_selector = Selector::parse("ul li b").unwrap();

    let mut articles = Vec::new();

    for title_b in document.select(&title_b_selector) {
        let title_text = title_b.text().collect::<Vec<_>>().join("").trim().to_string();
        let mut urls = Vec::new();

        let mut sibling = title_b.next_sibling();
        while let Some(node) = sibling {
            if let Some(element) = ElementRef::wrap(node) {
                if element.value().name() == "ul" {
                    for li in element.select(&li_selector) {
                        for b_tag in li.select(&b_selector) {  
                            let text = b_tag.text().collect::<Vec<_>>().join("").trim().to_string();
                            urls.push(text);
                        }
                    }
                    break;
                }
            }
            sibling = node.next_sibling();
        }

        if !urls.is_empty() {
            articles.push(Article {
                title: title_text,
                urls,
            });
        }
    }

    // Vec<Article> を JsValue に変換
    let js_articles = to_value(&articles).unwrap();
    Ok(js_articles)
}



#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
    match fetch_movie_keywords().await {
        Ok(js_articles) => {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let body = document.body().unwrap();

            // `data-target` の値を取得（設定がない場合は None）
            let target_id = body.get_attribute("data-target");

            // 指定された `data-target` が存在するか確認
            let target = if let Some(id) = target_id {
                document.get_element_by_id(&id).unwrap_or(body.clone().into())
            } else {
                body.clone().into() // `data-target` がない場合は `body` に追加
            };

            // JsValue を配列に変換
            let articles: Array = js_articles.dyn_into().unwrap();

            for article in articles.iter() {
                let article_obj = article.dyn_into::<js_sys::Object>().unwrap();
                let title = Reflect::get(&article_obj, &JsValue::from_str("title")).unwrap();
                let urls = Reflect::get(&article_obj, &JsValue::from_str("urls")).unwrap();

                let title_element = document.create_element("h2").unwrap();
                title_element.set_inner_html(&title.as_string().unwrap());
                target.append_child(&title_element).unwrap();

                let urls_array: Array = urls.dyn_into().unwrap();
                let list = document.create_element("ul").unwrap();
                for url in urls_array.iter() {
                    let list_item = document.create_element("li").unwrap();
                    list_item.set_inner_html(&url.as_string().unwrap()); // チェックボックスを削除
                    list.append_child(&list_item).unwrap();
                }
                target.append_child(&list).unwrap();
            }
        }
        Err(e) => {
            web_sys::console::error_1(&e);
        }
    }
    Ok(())
}


