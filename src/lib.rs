//! # Baromètre des prix de création de site web en France 2026
//!
//! This crate embeds the open dataset behind the [Baromètre des prix de
//! création de site web en France 2026](https://lescreavores.fr/prix-creation-site-internet/),
//! a survey of real-world website-creation prices collected by [Les
//! Créavores](https://lescreavores.fr/) across French freelancers, agencies
//! and SaaS website builders.
//!
//! The dataset contains 103 individually sourced price points covering six
//! service categories (`vitrine`, `ecommerce`, `blog`, `refonte`,
//! `seo_mensuel`, `sur_mesure`), three provider types (`freelance`,
//! `agence_guide`, `builder_saas`) and three pricing models
//! (`forfait_unique`, `abonnement_mensuel`, `tjm`). Every record carries its
//! own source (a public URL) and the date it was recorded, so the data can
//! be audited and re-verified.
//!
//! This crate has **zero external dependencies**: the CSV is parsed by hand
//! at runtime from a string embedded at compile time via [`include_str!`],
//! which keeps the build fully reproducible on docs.rs and everywhere else.
//!
//! ## Methodology
//!
//! Each row is an individually sourced price observation, not an estimate:
//! `source_url` points at the public page the price was read from, and
//! `date_releve` records when it was collected. Prices are stored in euros
//! (EUR). Where a provider publishes a price range, `prix_min_eur` and
//! `prix_max_eur` capture the range and `prix_typique_eur` is the price used
//! for aggregate statistics.
//!
//! ## License
//!
//! The **code** of this crate is licensed under MIT (see `LICENSE`).
//!
//! The **dataset** itself (`data/barometre-prix-creation-site-web-france-2026.csv`)
//! is licensed separately under **CC-BY 4.0**. If you redistribute or build
//! upon the data, please credit:
//!
//! > Data: Les Créavores — Baromètre des prix de création de site web en
//! > France 2026 (CC-BY 4.0). Source:
//! > <https://lescreavores.fr/prix-creation-site-internet/>
//!
//! ## Example
//!
//! ```rust
//! let records = barometre_prix_web_fr::all();
//! assert!(records.len() > 90);
//!
//! let vitrine_stats = barometre_prix_web_fr::stats(&barometre_prix_web_fr::by_category(
//!     barometre_prix_web_fr::category::VITRINE,
//! ));
//! if let Some(stats) = vitrine_stats {
//!     println!(
//!         "vitrine: {} records, typical price from {:.2} to {:.2} EUR (median {:.2})",
//!         stats.count, stats.min, stats.max, stats.median
//!     );
//! }
//! ```

/// The raw dataset, embedded at compile time.
///
/// Columns (in order): `categorie_prestation`, `type_prestataire`,
/// `pricing_model`, `prix_min_eur`, `prix_max_eur`, `prix_typique_eur`,
/// `region`, `source_categorie`, `source_url`, `date_releve`, `notes`.
const CSV_DATA: &str = include_str!("../data/barometre-prix-creation-site-web-france-2026.csv");

/// Known `categorie_prestation` codes found in the dataset.
pub mod category {
    /// Showcase / brochure website (`vitrine`).
    pub const VITRINE: &str = "vitrine";
    /// Online store (`ecommerce`).
    pub const ECOMMERCE: &str = "ecommerce";
    /// Blog / editorial website (`blog`).
    pub const BLOG: &str = "blog";
    /// Redesign of an existing website (`refonte`).
    pub const REFONTE: &str = "refonte";
    /// Recurring monthly SEO service (`seo_mensuel`).
    pub const SEO_MENSUEL: &str = "seo_mensuel";
    /// Fully custom / bespoke development (`sur_mesure`).
    pub const SUR_MESURE: &str = "sur_mesure";
}

/// Known `type_prestataire` codes found in the dataset.
pub mod provider_type {
    /// Independent freelance developer or designer.
    pub const FREELANCE: &str = "freelance";
    /// Web agency, priced from a published rate card / guide.
    pub const AGENCE_GUIDE: &str = "agence_guide";
    /// SaaS website builder (Wix, Webflow, Shopify, etc.).
    pub const BUILDER_SAAS: &str = "builder_saas";
}

/// A single price observation from the Baromètre dataset.
///
/// All monetary fields are expressed in euros (EUR). Text fields that are a
/// single CSV token (no embedded commas in the source data) are kept as
/// zero-copy `&'static str` slices into the embedded dataset; the two
/// free-text fields that can legitimately contain commas (`source_label`
/// and `notes`) are reconstructed into owned `String`s.
#[derive(Debug, Clone, PartialEq)]
pub struct PriceRecord {
    /// Service category, e.g. `"vitrine"`, `"ecommerce"`, `"blog"`.
    pub category: &'static str,
    /// Provider type, e.g. `"freelance"`, `"agence_guide"`, `"builder_saas"`.
    pub provider_type: &'static str,
    /// Pricing model, e.g. `"forfait_unique"`, `"abonnement_mensuel"`, `"tjm"`.
    pub pricing_model: &'static str,
    /// Minimum observed price, in EUR. `0.0` if missing or unparsable.
    pub price_min_eur: f64,
    /// Maximum observed price, in EUR. `0.0` if missing or unparsable.
    pub price_max_eur: f64,
    /// Typical (representative) price used for aggregate statistics, in EUR.
    /// `0.0` if missing or unparsable.
    pub typical_price_eur: f64,
    /// Region the price applies to (e.g. `"France"`, `"Paris"`, `"Lyon"`).
    pub region: &'static str,
    /// Human-readable label for the source (offer name, plan name, etc.).
    pub source_label: String,
    /// URL of the public source the price was read from.
    pub source_url: &'static str,
    /// Date the price was recorded, in `YYYY-MM-DD` format.
    pub date_recorded: &'static str,
    /// Free-text notes about the observation (scope, caveats, inclusions).
    pub notes: String,
}

