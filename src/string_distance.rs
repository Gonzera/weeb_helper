use regex::Regex;

fn clean_string<'a>(str: &str) -> String {
    let pattern = Regex::new(r"[^a-zA-Z0-9\s]").unwrap();

    // Replace all occurrences of the pattern with an empty string
    let mut clean = pattern.replace_all(str, "").into_owned();
    clean = clean.replace(" ", "");
    clean.to_lowercase()
}

fn get_distance(s1: &str, s2: &str) -> usize {
    let mut result = 0;
    if s1 == s2 {}

    let length_1 = s1.chars().count();
    let length_2 = s2.chars().count();

    if length_1 == 0 {
        return length_2;
    }
    if length_2 == 0 {
        return length_1;
    }

    let mut distance_1;
    let mut distance_2;

    let mut cache: Vec<usize> = (1..).take(length_1).collect();

    for (index_2, char_2) in s2.chars().enumerate() {
        result = index_2;
        distance_1 = index_2;

        for (index_1, char_1) in s1.chars().enumerate() {
            distance_2 = if char_1 == char_2 {
                distance_1
            } else {
                distance_1 + 1
            };

            distance_1 = cache[index_1];

            result = if distance_1 > result {
                if distance_2 > result {
                    result + 1
                } else {
                    distance_2
                }
            } else if distance_2 > distance_1 {
                distance_1 + 1
            } else {
                distance_2
            };

            cache[index_1] = result;
        }
    }

    return result;
}

pub fn calc_similarity(s1: &str, s2: &str) -> f64 {
    let s1_clean = clean_string(s1);
    let s2_clean = clean_string(s2);
    let length_1 = s1_clean.len();
    let length_2 = s2_clean.len();
    let result = get_distance(&s1_clean, &s2_clean);

    let max_length = length_1.max(length_2) as f64;

    let similarity = 1.0 - (result as f64 / max_length);

    return similarity * 100.0;
}
