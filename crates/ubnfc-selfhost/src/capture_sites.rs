//! golden IR の capture site → 宣言 rule。生成器の出力は変更しない。
//! tests/regenerate.rs で全 site の一致を検証する。
pub(crate) const CAPTURE_RULES: &[(&str, &str)] = &[
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:1153:1168:body/0/repeat/capture/0",
        "UBNF::UBNFFile",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:1336:1346:body/1/tokenRef/capture/0",
        "UBNF::ImportDecl",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:1361:1367:body/3/tokenRef/capture/0",
        "UBNF::ImportDecl",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:1517:1527:body/1/tokenRef/capture/0",
        "UBNF::GrammarDecl",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:1550:1564:body/3/repeat/capture/0",
        "UBNF::GrammarDecl",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:1584:1601:body/4/repeat/capture/0",
        "UBNF::GrammarDecl",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:1619:1632:body/5/repeat/capture/0",
        "UBNF::GrammarDecl",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:1652:1664:body/6/repeat/capture/0",
        "UBNF::GrammarDecl",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:1789:1799:body/1/tokenRef/capture/0",
        "UBNF::GlobalSetting",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:1809:1821:body/3/ruleRef/capture/0",
        "UBNF::GlobalSetting",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:1982:1998:body/0/ruleRef/capture/0",
        "UBNF::StringSettingValue",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:2088:2104:body/1/repeat/capture/0",
        "UBNF::BlockSettingValue",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:2189:2199:body/0/tokenRef/capture/0",
        "UBNF::KeyValuePair",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:2209:2215:body/2/tokenRef/capture/0",
        "UBNF::KeyValuePair",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:2365:2375:body/1/tokenRef/capture/0",
        "UBNF::TokenDecl",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:2386:2396:body/3/ruleRef/capture/0",
        "UBNF::TokenDecl",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:2833:2839:body/2/tokenRef/capture/0",
        "UBNF::UntilExpression",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:2973:2979:body/2/tokenRef/capture/0",
        "UBNF::NegationExpression",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:3116:3122:body/2/tokenRef/capture/0",
        "UBNF::LookaheadExpression",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:3295:3301:body/2/tokenRef/capture/0",
        "UBNF::NegativeLookaheadExpression",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:3441:3447:body/2/tokenRef/capture/0",
        "UBNF::CharRangeExpression",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:3457:3463:body/4/tokenRef/capture/0",
        "UBNF::CharRangeExpression",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:3567:3573:body/2/tokenRef/capture/0",
        "UBNF::CIExpression",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:3693:3699:body/2/tokenRef/capture/0",
        "UBNF::RegexExpression",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:4115:4129:body/0/repeat/capture/0",
        "UBNF::RuleDecl",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:4148:4158:body/1/tokenRef/capture/0",
        "UBNF::RuleDecl",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:4176:4184:body/3/ruleRef/capture/0",
        "UBNF::RuleDecl",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:4949:4959:body/2/tokenRef/capture/0",
        "UBNF::MappingAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:5001:5011:body/3/0/4/tokenRef/capture/0",
        "UBNF::MappingAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:5030:5040:body/3/0/5/0/1/tokenRef/capture/0",
        "UBNF::MappingAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:5154:5166:body/2/ruleRef/capture/0",
        "UBNF::EvalAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:5182:5194:body/3/0/1/ruleRef/capture/0",
        "UBNF::EvalAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:5312:5322:body/1/0/1/tokenRef/capture/0",
        "UBNF::WhitespaceAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:5452:5462:body/4/tokenRef/capture/0",
        "UBNF::InterleaveAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:5577:5587:body/4/tokenRef/capture/0",
        "UBNF::BackrefAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:5705:5715:body/4/tokenRef/capture/0",
        "UBNF::ScopeTreeAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:5847:5857:body/4/tokenRef/capture/0",
        "UBNF::DeclaresAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:5890:5896:body/5/0/3/tokenRef/capture/0",
        "UBNF::DeclaresAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:6023:6029:body/4/tokenRef/capture/0",
        "UBNF::CatalogAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:6499:6515:body/4/tokenRef/capture/0",
        "UBNF::PrecedenceAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:6605:6611:body/2/tokenRef/capture/0",
        "UBNF::DocAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:6740:6746:body/2/0/0/2/tokenRef/capture/0",
        "UBNF::RecoveryAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:6761:6771:body/2/0/1/tokenRef/capture/0",
        "UBNF::RecoveryAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:7027:7037:body/2/tokenRef/capture/0",
        "UBNF::CommonFieldAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:7056:7066:body/3/0/1/tokenRef/capture/0",
        "UBNF::CommonFieldAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:7277:7287:body/1/tokenRef/capture/0",
        "UBNF::SimpleAnnotation",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:7407:7419:body/0/ruleRef/capture/0",
        "UBNF::ChoiceBody",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:7440:7452:body/1/0/1/ruleRef/capture/0",
        "UBNF::ChoiceBody",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:7554:7574:body/0/repeat/capture/0",
        "UBNF::SequenceBody",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:7740:7753:body/0/0/ruleRef/capture/0",
        "UBNF::AnnotatedElement",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:7781:7794:body/1/ruleRef/capture/0",
        "UBNF::AnnotatedElement",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:7898:7908:body/5/0/1/tokenRef/capture/0",
        "UBNF::AnnotatedElement",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:8539:8547:body/1/ruleRef/capture/0",
        "UBNF::GroupElement",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:8651:8659:body/1/ruleRef/capture/0",
        "UBNF::OptionalElement",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:8764:8772:body/1/ruleRef/capture/0",
        "UBNF::RepeatElement",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:8894:8900:body/2/tokenRef/capture/0",
        "UBNF::ErrorElement",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:9008:9014:body/0/tokenRef/capture/0",
        "UBNF::TerminalElement",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:9351:9369:body/0/repeat/capture/0",
        "UBNF::QuantifiedRef",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:9381:9391:body/1/tokenRef/capture/0",
        "UBNF::QuantifiedRef",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:9532:9542:body/2/tokenRef/capture/0",
        "UBNF::TypeofElement",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:9663:9673:body/0/tokenRef/capture/0",
        "UBNF::DottedIdentifier",
    ),
    (
        "expr:unlaxer-dsl/grammar/ubnf.ubnf:9687:9697:body/1/0/1/tokenRef/capture/0",
        "UBNF::DottedIdentifier",
    ),
];

