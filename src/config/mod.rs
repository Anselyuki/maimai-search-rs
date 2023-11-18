pub mod profiles;

pub mod consts {
    extern crate lazy_static;

    use std::path::PathBuf;

    use lazy_static::lazy_static;
    use platform_dirs::AppDirs;
    use prettytable::color::{GREEN, MAGENTA, RED, WHITE, YELLOW};
    use prettytable::format::*;
    use prettytable::{Attr, Cell};
    use tantivy::schema::Schema;

    use crate::clients::song_data::entity::Song;
    use crate::config::profiles::Profile;

    lazy_static! {
        // 在 MacOS下遵守 XDG 规范,即创建的配置文件夹为 `~/.config/maimai-search`
        pub static ref CONFIG_PATH: PathBuf = AppDirs::new(Some("maimai-search"), true).unwrap().config_dir;
        pub static ref PROFILE: Profile = Profile::new();
        pub static ref DIFFICULT_NAME: Vec<Cell> = vec!["BASIC", "ADVANCED", "EXPERT", "MASTER", "Re:MASTER"].iter()
            .zip(&[GREEN, YELLOW, RED, MAGENTA, WHITE])
            .map(|(difficult, column_color)| Cell::new(difficult).with_style(Attr::ForegroundColor(*column_color)))
            .collect();
        pub static ref LAUNCH_PATH: PathBuf = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
        pub static ref MARKDOWN_TABLE_STYLE: TableFormat = FormatBuilder::new()
            .column_separator('|').borders('|')
            .separators(&[LinePosition::Title], LineSeparator::new('-', '|', '|', '|'))
            .padding(1, 1).build();
        pub static ref SONG_SCHEMA: Schema = Song::init_schema();
    }
}
