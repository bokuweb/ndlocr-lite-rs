//! 低信頼度の短い横書きを、文字集合不変の2モデル合意で再確認する。

use crate::infer::parseq::RecognizeResult;

/// 元の文字数/種類を保持する2モデル合意だけを採用する。
/// 補助モデルが失敗した場合も、最初の有効な認識を返す。
pub(crate) fn check_short_line_order<F50, F100>(
    base: RecognizeResult,
    width: usize,
    height: usize,
    recognize50: F50,
    recognize100: F100,
) -> RecognizeResult
where
    F50: FnOnce() -> anyhow::Result<RecognizeResult>,
    F100: FnOnce() -> anyhow::Result<RecognizeResult>,
{
    let geometry_ok = height > 0
        && height.checked_mul(2).is_some_and(|min| width >= min)
        && height.checked_mul(6).is_some_and(|max| width <= max);
    let count = base.text.chars().count();
    if !geometry_ok
        || !(2..=4).contains(&count)
        || !base.text.chars().all(char::is_alphanumeric)
        || !(0.0..0.95).contains(&base.mean_confidence)
    {
        return base;
    }
    let Ok(candidate50) = recognize50() else {
        return base;
    };
    let confident = |candidate: &RecognizeResult| {
        (0.98..=1.0).contains(&candidate.mean_confidence)
            && candidate.mean_confidence > base.mean_confidence + 0.03
    };
    let sorted = |text: &str| {
        let mut chars: Vec<_> = text.chars().collect();
        chars.sort_unstable();
        chars
    };
    if !confident(&candidate50)
        || candidate50.text == base.text
        || sorted(&candidate50.text) != sorted(&base.text)
    {
        return base;
    }
    let Ok(candidate100) = recognize100() else {
        return base;
    };
    if confident(&candidate100) && candidate50.text == candidate100.text {
        candidate100
    } else {
        base
    }
}

#[cfg(test)]
#[path = "../../tests/support/short_line_order_policy.rs"]
mod tests;
