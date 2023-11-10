use std::collections::HashMap;

/// # 字符串全角转半角
///
/// > 舞萌 DX 里用的用户名都是全角,这就是为什么在机台上看起来 **那 么 宽** 的原因
/// >
/// > 在这里还是做一个转换,把能转换的全角字符都换成半角字符
///
/// - 遍历UTF-16编码字符，并判断是否为全角字符。全角字符的范围是`\u{FF00}`到`\u{FFEF}`
/// - 如果该字符是全角字符，则将其转换为对应的半角字符。全角字符与半角字符的 Unicode 值之间的差是`0xFEE0`
/// - 将UTF-16编码字符重新转换为字符串
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

/// # 计算单首歌曲的 Rating 值
///
/// 计算方法比较简单
///
/// ```text
/// 定数 * MIN(完成率,100.5) /100 * 基础 Rating
/// ```
///
/// - 基础 rating 是一组固定值,类似一个跳变函数,直接看代码
/// - 当你的准度超过 100.5 就只会按照 100.5 来算 Rating 了,所以打到鸟加就没有分辣
///
/// 值向下取整
pub fn compute_ra(ds: f32, achievement: f32) -> i32 {
    let base_ra = match achievement {
        a if a < 50.0 => 7.0,
        a if a < 60.0 => 8.0,
        a if a < 70.0 => 9.6,
        a if a < 75.0 => 11.2,
        a if a < 80.0 => 12.0,
        a if a < 90.0 => 13.6,
        a if a < 94.0 => 15.2,
        a if a < 97.0 => 16.8,
        a if a < 98.0 => 20.0,
        a if a < 99.0 => 20.3,
        a if a < 99.5 => 20.8,
        a if a < 100.0 => 21.1,
        a if a < 100.5 => 21.6,
        _ => 22.4,
    };
    return (ds * (f32::min(achievement, 100.5f32) / 100.0) * base_ra).floor() as i32;
}

/// 获得 Rating 对应的姓名牌文件名
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

static WIDTHS: [(u32, i32); 38] = [
    (126, 1),
    (159, 0),
    (687, 1),
    (710, 0),
    (711, 1),
    (727, 0),
    (733, 1),
    (879, 0),
    (1154, 1),
    (1161, 0),
    (4347, 1),
    (4447, 2),
    (7467, 1),
    (7521, 0),
    (8369, 1),
    (8426, 0),
    (9000, 1),
    (9002, 2),
    (11021, 1),
    (12350, 2),
    (12351, 1),
    (12438, 2),
    (12442, 0),
    (19893, 2),
    (19967, 1),
    (55203, 2),
    (63743, 1),
    (64106, 2),
    (65039, 1),
    (65059, 0),
    (65131, 2),
    (65279, 1),
    (65376, 2),
    (65500, 1),
    (65510, 2),
    (120831, 1),
    (262141, 2),
    (1114109, 1),
];

/// # 获取字符宽度
pub(crate) fn get_char_width(o: u32) -> i32 {
    match o {
        _ => {}
    }
    if o == 0xe || o == 0xf {
        return 0;
    }
    for &(num, wid) in WIDTHS.iter() {
        if o <= num {
            return wid;
        }
    }
    return 1;
}

/// # 计算歌曲标题字符数量
pub(crate) fn column_width(s: &str) -> i32 {
    s.chars().map(|ch| get_char_width(ch as u32)).sum()
}

/// # 截断过长的歌曲标题
///
/// > 使用了一个哈希表来缓存 `get_char_width()` 方法的结果,这样可以避免在每个字符上重复调用该方法
pub(crate) fn change_column_width(s: &str, len: i32) -> String {
    let mut res = 0;
    let mut char_list = Vec::new();
    let mut char_width_cache = HashMap::new();

    for ch in s.chars() {
        res += *char_width_cache
            .entry(ch)
            .or_insert_with(|| get_char_width(ch as u32));
        if res <= len {
            char_list.push(ch);
        } else {
            break;
        }
    }

    let mut output_str = String::with_capacity(char_list.len());
    output_str.extend(char_list);
    output_str
}
