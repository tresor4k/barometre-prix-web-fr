# barometre-prix-web-fr

[![Crates.io](https://img.shields.io/crates/v/barometre-prix-web-fr.svg)](https://crates.io/crates/barometre-prix-web-fr)
[![Docs.rs](https://docs.rs/barometre-prix-web-fr/badge.svg)](https://docs.rs/barometre-prix-web-fr)

An open dataset of **French website-creation prices (2026)**, embedded as a
Rust crate with zero external dependencies.

This crate ships the raw CSV data behind the [Baromètre des prix de création
de site web en France 2026](https://lescreavores.fr/prix-creation-site-internet/)
published by **Les Créavores**, plus a small, dependency-free API to load,
filter and summarize it.

- **103 individually sourced price observations**
- 6 service categories: `vitrine` (showcase site), `ecommerce`, `blog`,
  `refonte` (redesign), `seo_mensuel` (recurring SEO), `sur_mesure` (bespoke)
- 3 provider types: `freelance`, `agence_guide` (agency rate cards),
  `builder_saas` (Wix, Webflow, Shopify, ...)
- 3 pricing models: `forfait_unique` (flat fee), `abonnement_mensuel`
  (monthly subscription), `tjm` (daily rate)
- Every record carries its own public `source_url` and `date_releve`
  (collection date) for auditability
- **Zero dependencies** — the CSV is parsed by hand, so the crate builds
  reliably everywhere, including on docs.rs

## Usage

Add the crate:

```toml
[dependencies]
barometre-prix-web-fr = "1.0"
```

Query the dataset:

```rust
use barometre_prix_web_fr::{all, by_category, category, stats};

fn main() {
    let records = all();
    println!("{} price observations loaded", records.len());

    let vitrine = by_category(category::VITRINE);
    if let Some(s) = stats(&vitrine) {
        println!(
            "Showcase websites: {} to {} EUR (median {:.0} EUR, n={})",
            s.min, s.max, s.median, s.count
        );
    }
}
```

See the [full API documentation on docs.rs](https://docs.rs/barometre-prix-web-fr)
for `by_provider_type`, `categories`, `provider_types` and the `PriceRecord`
/ `Stats` types.

## Data source & license

- **Code** (this crate): [MIT license](LICENSE).
- **Data** (`data/barometre-prix-creation-site-web-france-2026.csv`):
  **CC-BY 4.0**, published by [Les Créavores](https://lescreavores.fr/) as
  part of the [Baromètre des prix de création de site web en France
  2026](https://lescreavores.fr/prix-creation-site-internet/).

If you reuse or redistribute the data, please credit:

> Data: Les Créavores — Baromètre des prix de création de site web en France
> 2026 (CC-BY 4.0). Source: <https://lescreavores.fr/prix-creation-site-internet/>

The dataset is a work in progress and reflects prices publicly available at
the time each row was collected (see the `date_releve` column). It is
provided for research and informational purposes; it is not a quote and
actual prices vary by provider, scope and negotiation.
