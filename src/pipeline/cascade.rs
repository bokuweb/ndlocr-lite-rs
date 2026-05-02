#[derive(Clone, Debug, PartialEq)]
pub struct LineCandidate {
    pub idx: usize,
    pub pred_char_cnt: f32,
}
impl LineCandidate {
    pub fn new(idx: usize, pred_char_cnt: f32) -> Self {
        Self { idx, pred_char_cnt }
    }
}

#[derive(Clone, Debug)]
struct LineResult {
    idx: usize,
    text: String,
}

pub fn run_cascade<F30, F50, F100>(
    lines: Vec<LineCandidate>,
    recognize30: F30,
    recognize50: F50,
    recognize100: F100,
) -> Vec<String>
where
    F30: Fn(&LineCandidate) -> String,
    F50: Fn(&LineCandidate) -> String,
    F100: Fn(&LineCandidate) -> String,
{
    run_cascade_with_idx(lines, recognize30, recognize50, recognize100)
        .into_iter()
        .map(|(_, t)| t)
        .collect()
}

pub fn run_cascade_with_idx<F30, F50, F100>(
    lines: Vec<LineCandidate>,
    recognize30: F30,
    recognize50: F50,
    recognize100: F100,
) -> Vec<(usize, String)>
where
    F30: Fn(&LineCandidate) -> String,
    F50: Fn(&LineCandidate) -> String,
    F100: Fn(&LineCandidate) -> String,
{
    let mut t30 = Vec::new();
    let mut t50 = Vec::new();
    let mut t100 = Vec::new();
    for l in lines {
        if l.pred_char_cnt == 3.0 {
            t30.push(l);
        } else if l.pred_char_cnt == 2.0 {
            t50.push(l);
        } else {
            t100.push(l);
        }
    }
    let mut all = Vec::new();
    for l in t30 {
        let s = recognize30(&l);
        if s.chars().count() >= 25 {
            t50.push(l);
        } else {
            all.push(LineResult {
                idx: l.idx,
                text: s,
            });
        }
    }
    for l in t50 {
        let s = recognize50(&l);
        if s.chars().count() >= 45 {
            t100.push(l);
        } else {
            all.push(LineResult {
                idx: l.idx,
                text: s,
            });
        }
    }
    for l in t100 {
        all.push(LineResult {
            idx: l.idx,
            text: recognize100(&l),
        });
    }
    all.sort_by_key(|r| r.idx);
    all.into_iter().map(|r| (r.idx, r.text)).collect()
}
