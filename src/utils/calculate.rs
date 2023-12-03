use std::vec::Vec;
// 创建一个结构体，用于输出calculator中的结果
#[allow(unused)]
pub struct CalculateResult {
    pub tap_value: Vec<f32>,
    pub hold_value: Vec<f32>,
    pub slide_value: Vec<f32>,
    pub touch_value: Vec<f32>,
    pub break_value: Vec<f32>,
}
// 接收一个变量，类型是notes
// unused function
#[allow(unused)]
pub fn calculator(notes: &Vec<u32>) -> CalculateResult {
    // TAP HOLD SLIDE TOUCH BREAK
    // 评价値 (%) = 100/(TAP数 +TOUCH数 +HOLD数*2 +SLIDE数*3 +BREAK数*5)
    let basic_value: f32 = 100.0
        / (notes[0] as f32
            + notes[3] as f32
            + (notes[1] * 2) as f32
            + (notes[2] * 3) as f32
            + (notes[4] * 5) as f32);
    // 额外加分 (%) =1 ÷ BREAK数目
    let extra_value = 1.0 / notes[4] as f32;
    let tap_value = [basic_value, basic_value * 0.8, basic_value * 0.5, 0.0];
    let hold_value = [
        basic_value * 2.0,
        basic_value * 2.0 * 0.8,
        basic_value * 2.0 * 0.5,
        0.0,
    ];
    let slide_value = [
        basic_value * 3.0,
        basic_value * 3.0 * 0.8,
        basic_value * 3.0 * 0.5,
        0.0,
    ];
    let touch_value = [basic_value, basic_value * 0.8, basic_value * 0.5, 0.0];
    // BreakValue = basicValue*5.0 + extraValue*1 or 0.75 0.5 / extraValue*0.4 / extraValue*0.3
    // [] = [没落，50落，100落，粉，绿，白]
    let break_value = [
        basic_value * 5.0 + extra_value * 1.0,
        basic_value * 5.0 + extra_value * 0.75,
        basic_value * 5.0 + extra_value * 0.5,
        basic_value * 5.0 * 0.8 + extra_value * 0.4,
        basic_value * 5.0 * 0.5 + extra_value * 0.3,
        0.0,
    ];
    CalculateResult {
        tap_value: tap_value.to_vec(),
        hold_value: hold_value.to_vec(),
        slide_value: slide_value.to_vec(),
        touch_value: touch_value.to_vec(),
        break_value: break_value.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_calculator() {
        let notes = vec![537, 53, 137, 94, 9];
        let result = calculator(&notes);
        // 输出结果
        println!("tap_value: {:?}", result.tap_value);
        println!("hold_value: {:?}", result.hold_value);
        println!("slide_value: {:?}", result.slide_value);
        println!("touch_value: {:?}", result.touch_value);
        println!("break_value: {:?}", result.break_value);
    }
}
