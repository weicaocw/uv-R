//! 解析 R 的 DCF 格式（`PACKAGES` / `DESCRIPTION` 文件的布局）。
//!
//! 一个 DCF 文档由若干"记录"组成，记录之间用空行分隔；每条记录是若干
//! `字段: 值` 行；以空白开头的行是上一字段的"折行续行"，要拼回去。

use std::collections::BTreeMap;

/// 一条 DCF 记录：字段名 → 值。用 BTreeMap 以便迭代顺序稳定（测试友好）。
pub type Record = BTreeMap<String, String>;

/// 把 DCF 文本解析成若干记录。
///
/// 对不含 `:` 又非续行的行，宽容地忽略（真实 `PACKAGES` 都是规整的，宽容能
/// 让解析器对末尾杂项更鲁棒）。
pub fn parse(input: &str) -> Vec<Record> {
    let mut records = Vec::new();
    let mut current = Record::new();
    let mut last_key: Option<String> = None;

    for line in input.lines() {
        if line.trim().is_empty() {
            // 空行 = 记录分隔
            if !current.is_empty() {
                records.push(std::mem::take(&mut current));
            }
            last_key = None;
            continue;
        }

        if line.starts_with(' ') || line.starts_with('\t') {
            // 折行续行：用空格拼到上一字段（let-chain，Rust 2024）
            if let Some(key) = &last_key
                && let Some(val) = current.get_mut(key)
            {
                val.push(' ');
                val.push_str(line.trim());
            }
            continue;
        }

        if let Some(idx) = line.find(':') {
            let key = line[..idx].trim().to_string();
            let value = line[idx + 1..].trim().to_string();
            current.insert(key.clone(), value);
            last_key = Some(key);
        }
    }
    if !current.is_empty() {
        records.push(current);
    }
    records
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
Package: A3
Version: 1.0.0
Depends: R (>= 2.15.0), xtable, pbapply
License: GPL (>= 2)

Package: aaSEA
Version: 1.1.0
Imports: DT (>= 0.4), networkD3 (>= 0.4),
        shiny (>= 1.0.5)
";

    #[test]
    fn parses_each_record() {
        let recs = parse(SAMPLE);
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0]["Package"], "A3");
        assert_eq!(recs[0]["Version"], "1.0.0");
        assert_eq!(recs[1]["Package"], "aaSEA");
    }

    #[test]
    fn folds_continuation_lines() {
        let recs = parse(SAMPLE);
        let imports = &recs[1]["Imports"];
        // 续行被拼进同一个值，且不含换行
        assert!(imports.contains("shiny (>= 1.0.5)"));
        assert!(!imports.contains('\n'));
    }

    #[test]
    fn ignores_trailing_blank_lines() {
        let recs = parse("Package: x\nVersion: 1.0\n\n\n");
        assert_eq!(recs.len(), 1);
    }
}