/// Aggregate price statistics computed over a set of [`PriceRecord`]s.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    /// Number of records the statistics were computed over.
    pub count: usize,
    /// Minimum typical price, in EUR.
    pub min: f64,
    /// Maximum typical price, in EUR.
    pub max: f64,
    /// Arithmetic mean of the typical price, in EUR.
    pub mean: f64,
    /// Median of the typical price, in EUR.
    pub median: f64,
}

/// Parses a single embedded CSV line into a [`PriceRecord`].
///
/// Most fields are simple, comma-free tokens and can be sliced directly out
/// of the 11 `split(',')` parts. Two fields (`source_categorie` and
/// `notes`) are free text and occasionally contain unescaped commas in the
/// source data (e.g. `"Webflow Basic site plan (annuel, en USD converti)"`).
/// Since `source_url` always starts with `http` and always immediately
/// precedes `date_releve`, we locate it by scanning instead of assuming a
/// fixed column index — this makes the parser robust to those embedded
/// commas without requiring a full CSV-quoting implementation.
///
/// Returns `None` for blank lines or lines that don't have enough fields to
/// be a valid record. This function never panics on malformed input: it is
/// used purely as an internal, best-effort parser over external data.
fn parse_row(line: &'static str) -> Option<PriceRecord> {
    if line.trim().is_empty() {
        return None;
    }

    let parts: Vec<&'static str> = line.split(',').collect();
    if parts.len() < 11 {
        return None;
    }

    let category = parts[0];
    let provider_type = parts[1];
    let pricing_model = parts[2];
    let price_min_eur = parts[3].trim().parse::<f64>().unwrap_or(0.0);
    let price_max_eur = parts[4].trim().parse::<f64>().unwrap_or(0.0);
    let typical_price_eur = parts[5].trim().parse::<f64>().unwrap_or(0.0);
    let region = parts[6];

    let url_idx = parts
        .iter()
        .enumerate()
        .skip(7)
        .find(|(_, p)| p.starts_with("http"))
        .map(|(i, _)| i)?;

    if url_idx + 1 >= parts.len() {
        return None;
    }

    let source_label = parts[7..url_idx].join(",");
    let source_url = parts[url_idx];
    let date_recorded = parts[url_idx + 1];
    let notes_start = (url_idx + 2).min(parts.len());
    let notes = parts[notes_start..].join(",");

    Some(PriceRecord {
        category,
        provider_type,
        pricing_model,
        price_min_eur,
        price_max_eur,
        typical_price_eur,
        region,
        source_label,
        source_url,
        date_recorded,
        notes,
    })
}

/// Returns every price record in the Baromètre dataset.
///
/// # Examples
///
/// ```rust
/// let records = barometre_prix_web_fr::all();
/// assert!(records.len() > 90);
/// ```
pub fn all() -> Vec<PriceRecord> {
    CSV_DATA.lines().skip(1).filter_map(parse_row).collect()
}

/// Returns all records matching the given service category (e.g.
/// `"vitrine"`, `"ecommerce"`; see the [`category`] module for known
/// values).
///
/// # Examples
///
/// ```rust
/// let vitrine = barometre_prix_web_fr::by_category(barometre_prix_web_fr::category::VITRINE);
/// assert!(!vitrine.is_empty());
/// for record in &vitrine {
///     assert_eq!(record.category, "vitrine");
/// }
/// ```
pub fn by_category(category: &str) -> Vec<PriceRecord> {
    all().into_iter().filter(|r| r.category == category).collect()
}

/// Returns all records matching the given provider type (e.g.
/// `"freelance"`, `"agence_guide"`, `"builder_saas"`; see the
/// [`provider_type`] module for known values).
///
/// # Examples
///
/// ```rust
/// let freelance = barometre_prix_web_fr::by_provider_type(
///     barometre_prix_web_fr::provider_type::FREELANCE,
/// );
/// assert!(!freelance.is_empty());
/// for record in &freelance {
///     assert_eq!(record.provider_type, "freelance");
/// }
/// ```
pub fn by_provider_type(provider_type: &str) -> Vec<PriceRecord> {
    all()
        .into_iter()
        .filter(|r| r.provider_type == provider_type)
        .collect()
}

