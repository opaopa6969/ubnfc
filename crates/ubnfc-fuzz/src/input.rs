//! 入力の生成。受理・拒否・部分消費が混ざるように、導出・変異・純乱数を混ぜる。

use crate::gen::trivia_for;
use crate::model::Grammar;
use crate::rng::Rng;

/// 変異・乱数で使う code point。非 BMP と NUL、改行、引用符を必ず含める。
const ALPHABET: &[char] = &[
    'a',
    'b',
    'c',
    'x',
    'z',
    '0',
    '1',
    '9',
    ' ',
    '\t',
    '\n',
    '\r',
    '(',
    ')',
    ',',
    ';',
    '+',
    '-',
    '*',
    '/',
    ':',
    '=',
    '#',
    '\'',
    '"',
    '\\',
    '\u{0}',
    'あ',
    '😀',
    '\u{10FFFF}',
    '\u{7f}',
    '\u{85}',
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    Derived,
    Mutated,
    Random,
}

#[derive(Clone, Debug)]
pub struct Input {
    pub text: String,
    pub origin: Origin,
}

/// 1 文法ぶんの入力集合。`count` は目安で、空文字は必ず 1 件含める。
pub fn generate(grammar: &Grammar, rng: &mut Rng, count: usize) -> Vec<Input> {
    let trivia = trivia_for(grammar);
    let mut out = vec![Input {
        text: String::new(),
        origin: Origin::Derived,
    }];
    let derived_count = (count / 2).max(1);
    let mut derived = Vec::new();
    for _ in 0..derived_count {
        let text = grammar.derive(rng, &trivia);
        derived.push(text.clone());
        out.push(Input {
            text,
            origin: Origin::Derived,
        });
    }
    while out.len() < count {
        if rng.chance(70) && !derived.is_empty() {
            let base = rng.pick(&derived).clone();
            out.push(Input {
                text: mutate(&base, rng),
                origin: Origin::Mutated,
            });
        } else {
            out.push(Input {
                text: random(rng),
                origin: Origin::Random,
            });
        }
    }
    out.truncate(count.max(1));
    out
}

/// 1 回の変異。code point 単位で行い、UTF-8 の途中で切らない。
pub fn mutate(base: &str, rng: &mut Rng) -> String {
    let mut chars: Vec<char> = base.chars().collect();
    match rng.weighted(&[16, 16, 14, 14, 14, 14, 12]) {
        // 末尾（または先頭）を落とす
        0 => {
            if !chars.is_empty() {
                let cut = rng.below(chars.len());
                if rng.chance(50) {
                    chars.truncate(cut);
                } else {
                    chars.drain(..cut);
                }
            }
        }
        // 部分列の複製
        1 => {
            if !chars.is_empty() {
                let start = rng.below(chars.len());
                let end = rng.between(start, chars.len().saturating_sub(1));
                let slice: Vec<char> = chars[start..=end.max(start)].to_vec();
                let at = rng.below(chars.len() + 1);
                for (offset, ch) in slice.into_iter().enumerate() {
                    chars.insert((at + offset).min(chars.len()), ch);
                }
            }
        }
        // 2 点の入れ替え
        2 => {
            if chars.len() > 1 {
                let a = rng.below(chars.len());
                let b = rng.below(chars.len());
                chars.swap(a, b);
            }
        }
        // 1 code point の挿入
        3 => {
            let at = rng.below(chars.len() + 1);
            chars.insert(at, *rng.pick(ALPHABET));
        }
        // 1 code point の削除
        4 => {
            if !chars.is_empty() {
                let at = rng.below(chars.len());
                chars.remove(at);
            }
        }
        // 1 code point の置換
        5 => {
            if !chars.is_empty() {
                let at = rng.below(chars.len());
                chars[at] = *rng.pick(ALPHABET);
            }
        }
        // trivia / コメントの挿入
        _ => {
            let trivia = [" ", "\t", "\n", "// c\n", "/* c */", "  \n\t"];
            let at = rng.below(chars.len() + 1);
            for (offset, ch) in rng.pick(&trivia).chars().enumerate() {
                chars.insert((at + offset).min(chars.len()), ch);
            }
        }
    }
    chars.into_iter().collect()
}

/// 文法と無関係な乱文字列。
pub fn random(rng: &mut Rng) -> String {
    let length = rng.below(12);
    (0..length).map(|_| *rng.pick(ALPHABET)).collect()
}

/// 入力の縮小候補。まず塊で削り、最後に 1 code point ずつ削る。
pub fn shrink_candidates(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    if chars.is_empty() {
        return out;
    }
    let mut width = chars.len();
    while width > 1 {
        width /= 2;
        let mut start = 0;
        while start < chars.len() {
            let end = (start + width).min(chars.len());
            let mut candidate: Vec<char> = chars[..start].to_vec();
            candidate.extend_from_slice(&chars[end..]);
            if candidate.len() < chars.len() {
                out.push(candidate.into_iter().collect());
            }
            start = end;
        }
    }
    for index in 0..chars.len() {
        let mut candidate = chars.clone();
        candidate.remove(index);
        out.push(candidate.into_iter().collect());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutations_stay_valid_utf8_and_can_introduce_non_bmp_and_nul() {
        let mut rng = Rng::new(11);
        let mut saw_non_bmp = false;
        let mut saw_nul = false;
        for _ in 0..4000 {
            let text = mutate("abc", &mut rng);
            saw_non_bmp |= text.chars().any(|c| c as u32 > 0xFFFF);
            saw_nul |= text.contains('\u{0}');
            assert_eq!(text, text.chars().collect::<String>());
        }
        assert!(saw_non_bmp && saw_nul);
    }

    #[test]
    fn input_shrinking_reaches_the_empty_string_by_repeated_application() {
        let mut current = "abcdefgh😀".to_string();
        let mut steps = 0;
        while !current.is_empty() {
            let next = shrink_candidates(&current)
                .into_iter()
                .min_by_key(|c| c.chars().count())
                .unwrap();
            assert!(next.chars().count() < current.chars().count());
            current = next;
            steps += 1;
            assert!(steps < 50);
        }
    }
}
