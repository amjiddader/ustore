use crate::registry::models::{Package, SourceRegistry};

pub fn search_packages<'a>(registry: &'a SourceRegistry, query: &str) -> Vec<&'a Package> {
    let query = query.to_lowercase();
    registry
        .packages
        .iter()
        .filter(|pkg| {
            pkg.name.to_lowercase().contains(&query)
                || pkg.id.to_lowercase().contains(&query)
                || pkg.description.to_lowercase().contains(&query)
                || pkg.category.to_lowercase().contains(&query)
                || pkg.tags.iter().any(|t| t.to_lowercase().contains(&query))
                || pkg.alt_names.as_ref().is_some_and(|alts| {
                    alts.iter().any(|a| a.to_lowercase().contains(&query))
                })
        })
        .collect()
}

#[allow(dead_code)]
pub fn filter_by_category<'a>(registry: &'a SourceRegistry, category: &str) -> Vec<&'a Package> {
    let category = category.to_lowercase();
    registry
        .packages
        .iter()
        .filter(|pkg| pkg.category.to_lowercase() == category)
        .collect()
}

pub fn find_package<'a>(registry: &'a SourceRegistry, id: &str) -> Option<&'a Package> {
    registry.packages.iter().find(|pkg| pkg.id == id)
}
