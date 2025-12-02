use meme_generator::error::Error;
use meme_generator::meme::Image;
use meme_generator::get_meme;
use std::collections::HashMap;
use std::env;
use std::fs::{read, write};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use base64::Engine;
use base64::engine::general_purpose;
use uuid::Uuid;
use kovi::{Message, PluginBuilder as plugin, PluginBuilder};
use kovi::log::{debug, error};
use kovi::tokio::fs;
use kovi::tokio::sync::Mutex;
use reqwest::Client;

static DATA_PATH: LazyLock<Mutex<String>> = LazyLock::new(|| {Mutex::new(String::new()) });

#[kovi::plugin]
async fn main() {
    let bot = PluginBuilder::get_runtime_bot();
    *DATA_PATH.lock().await = bot.get_data_path().to_str().unwrap().to_string();
    unsafe { env::set_var("MEME_HOME", DATA_PATH.lock().await.as_str()); }
    debug!("{:?}", env::var("MEME_HOME"));
    plugin::on_msg(move |event| {
        async move {
            let text = event.borrow_text().unwrap_or("");
            if text.starts_with("/摸摸头") {
                let mut qq_number= String::new();
                for segment in event.message.iter() {
                    debug!("segment = {:?}", segment);
                    if segment.type_ == "at" {
                        if let Some(qq) = segment.data.get("qq").and_then(|v| v.as_str()) {
                            qq_number = qq.to_string();
                        }
                    }
                }
                if qq_number.is_empty() {
                    return
                }
                let avatar = get_qq_avatar(qq_number.clone()).await;
                let key = "petpet";
                let meme = get_meme(key).expect(format!("表情 `{key}` 不存在").as_str());
                let images= vec![{
                    let data = read(avatar).unwrap();
                    let name = format!("{}.jpg", qq_number);
                    Image {name, data}
                }];
                let texts = vec![];
                let options = HashMap::new();
                let result = meme.generate(images, texts, options);
                let avatar = handle_result(result).await;
                // 将 path 转为当前系统格式后的字符串
                let msg = Message::new()
                    .add_image(avatar.as_str());
                event.reply(msg);
            }
        }
    });
}

async fn get_qq_avatar(qq_number: String) -> String {
    let client = Client::new();
    let url = format!("https://q1.qlogo.cn/g?b=qq&nk={qq_number}&s=640");
    let image = client.get(url).send().await.unwrap().bytes().await.unwrap();
    let path = format!("{}/avatar/{qq_number}.jpg", DATA_PATH.lock().await);
    debug!("path = {path}");
    let path = Path::new(path.as_str());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await.unwrap(); // ✔ 自动创建所有不存在的目录
    }
    write(path, image).expect("头像保存失败!");
    path.to_str().unwrap().to_string()
}

async fn handle_result(result: Result<Vec<u8>, Error>) -> String {
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
