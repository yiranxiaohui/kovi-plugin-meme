mod data;
mod meme;

use meme_generator::error::Error;
use meme_generator::meme::{Image, OptionValue};
use meme_generator::get_meme;
use std::collections::HashMap;
use std::env;
use base64::Engine;
use base64::engine::general_purpose;
use kovi::{Message, PluginBuilder as plugin, PluginBuilder};
use kovi::log::{debug, error, info};
use meme_generator::resources::{check_resources};
use reqwest::Client;
use crate::data::{get_data_path, set_data_path};

#[kovi::plugin]
async fn main() {
    let bot = PluginBuilder::get_runtime_bot();
    set_data_path(bot.get_data_path()).await;
    let path = get_data_path();
    unsafe { env::set_var("MEME_HOME", path); }
    check_resources(None).await;
    info!("Check kovi-plugin-meme resources successfully.");
    plugin::on_msg(move |event| {
        async move {
            let text = event.borrow_text().unwrap_or("");
            let qq_number = get_qq_number(event.message.clone()).await;
            let mut options = HashMap::new();
            if text.starts_with("摸摸头") {
                let avatar = generate_image("petpet", qq_number, options).await;
                // 将 path 转为当前系统格式后的字符串
                let msg = Message::new()
                    .add_image(avatar.as_str());
                event.reply(msg);
            } else if text.starts_with("捶爆") || text.starts_with("爆捶") {
                let avatar = generate_image("thump_wildly", qq_number, options).await;
                // 将 path 转为当前系统格式后的字符串
                let msg = Message::new()
                    .add_image(avatar.as_str());
                event.reply(msg);
            } else if text.starts_with("戒导") {
                let avatar = generate_image("abstinence", qq_number, options).await;
                // 将 path 转为当前系统格式后的字符串
                let msg = Message::new()
                    .add_image(avatar.as_str());
                event.reply(msg);
            } else if text.starts_with("小丑面具") {
                options.insert(String::from("behind"), OptionValue::Boolean(true));
                let avatar = generate_image("clown_mask", qq_number, options).await;
                // 将 path 转为当前系统格式后的字符串
                let msg = Message::new()
                    .add_image(avatar.as_str());
                event.reply(msg);
            } else if text.starts_with("小丑") {
                let avatar = generate_image("clown", qq_number, options).await;
                // 将 path 转为当前系统格式后的字符串
                let msg = Message::new()
                    .add_image(avatar.as_str());
                event.reply(msg);
            } else if text.starts_with("手枪") {
                let avatar = generate_image("gun", qq_number, options).await;
                // 将 path 转为当前系统格式后的字符串
                let msg = Message::new()
                    .add_image(avatar.as_str());
                event.reply(msg);
            } else if text.starts_with("上香") {
                options.insert(String::from("gray"), OptionValue::Boolean(true));
                let avatar = generate_image("mourning", qq_number, options).await;
                // 将 path 转为当前系统格式后的字符串
                let msg = Message::new()
                    .add_image(avatar.as_str());
                event.reply(msg);
            } else if text.starts_with("催眠app") {
                let avatar = generate_image("saimin_app", qq_number, options).await;
                // 将 path 转为当前系统格式后的字符串
                let msg = Message::new()
                    .add_image(avatar.as_str());
                event.reply(msg);
            }
        }
    });
}

async fn get_qq_number(message: Message) -> String{
    let mut qq_number= String::new();
    for segment in message.iter() {
        debug!("segment = {:?}", segment);
        if segment.type_ == "at" {
            if let Some(qq) = segment.data.get("qq").and_then(|v| v.as_str()) {
                qq_number = qq.to_string();
            }
        }
    }
    if qq_number.is_empty() {
        return String::new()
    }
    qq_number
}

async fn get_qq_avatar(qq_number: String) -> Image {
    let client = Client::new();
    let url = format!("https://q1.qlogo.cn/g?b=qq&nk={qq_number}&s=640");
    let image = client.get(url).send().await.unwrap().bytes().await.unwrap();
    Image {name: String::new(), data: image.to_vec()}
}

async fn generate_image(key: &str, qq_number: String, options: HashMap<String, OptionValue>) -> String {
    let meme = get_meme(key).expect(format!("表情 `{key}` 不存在").as_str());
    let image = get_qq_avatar(qq_number).await;
    let texts = vec![];
    let result = meme.generate(vec![image], texts, options);
    get_image_base64(result).await
}

async fn get_image_base64(result: Result<Vec<u8>, Error>) -> String {
    match result {
        Err(Error::ImageDecodeError(err)) => {
            error!("图片解码失败：{err}");
            String::new()
        }
        Err(Error::ImageEncodeError(err)) => {
            error!("图片编码失败：{err}");
            String::new()
        }
        Err(Error::ImageAssetMissing(path)) => {
            error!("图片资源缺失：{path}");
            String::new()
        }
        Err(Error::DeserializeError(err)) => {
            error!("反序列化失败：{err}");
            String::new()
        }
        Err(Error::ImageNumberMismatch(min, max, actual)) => {
            let range = {
                if min == max {
                    min.to_string()
                } else {
                    format!("{min}~{max}")
                }
            };
            error!("图片数量不符，应为 {range}，实际传入 {actual}");
            String::new()
        }
        Err(Error::TextNumberMismatch(min, max, actual)) => {
            let range = {
                if min == max {
                    min.to_string()
                } else {
                    format!("{min}~{max}")
                }
            };
            error!("文本数量不符，应为 {range}，实际传入 {actual}");
            String::new()
        }
        Err(Error::TextOverLength(text)) => {
            error!("文字过长：{text}");
            String::new()
        }
        Err(Error::MemeFeedback(feedback)) => {
            error!("{feedback}");
            String::new()
        }
        Ok(result) => {
            let b64 = general_purpose::STANDARD.encode(result);
            let avatar = format!("base64://{}", b64);
            debug!("表情制作成功！");
            avatar
        }
    }
}
