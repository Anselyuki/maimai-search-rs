use rusttype::{point, Scale};
use unicode_segmentation::UnicodeSegmentation;

use crate::utils::file::get_adobe_simhei_font;

/// # 字符串全角转半角
///
/// > 舞萌 DX 里用的用户名都是全角,这就是为什么在机台上看起来 **那 么 宽** 的原因
/// >
/// > 在这里还是做一个转换,把能转换的全角字符都换成半角字符
///
/// - 遍历 UTF-16 编码字符，并判断是否为全角字符。全角字符的范围是 `\u{FF00}` 到 `\u{FFEF}`
/// - 如果该字符是全角字符，则将其转换为对应的半角字符。全角字符与半角字符的 Unicode 值之间的差是 `0xFEE0`
/// - 将 UTF-16 编码字符重新转换为字符串
pub fn string_to_half_width(input_string: &str) -> String {
    let mut utf16_chars: Vec<u16> = input_string.encode_utf16().collect();
    for i in 0..utf16_chars.len() {
        let char_code = utf16_chars[i];
        // 全角空格,直接进行转换
        if char_code == 0x3000 {
            utf16_chars[i] = 0x0020;
        } else if char_code >= 0xFF00 && char_code <= 0xFFEF {
            utf16_chars[i] = char_code - 0xFEE0;
        }
    }
    return String::from_utf16_lossy(&utf16_chars);
}

/// 获得 Rating 对应的姓名牌文件名
#[inline]
pub fn get_ra_pic(rating: u32) -> String {
    format!(
        "UI_CMN_DXRating_S_{}.png",
        match rating {
            ra if ra < 1000 => "01",
            ra if ra < 2000 => "02",
            ra if ra < 4000 => "03",
            ra if ra < 7000 => "04",
            ra if ra < 10000 => "05",
            ra if ra < 12000 => "06",
            ra if ra < 13000 => "07",
            ra if ra < 14500 => "08",
            ra if ra < 15000 => "09",
            _ => "10",
        }
    )
}

/// # 截断过长的歌曲标题
pub fn change_column_width(raw_title: &str, max_width: i32) -> String {
    let mut title = String::new();
    for (_, grapheme) in raw_title.grapheme_indices(true) {
        let font = get_adobe_simhei_font();
        let glyphs: Vec<_> = font
            .layout(title.as_str(), Scale::uniform(16.0), point(0.0, 0.0))
            .collect();
        let title_width: f32 = glyphs
            .iter()
            .map(|g| g.unpositioned().h_metrics().advance_width)
            .sum();
        if title_width + 25.0 > max_width as f32 {
            title.pop().unwrap();
            return format!("{}...", title);
        }
        title.push_str(grapheme);
    }
    title
}