/// Computes min / max / mean / median statistics over the *typical* price
/// (`typical_price_eur`) of the given records.
///
/// Returns `None` if `records` is empty, so callers never need to guess a
/// sentinel value for an empty dataset slice.
///
/// # Examples
///
/// ```rust
/// let records = barometre_prix_web_fr::all();
/// match barometre_prix_web_fr::stats(&records) {
///     Some(stats) => {
///         assert!(stats.min <= stats.median);
///         assert!(stats.median <= stats.max);
///         assert_eq!(stats.count, records.len());
///     }
///     None => panic!("dataset should not be empty"),
/// }
///
/// // An empty slice yields no statistics.
/// assert!(barometre_prix_web_fr::stats(&[]).is_none());
/// ```
pub fn stats(records: &[PriceRecord]) -> Option<Stats> {
    if records.is_empty() {
        return None;
    }

    let mut prices: Vec<f64> = records.iter().map(|r| r.typical_price_eur).collect();
    prices.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let count = prices.len();
    let min = prices[0];
    let max = prices[count - 1];
    let mean = prices.iter().sum::<f64>() / count as f64;
    let median = if count % 2 == 0 {
        (prices[count / 2 - 1] + prices[count / 2]) / 2.0
    } else {
        prices[count / 2]
    };

    Some(Stats {
        count,
        min,
        max,
        mean,
        median,
    })
}

/// Returns the distinct list of service categories present in the dataset,
/// sorted alphabetically.
///
/// # Examples
///
/// ```rust
/// let categories = barometre_prix_web_fr::categories();
/// assert!(categories.contains(&"vitrine"));
/// assert!(categories.contains(&"ecommerce"));
/// ```
pub fn categories() -> Vec<&'static str> {
    let mut values: Vec<&'static str> = all().iter().map(|r| r.category).collect();
    values.sort_unstable();
    values.dedup();
    values
}

/// Returns the distinct list of provider types present in the dataset,
/// sorted alphabetically.
///
/// # Examples
///
/// ```rust
/// let providers = barometre_prix_web_fr::provider_types();
/// assert!(providers.contains(&"freelance"));
/// ```
pub fn provider_types() -> Vec<&'static str> {
    let mut values: Vec<&'static str> = all().iter().map(|r| r.provider_type).collect();
    values.sort_unstable();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dataset_parses_without_error() {
        let records = all();
        assert!(!records.is_empty(), "the dataset should parse at least one record");
    }

    #[test]
    fn record_count_is_above_ninety() {
        let records = all();
        assert!(
            records.len() > 90,
            "expected more than 90 records, got {}",
            records.len()
        );
    }

    #[test]
    fn stats_are_internally_consistent() {
        let records = all();
        let s = stats(&records).expect("dataset is not empty");
        assert!(s.min <= s.median, "min ({}) should be <= median ({})", s.min, s.median);
        assert!(s.median <= s.max, "median ({}) should be <= max ({})", s.median, s.max);
        assert!(s.min <= s.mean, "min ({}) should be <= mean ({})", s.min, s.mean);
        assert!(s.mean <= s.max, "mean ({}) should be <= max ({})", s.mean, s.max);
        assert_eq!(s.count, records.len());
    }

    #[test]
    fn known_category_returns_results() {
        let vitrine = by_category(category::VITRINE);
        assert!(!vitrine.is_empty());
        for record in &vitrine {
            assert_eq!(record.category, "vitrine");
        }
    }

    #[test]
    fn known_provider_type_returns_results() {
        let builders = by_provider_type(provider_type::BUILDER_SAAS);
        assert!(!builders.is_empty());
        for record in &builders {
            assert_eq!(record.provider_type, "builder_saas");
        }
    }

    #[test]
    fn unknown_category_returns_empty() {
        let none = by_category("does_not_exist");
        assert!(none.is_empty());
    }

    #[test]
    fn categories_and_provider_types_are_deduplicated_and_sorted() {
        let cats = categories();
        let mut sorted = cats.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(cats, sorted, "categories() should be sorted and deduplicated");

        let providers = provider_types();
        let mut sorted_providers = providers.clone();
        sorted_providers.sort_unstable();
        sorted_providers.dedup();
        assert_eq!(
            providers, sorted_providers,
            "provider_types() should be sorted and deduplicated"
        );
    }

    #[test]
    fn rows_with_embedded_commas_in_notes_still_parse_source_url() {
        // Two rows in the dataset (Webflow plans) have an unescaped comma
        // inside the `source_categorie` field. Make sure those still parse
        // into a valid https URL for source_url rather than swallowing the
        // URL into source_label.
        let records = all();
        let matches: Vec<&PriceRecord> = records
            .iter()
            .filter(|r| r.source_url.starts_with("https://www.agence-synqro.fr"))
            .collect();
        assert!(!matches.is_empty(), "expected at least one Webflow-sourced record");
        for record in matches {
            assert!(record.source_url.starts_with("http"));
            assert_eq!(record.date_recorded.len(), 10, "date should be YYYY-MM-DD");
        }
    }

    #[test]
    fn price_fields_are_never_negative() {
        for record in all() {
            assert!(record.price_min_eur >= 0.0);
            assert!(record.price_max_eur >= 0.0);
            assert!(record.typical_price_eur >= 0.0);
        }
    }
}
