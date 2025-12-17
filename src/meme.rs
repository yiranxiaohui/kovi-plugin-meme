use meme_generator::get_memes;


#[tokio::test]
pub async fn get_list() {
    let list = get_memes()
        .into_iter()
        .enumerate()
        .map(|(i, meme)| {
            let index = i + 1;
            let key = meme.key();
            let info = meme.info();
            let keywords = info.keywords.join("/");
            format!("{index}. {key} ({keywords})")
        })
        .collect::<Vec<_>>()
        .join("\n");
    println!("表情列表：\n{list}");
}