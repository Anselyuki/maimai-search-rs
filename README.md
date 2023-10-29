# maimai-search-rs

使用 maimaidxprober 的 json 数据的命令行小工具

> 数据来源[舞萌 DX 查分器](https://github.com/Diving-Fish/maimaidx-prober)，感谢大佬提供的 API 接口与数据

主要的功能是查找歌曲的难度，以及查找难度的歌曲,由于本人是 Rust 初学者，故从自己的需求入手写一个小工具，仅支持命令行模式请求

## 注意事项

项目使用了 SQLite 数据库，在 MacOS 与 Linux/UNIX 平台上遵守 XDG 规范，数据库与配置文件均放置于 `~/.config/maimai-search`
路径下

可以选择把本程序放置于 PATH 下，或者在使用时指定路径

## 主要功能

### 更新歌曲数据

> 只要在使用前运行一次即可，不需要每次都运行

```bash
maimai-search update
```

通过添加`--md`参数可以将歌曲信息输出为 Markdown 表格

## TODO:

- 添加推分 list 功能，可以将自己的推分列表导入到数据库中