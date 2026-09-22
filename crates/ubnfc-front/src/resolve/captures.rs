use std::collections::BTreeMap;

use super::{CaptureSite, PathStep, ResolvedAtom, ResolvedBody};

pub(super) fn collect(body: &ResolvedBody) -> Vec<CaptureSite> {
    let mut sites = Vec::new();
    body_sites(body, &[], 0, &mut sites);
    let mut counts = BTreeMap::new();
    for site in &sites {
        *counts.entry(site.name.clone()).or_insert(0) += 1;
    }
    for site in &mut sites {
        site.occurrences = counts[&site.name];
    }
    sites
}

fn body_sites(body: &ResolvedBody, path: &[PathStep], depth: usize, sites: &mut Vec<CaptureSite>) {
    for (i, alternative) in body.alternatives.iter().enumerate() {
        for (j, element) in alternative.iter().enumerate() {
            let mut path = path.to_vec();
            path.extend([PathStep::Alternative(i), PathStep::Element(j)]);
            if let Some(name) = &element.capture {
                sites.push(CaptureSite {
                    id: sites.len(),
                    name: name.clone(),
                    span: element.span,
                    path: path.clone(),
                    inside_quantifier: depth > 0,
                    quantifier_depth: depth,
                    quantified: matches!(
                        element.atom,
                        ResolvedAtom::Optional { .. }
                            | ResolvedAtom::Repeat { .. }
                            | ResolvedAtom::OneOrMore { .. }
                            | ResolvedAtom::Bounded { .. }
                            | ResolvedAtom::Separated { .. }
                    ),
                    occurrences: 0,
                });
            }
            atom_sites(&element.atom, &path, depth, sites);
        }
    }
}

fn atom_sites(atom: &ResolvedAtom, path: &[PathStep], depth: usize, sites: &mut Vec<CaptureSite>) {
    use ResolvedAtom as A;
    let mut path = path.to_vec();
    match atom {
        A::Group { body, .. } => {
            path.push(PathStep::Group);
            body_sites(body, &path, depth, sites);
        }
        A::Optional { body, .. } => {
            path.push(PathStep::Optional);
            body_sites(body, &path, depth + 1, sites);
        }
        A::Repeat { body, .. } => {
            path.push(PathStep::Repeat);
            body_sites(body, &path, depth + 1, sites);
        }
        A::OneOrMore { body, .. } => {
            path.push(PathStep::OneOrMore);
            atom_sites(body, &path, depth + 1, sites);
        }
        A::Bounded { body, .. } => {
            path.push(PathStep::Bounded);
            atom_sites(body, &path, depth + 1, sites);
        }
        A::Separated {
            element, separator, ..
        } => {
            path.push(PathStep::SeparatedElement);
            atom_sites(element, &path, depth + 1, sites);
            path.pop();
            path.push(PathStep::Separator);
            atom_sites(separator, &path, depth + 1, sites);
        }
        A::Ref { .. } | A::Literal { .. } | A::Error { .. } => {}
    }
}
