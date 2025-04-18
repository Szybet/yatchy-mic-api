use log::debug;

pub fn text_match(str1: String, str2: String) -> f32 {
    let ratio = fuzzy_match_flex::partial_ratio(&str1, &str2, None);
    debug!("Ratio is: {} for strings: {} AND {}", ratio, str1, str2);
    return ratio;
}