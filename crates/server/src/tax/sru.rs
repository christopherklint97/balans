use std::fmt::Write;

use super::ink2::Ink2Data;

/// Generate an SRU file for Skatteverket.
/// SRU format is a simple text format with key-value pairs.
///
/// Format:
/// ```
/// #DATABESKRIVNING_START
/// #PRODUKT SRU
/// #SESSION <session_id>
/// #FLAGGA 0
/// #PROGRAM Balans <version>
/// #MEDESSION <session_id>
/// #DATABESKRIVNING_SLUT
/// #MEDGIVANDE_START
/// #ORGNR <org_number>
/// #MEDESSION <session_id>
/// #MEDGIVANDE_SLUT
/// #BLANKETTSTART
/// #IDENTITET <org_number> <date>
/// #UPPGIFT <sru_code> <value>
/// ...
/// #BLANKETTSLUT
/// ```
pub fn generate_sru(data: &Ink2Data) -> String {
    let mut out = String::with_capacity(4_000);
    let org = &data.org_number;
    let date = data.fiscal_year_end.replace('-', "");

    // Data description block
    writeln!(out, "#DATABESKRIVNING_START").unwrap();
    writeln!(out, "#PRODUKT SRU").unwrap();
    writeln!(out, "#SKATEFIL {date}").unwrap();
    writeln!(out, "#PROGRAM Balans {}", env!("CARGO_PKG_VERSION")).unwrap();
    writeln!(out, "#FILNAMN BLANKETTER.SRU").unwrap();
    writeln!(out, "#DATABESKRIVNING_SLUT").unwrap();

    // Consent block
    writeln!(out, "#MEDGIVANDE_START").unwrap();
    writeln!(out, "#ORGNR {org}").unwrap();
    writeln!(out, "#MEDGIVANDE_SLUT").unwrap();

    // INK2R — Resultat- och balansräkning
    writeln!(out, "#BLANKETTSTART").unwrap();
    writeln!(out, "#IDENTITET {org} {date}").unwrap();
    writeln!(out, "#BLANESSION INK2R").unwrap();

    // Company info fields
    writeln!(out, "#UPPGIFT 7011 {}", data.fiscal_year_start.replace('-', "")).unwrap();
    writeln!(out, "#UPPGIFT 7012 {date}").unwrap();
    writeln!(out, "#UPPGIFT 7014 {}", data.company_name).unwrap();

    // Financial data fields — only output non-zero values
    for field in &data.fields {
        let ore = field.amount.to_ore();
        if ore == 0 {
            continue;
        }
        // SRU values are in whole SEK (kronor), not ören
        let sek = ore / 100;
        writeln!(out, "#UPPGIFT {} {sek}", field.sru_code).unwrap();
    }

    writeln!(out, "#BLANKETTSLUT").unwrap();

    // INK2S — Skattemässiga justeringar (basic)
    writeln!(out, "#BLANKETTSTART").unwrap();
    writeln!(out, "#IDENTITET {org} {date}").unwrap();
    writeln!(out, "#BLANESSION INK2S").unwrap();

    // Bokfört resultat (SRU 7650) — sum of all IS fields
    let revenue: i64 = data
        .fields
        .iter()
        .filter(|f| f.sru_code.starts_with("741") || f.sru_code == "7412")
        .map(|f| f.amount.to_ore())
        .sum();
    let expenses: i64 = data
        .fields
        .iter()
        .filter(|f| {
            let c = f.sru_code.as_str();
            c.starts_with("742") || c.starts_with("743") || c.starts_with("744")
                || c.starts_with("745") || c.starts_with("751")
        })
        .map(|f| f.amount.to_ore())
        .sum();

    let bokfort_resultat = (revenue - expenses) / 100;
    writeln!(out, "#UPPGIFT 7650 {bokfort_resultat}").unwrap();

    writeln!(out, "#BLANKETTSLUT").unwrap();

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::money::Money;
    use crate::tax::ink2::{Ink2Field, Ink2Section};

    #[test]
    fn test_generate_sru_basic() {
        let data = Ink2Data {
            company_name: "Test AB".to_string(),
            org_number: "5561234567".to_string(),
            fiscal_year_start: "2025-01-01".to_string(),
            fiscal_year_end: "2025-12-31".to_string(),
            fields: vec![
                Ink2Field {
                    sru_code: "7410".to_string(),
                    label: "Nettoomsättning".to_string(),
                    amount: Money::from_ore(10_000_00),
                    accounts: vec![3010],
                },
                Ink2Field {
                    sru_code: "7430".to_string(),
                    label: "Övriga externa kostnader".to_string(),
                    amount: Money::from_ore(3_000_00),
                    accounts: vec![5010],
                },
            ],
            sections: vec![],
        };

        let sru = generate_sru(&data);

        assert!(sru.contains("#PRODUKT SRU"));
        assert!(sru.contains("#ORGNR 5561234567"));
        assert!(sru.contains("#BLANESSION INK2R"));
        assert!(sru.contains("#UPPGIFT 7410 100")); // 10000 ören = 100 SEK
        assert!(sru.contains("#UPPGIFT 7430 30"));  // 3000 ören = 30 SEK
        assert!(sru.contains("#BLANESSION INK2S"));
        assert!(sru.contains("#BLANKETTSLUT"));
    }

    #[test]
    fn test_sru_format_structure() {
        let data = Ink2Data {
            company_name: "Empty AB".to_string(),
            org_number: "5561234567".to_string(),
            fiscal_year_start: "2025-01-01".to_string(),
            fiscal_year_end: "2025-12-31".to_string(),
            fields: vec![],
            sections: vec![],
        };

        let sru = generate_sru(&data);

        // Verify required sections exist
        assert!(sru.contains("#DATABESKRIVNING_START"));
        assert!(sru.contains("#DATABESKRIVNING_SLUT"));
        assert!(sru.contains("#MEDGIVANDE_START"));
        assert!(sru.contains("#MEDGIVANDE_SLUT"));
        // Two blankett blocks (INK2R + INK2S)
        assert_eq!(sru.matches("#BLANKETTSTART").count(), 2);
        assert_eq!(sru.matches("#BLANKETTSLUT").count(), 2);
    }
}
