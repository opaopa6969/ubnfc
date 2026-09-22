use std::{collections::BTreeSet, fmt::Write};
use ubnfc_front::ir::GrammarIr;

pub(crate) fn generate(ir: &GrammarIr) -> String {
    let mut s = String::from("//! Grammar metadata; offsets in catalog sites refer to capture IDs, not input positions.\n#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub struct Catalog {pub rule_id:&'static str,pub rule:&'static str,pub context:&'static str,pub captures:&'static [&'static str],pub site_ids:&'static [&'static str]}\npub const CATALOGS:&[Catalog]=&[\n");
    for rule in &ir.semantics.rules {
        let mut seen = BTreeSet::new();
        for catalog in ir
            .semantics
            .catalogs
            .iter()
            .filter(|c| c.rule_id == rule.symbol_id && seen.insert(c.context.as_str()))
        {
            let rows: Vec<_> = ir
                .semantics
                .catalogs
                .iter()
                .filter(|c| c.rule_id == rule.symbol_id && c.context == catalog.context)
                .collect();
            let sites: Vec<_> = rows.iter().map(|c| c.capture_site_id.as_str()).collect();
            let names: BTreeSet<_> = ir
                .semantics
                .captures
                .iter()
                .filter(|c| sites.contains(&c.site_id.as_str()))
                .map(|c| c.name.as_str())
                .collect();
            let name = ir
                .semantics
                .symbols
                .iter()
                .find(|s| s.id == rule.symbol_id)
                .unwrap()
                .name
                .as_str();
            writeln!(s,"Catalog{{rule_id:{:?},rule:{name:?},context:{:?},captures:&{:?},site_ids:&{sites:?}}},",rule.symbol_id,catalog.context,names.into_iter().collect::<Vec<_>>()).unwrap();
        }
    }
    s.push_str("];\n");
    s
}