/// `CAPTURE_RULES` を site_id で引く。parse のたび 1,000 件超の capture を
/// 62 件の線形探索で照合していたため、プロセス寿命で 1 回だけ構築する
/// `HashMap` に置き換える。中身・順序は `CAPTURE_RULES` を単に索引するだけで、
/// `tests::capture_site_metadata_matches_unmodified_golden` が検査する配列自体は変えない。
///
/// site_id は 60 文字前後で先頭 35 文字が共通なので、既定の SipHash では
/// 1 parse あたり 1,383 回の hash が無視できない。索引は同一プロセス内に閉じた
/// 一時的なもの（外部へ漏れず、鍵も持たない）なので、乗算とシフトだけの
/// 安価な hash を使う。
pub(crate) fn rule_for(site_id: &str) -> Option<&'static str> {
    use std::collections::HashMap;
    use std::hash::BuildHasherDefault;
    use std::sync::OnceLock;
    static INDEX: OnceLock<HashMap<&'static str, &'static str, BuildHasherDefault<SiteHasher>>> =
        OnceLock::new();
    INDEX
        .get_or_init(|| CAPTURE_RULES.iter().copied().collect())
        .get(site_id)
        .copied()
}

/// site_id 専用の安価な hash（FxHash と同じ乗算・回転の繰り返し）。
#[derive(Default)]
pub(crate) struct SiteHasher(u64);
impl std::hash::Hasher for SiteHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;
        let mut rest = bytes;
        while let Some((chunk, next)) = rest.split_at_checked(8) {
            let word = u64::from_le_bytes(chunk.try_into().unwrap());
            self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(SEED);
            rest = next;
        }
        for &byte in rest {
            self.0 = (self.0.rotate_left(5) ^ u64::from(byte)).wrapping_mul(SEED);
        }
    }
}
