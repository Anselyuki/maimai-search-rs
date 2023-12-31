use std::fs::File;
use std::io::Write;
use std::process::exit;

use log::{error, info, warn};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::config::consts::{CONFIG_PATH, PROFILE};

/// 配置文件解析结果
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub remote_api: RemoteAPIConfig,
    pub markdown: MarkdownConfig,
}

/// 远程配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoteAPIConfig {
    pub json_url: String,
    pub resource_url: String,
    pub maimaidxprober: MaimaiDXProberConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MaimaiDXProberConfig {
    pub data_url: String,
    pub username: Option<String>,
}

/// markdown 配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarkdownConfig {
    pub picture: PictureConfig,
}

/// 远程配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PictureConfig {
    pub local: LocalPictureConfig,
    pub remote: RemotePictureConfig,
    pub console_picture: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalPictureConfig {
    pub enable: bool,
    pub path: Option<String>,
    pub absolute: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemotePictureConfig {
    pub prefix_url: String,
}

impl Profile {
    /// 创建默认配置文件
    ///
    /// 详细的默认配置文件可以参考私有方法:`Profile::default_profile()`
    pub fn create_default() {
        let path = &CONFIG_PATH.join("config.yml");
        // 将 profile 序列化为 YAML 字符串
        let yaml = serde_yaml::to_string(&Self::default_profile()).unwrap();
        // 打开文件并写入 yaml 字符串
        let mut file = match File::create(path) {
            Ok(file) => file,
            Err(e) => {
                error!("无法创建文件: {:?}", e);
                exit(exitcode::IOERR);
            }
        };
        match file.write_all(yaml.as_bytes()) {
            Ok(_) => {
                info!("已成功创建配置文件:{}", path.display());
            }
            Err(e) => {
                error!("无法写入文件{:?}", e);
                exit(exitcode::IOERR);
            }
        }
    }

    pub fn open_config() {
        let path = &CONFIG_PATH.join("config.yml");
        if !path.exists() {
            info!("不存在已有的配置文件,请使用 config --detail(-d) 标志来创建默认配置文件");
            exit(exitcode::OK)
        }
        match open::that(path) {
            Ok(_) => {
                info!("已成功打开配置文件:{}", path.display());
            }
            Err(e) => {
                error!("无法打开文件{:?}", e);
                exit(exitcode::IOERR);
            }
        }
    }

    /// 加载配置文件,默认配置文件为`config.yml`
    ///
    /// > 如果想要创建默认配置文件,请使用`Profile::create_default()`方法
    ///
    /// - 不会抛出异常,最坏的情况下也会返回默认配置文件
    /// - 如果指定的配置文件不存在或解析失败,会产生警告信息提示配置文件配置不正确
    pub fn new() -> Profile
    where
        Profile: DeserializeOwned,
    {
        let path = &CONFIG_PATH.join("config.yml");
        if !path.exists() {
            return Self::default_profile();
        }

        // 通过 std::fs 读取配置文件内容,解析失败也返回默认配置文件
        let yaml_value = match std::fs::read_to_string(path) {
            Ok(file_str) => file_str,
            Err(error) => return Self::error_handler(error.to_string()),
        };
        serde_yaml::from_str(&yaml_value)
            .unwrap_or_else(|error| Self::error_handler(error.to_string()))
    }
    pub fn get_username() -> Option<String> {
        PROFILE.remote_api.maimaidxprober.username.clone()
    }

    /// 处理失败处理,返回默认配置文件
    fn error_handler(error: String) -> Profile {
        warn!("配置文件解析失败,使用默认值\n[Cause]: {}", error);
        info!("因为懒的问题没有配置跳过空字段,所以请在默认配置文件基础上修改喵: (config --default 生成默认配置文件)");
        Self::default_profile()
    }

    /// 默认配置文件的字段
    fn default_profile() -> Profile {
        Profile {
            remote_api: RemoteAPIConfig {
                json_url: "https://www.diving-fish.com/api/maimaidxprober/music_data".to_string(),
                resource_url: "https://www.diving-fish.com/maibot/static.zip".to_string(),
                maimaidxprober: MaimaiDXProberConfig {
                    data_url: "https://www.diving-fish.com/api/maimaidxprober/query/player"
                        .to_string(),
                    username: None,
                },
            },
            markdown: MarkdownConfig {
                picture: PictureConfig {
                    local: LocalPictureConfig {
                        enable: false,
                        path: None,
                        absolute: false,
                    },
                    remote: RemotePictureConfig {
                        prefix_url: "https://www.diving-fish.com/covers/".to_string(),
                    },
                    console_picture: false,
                },
            },
        }
    }
}
