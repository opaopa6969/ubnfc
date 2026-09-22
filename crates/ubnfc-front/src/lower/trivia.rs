use super::*;
pub(super) fn settings(settings: &[a::GlobalSetting]) -> Settings {
    let mut out = Settings {
        comment: None,
        interleave: None,
        whitespace: None,
    };
    for setting in settings {
        match setting {
            a::GlobalSetting::String { key, value, .. } if key == "whitespace" => {
                out.whitespace = Some(style(value))
            }
            a::GlobalSetting::String { key, value, .. } if key == "interleave" => {
                out.interleave = Some(value.clone())
            }
            a::GlobalSetting::Block { key, entries, .. } if key == "comment" => {
                let get = |key| {
                    entries
                        .iter()
                        .find(|e| e.key == key)
                        .map(|e| e.value.clone())
                };
                out.comment = Some(CommentSettings {
                    line: get("line"),
                    block_open: get("blockOpen"),
                    block_close: get("blockClose"),
                });
            }
            _ => {}
        }
    }
    out
}
fn style(value: &str) -> SettingsWhitespace {
    if value.trim().eq_ignore_ascii_case("none") {
        SettingsWhitespace::None
    } else {
        SettingsWhitespace::JavaStyle
    }
}
pub(super) fn policy(cx: &mut Context<'_>, grammar: &r::Grammar, rule: &r::ResolvedRule) -> String {
    let mut raw = settings(&grammar.settings);
    let whitespace = rule.annotations.iter().find_map(|a| match a {
        a::Annotation::Whitespace { style: s, .. } => Some(s.as_deref().unwrap_or("javaStyle")),
        _ => None,
    });
    let interleave = rule.annotations.iter().find_map(|a| match a {
        a::Annotation::Interleave { profile, .. } => Some(profile.clone()),
        _ => None,
    });
    if let Some(s) = whitespace {
        raw.whitespace = Some(style(s));
    }
    if let Some(p) = &interleave {
        raw.interleave = Some(p.clone());
    }
    let enabled = whitespace
        .map(|s| style(s) == SettingsWhitespace::JavaStyle)
        .unwrap_or_else(|| {
            interleave.is_some()
                || raw.whitespace == Some(SettingsWhitespace::JavaStyle)
                || raw.interleave.is_some()
                || raw
                    .comment
                    .as_ref()
                    .is_some_and(|c| c.line.is_some() || c.block_open.is_some())
        });
    // Java の delimiter クラスは grammar で共有する。別 rule が javaStyle を
    // 要求していると comment-only rule でも同じ集合を使う。
    let full=settings(&grammar.settings).whitespace==Some(SettingsWhitespace::JavaStyle)||grammar.rules.iter().any(|r|r.annotations.iter().any(|a|matches!(a,a::Annotation::Interleave {..})||matches!(a,a::Annotation::Whitespace {style:s,..} if s.as_deref().is_none_or(|s|style(s)==SettingsWhitespace::JavaStyle))))||raw.interleave.is_some();
    let mut delimiters = vec![];
    if enabled {
        if full {
            delimiters.push(Delimiter::Characters {
                code_points: vec![9, 10, 11, 12, 13, 32],
            });
        }
        if full || raw.comment.as_ref().is_some_and(|c| c.line.is_some()) {
            delimiters.push(Delimiter::LineComment {
                open: "//".into(),
                line_end_profile: DelimiterLineCommentLineEndProfile,
            });
        }
        if full {
            delimiters.push(Delimiter::BlockComment {
                open: "/*".into(),
                close: "*/".into(),
            });
        }
        if let Some(CommentSettings {
            block_open: Some(open),
            block_close: Some(close),
            ..
        }) = &raw.comment
        {
            let d = Delimiter::BlockComment {
                open: open.clone(),
                close: close.clone(),
            };
            if !delimiters.contains(&d) {
                delimiters.push(d);
            }
        }
    }
    if let Some(p) = cx
        .out
        .semantics
        .trivia
        .policies
        .iter()
        .find(|p| p.raw_settings == raw && p.effective_delimiter_set == delimiters)
    {
        return p.id.clone();
    }
    let base = if enabled {
        "trivia:javaStyle"
    } else {
        "trivia:none"
    };
    let id = if cx
        .out
        .semantics
        .trivia
        .policies
        .iter()
        .any(|p| p.id == base)
    {
        format!("{base}:{}", cx.sid(rule.id))
    } else {
        base.into()
    };
    cx.out.semantics.trivia.policies.push(TriviaPolicy {
        id: id.clone(),
        raw_settings: raw,
        effective_delimiter_set: delimiters,
        precedence: TriviaPolicyPrecedence,
    });
    id
}
pub(super) fn insert(
    cx: &mut Context<'_>,
    policy: &str,
    id: &ExprId,
    at: BoundaryAt,
    index: Option<u64>,
) {
    if cx
        .out
        .semantics
        .trivia
        .policies
        .iter()
        .any(|p| p.id == policy && !p.effective_delimiter_set.is_empty())
    {
        cx.out
            .semantics
            .trivia
            .insertion_plan
            .push(TriviaInsertion {
                policy_id: policy.into(),
                boundary: Boundary {
                    at,
                    child_index: index,
                    owner_expr_id: id.clone(),
                },
            });
    }
}
pub(super) fn chain(cx: &mut Context<'_>, policy: &str, id: &ExprId, n: usize) {
    insert(cx, policy, id, BoundaryAt::Entry, None);
    for i in 0..n {
        insert(cx, policy, id, BoundaryAt::AfterChild, Some(i as u64));
    }
}
