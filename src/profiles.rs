use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use schemars::schema::RootSchema;

use crate::CONFIG_PATH;

// 用来接收application-dev.yml解析结果
#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    pub url: String,
}

impl Profile {
    /// 加载指定配置文件
    ///
    /// 不会抛出异常,即使配置文件不存在或者解析失败
    /// 如果配置文件不存在或解析失败,会产生警告信息提示配置文件配置不正确
    pub fn new() -> Profile where Profile: DeserializeOwned {
        let path = &CONFIG_PATH.join("config.yml");
        // 配置文件不存在则返回默认配置文件
        if !path.exists() { return default_profile(); }
        // 通过 std::fs 读取配置文件内容,解析失败也返回默认配置文件
        let yaml_value = match std::fs::read_to_string(path) {
            Ok(file_str) => file_str,
            Err(error) => {
                println!("配置文件解析失败,使用默认值\n[Cause]: {}", error);
                return default_profile();
            }
        };

        // 2.通过serde_yaml解析读取到的yaml配置转换成json对象
        match serde_yaml::from_str::<RootSchema>(&yaml_value) {
            Ok(root_schema) => {
                // 通过serde_json把json对象转换指定的model
                let data = match serde_json::to_string_pretty(&root_schema) {
                    Ok(data) => data,
                    Err(error) => {
                        println!("解析 RootSchema 失败,使用默认值\n[Cause]: {}", error);
                        return default_profile();
                    }
                };
                match serde_json::from_str::<Profile>(&*data) {
                    Ok(profile) => profile,
                    Err(error) => {
                        println!("解析 Profile 失败,使用默认值\n[Cause]: {}", error);
                        return default_profile();
                    }
                }
            }
            _ => default_profile()
        }
    }
}

fn default_profile() -> Profile {
    Profile { url: "https://www.diving-fish.com/api/maimaidxprober/music_data".to_string() }
}