use std::fmt::Write;

use crate::money::Money;
use crate::report::annual_report::AnnualReport;

/// Swedish XBRL taxonomy namespaces for K2.
const SE_GEN_BASE: &str = "http://www.taxonomier.se/se/fr/gaap/gen-base";
const SE_K2_TYPE: &str = "http://www.taxonomier.se/se/fr/k2/datatype";
const SE_CD_TYPE: &str = "http://www.taxonomier.se/se/fr/cd/datatype";

/// Generate an iXBRL (Inline XBRL) document for Bolagsverket filing.
/// The output is valid XHTML with embedded XBRL tags per the Swedish taxonomy.
pub fn generate_ixbrl(report: &AnnualReport) -> String {
    let mut doc = String::with_capacity(32_000);

    write_header(&mut doc, report);
    write_hidden_context(&mut doc, report);
    write_directors_report(&mut doc, report);
    write_income_statement(&mut doc, report);
    write_balance_sheet(&mut doc, report);
    write_notes(&mut doc, report);
    write_footer(&mut doc);

    doc
}

/// Compute SHA-256 checksum of the iXBRL content.
pub fn compute_checksum(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

fn write_header(doc: &mut String, report: &AnnualReport) {
    let org = format_org_number(&report.company.org_number);
    let company = &report.company.name;
    let fy_start = &report.fiscal_year.start_date;
    let fy_end = &report.fiscal_year.end_date;

    write!(
        doc,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml"
      xmlns:ix="http://www.xbrl.org/2013/inlineXBRL"
      xmlns:ixt="http://www.xbrl.org/inlineXBRL/transformation/2020-02-12"
      xmlns:xbrli="http://www.xbrl.org/2003/instance"
      xmlns:link="http://www.xbrl.org/2003/linkbase"
      xmlns:xlink="http://www.w3.org/1999/xlink"
      xmlns:iso4217="http://www.xbrl.org/2003/iso4217"
      xmlns:se-gen-base="{SE_GEN_BASE}"
      xmlns:se-k2-type="{SE_K2_TYPE}"
      xmlns:se-cd-type="{SE_CD_TYPE}"
      xml:lang="sv">
<head>
  <meta charset="UTF-8" />
  <title>Årsredovisning {company} {fy_end}</title>
  <meta name="generator" content="Balans {version}" />
  <style>
    body {{ font-family: 'Liberation Sans', Arial, sans-serif; font-size: 10pt; margin: 2cm; color: #000; }}
    h1 {{ font-size: 16pt; text-align: center; margin-bottom: 4pt; }}
    h2 {{ font-size: 13pt; margin-top: 18pt; border-bottom: 1px solid #000; padding-bottom: 3pt; }}
    h3 {{ font-size: 11pt; margin-top: 12pt; }}
    .header {{ text-align: center; margin-bottom: 24pt; }}
    .header p {{ margin: 2pt 0; }}
    .fin-table {{ width: 100%; border-collapse: collapse; margin: 8pt 0; }}
    .fin-table th, .fin-table td {{ padding: 2pt 6pt; text-align: right; font-size: 9pt; }}
    .fin-table th {{ text-align: left; font-weight: normal; }}
    .fin-table .label {{ text-align: left; }}
    .fin-table .total {{ font-weight: bold; border-top: 1px solid #000; }}
    .fin-table .subtotal {{ font-weight: bold; }}
    .fin-table .section {{ font-weight: bold; text-align: left; padding-top: 8pt; }}
    .note {{ margin-top: 12pt; }}
    .note-title {{ font-weight: bold; }}
    .hidden {{ display: none; }}
  </style>
</head>
<body>
<ix:header>
  <ix:hidden>
    <ix:nonNumeric name="se-gen-base:Foretagsnamn" contextRef="current">{company}</ix:nonNumeric>
    <ix:nonNumeric name="se-gen-base:Organisationsnummer" contextRef="current">{org}</ix:nonNumeric>
  </ix:hidden>
  <ix:resources>
    <xbrli:context id="current">
      <xbrli:entity>
        <xbrli:identifier scheme="http://www.bolagsverket.se">{org_raw}</xbrli:identifier>
      </xbrli:entity>
      <xbrli:period>
        <xbrli:startDate>{fy_start}</xbrli:startDate>
        <xbrli:endDate>{fy_end}</xbrli:endDate>
      </xbrli:period>
    </xbrli:context>
    <xbrli:context id="current_instant">
      <xbrli:entity>
        <xbrli:identifier scheme="http://www.bolagsverket.se">{org_raw}</xbrli:identifier>
      </xbrli:entity>
      <xbrli:period>
        <xbrli:instant>{fy_end}</xbrli:instant>
      </xbrli:period>
    </xbrli:context>"#,
        version = env!("CARGO_PKG_VERSION"),
        org_raw = report.company.org_number,
    )
    .unwrap();

    // Add previous year context if available
    if let Some(ref prev) = report.income_statement.previous {
        let prev_label = &prev.fiscal_year;
        // Parse dates from the label "YYYY-MM-DD — YYYY-MM-DD"
        let parts: Vec<&str> = prev_label.split(" — ").collect();
        if parts.len() == 2 {
            write!(
                doc,
                r#"
    <xbrli:context id="previous">
      <xbrli:entity>
        <xbrli:identifier scheme="http://www.bolagsverket.se">{org_raw}</xbrli:identifier>
      </xbrli:entity>
      <xbrli:period>
        <xbrli:startDate>{start}</xbrli:startDate>
        <xbrli:endDate>{end}</xbrli:endDate>
      </xbrli:period>
    </xbrli:context>"#,
                org_raw = report.company.org_number,
                start = parts[0],
                end = parts[1],
            )
            .unwrap();
        }
    }

    write!(
        doc,
        r#"
    <xbrli:unit id="SEK">
      <xbrli:measure>iso4217:SEK</xbrli:measure>
    </xbrli:unit>
  </ix:resources>
</ix:header>
"#
    )
    .unwrap();
}

fn write_hidden_context(doc: &mut String, _report: &AnnualReport) {
    // Metadata section — hidden XBRL facts
    write!(
        doc,
        r#"
<div class="hidden">
  <ix:nonNumeric name="se-gen-base:RedovisningsramverkK2" contextRef="current">true</ix:nonNumeric>
</div>
"#
    )
    .unwrap();
}

fn write_directors_report(doc: &mut String, report: &AnnualReport) {
    let company = &report.company.name;
    let org = format_org_number(&report.company.org_number);
    let dr = &report.directors_report;

    write!(
        doc,
        r#"
<div class="header">
  <h1>ÅRSREDOVISNING</h1>
  <p><strong>{company}</strong></p>
  <p>Org.nr: {org}</p>
  <p>Räkenskapsår: {start} — {end}</p>
</div>

<h2>Förvaltningsberättelse</h2>

<h3>Verksamheten</h3>
<p><ix:nonNumeric name="se-gen-base:VerksamhetenArtInriktning" contextRef="current">{business}</ix:nonNumeric></p>

<h3>Väsentliga händelser under räkenskapsåret</h3>
<p><ix:nonNumeric name="se-gen-base:ViktigaHandelserRakenskapsar" contextRef="current">{events}</ix:nonNumeric></p>

<h3>Förväntad framtida utveckling</h3>
<p><ix:nonNumeric name="se-gen-base:ForvantadFramtidaUtveckling" contextRef="current">{future}</ix:nonNumeric></p>
"#,
        start = report.fiscal_year.start_date,
        end = report.fiscal_year.end_date,
        business = escape_xml(&dr.business_description),
        events = escape_xml(&dr.important_events),
        future = escape_xml(&dr.future_outlook),
    )
    .unwrap();

    if let Some(ref pa) = dr.profit_allocation {
        write!(
            doc,
            r#"
<h3>Förslag till vinstdisposition</h3>
<p>Styrelsen föreslår att till förfogande stående medel:</p>
<table class="fin-table">
  <tr><th class="label">Årets resultat</th><td>{result}</td></tr>
  <tr><th class="label">Balanserat resultat</th><td>{retained}</td></tr>
  <tr class="total"><th class="label">Summa</th><td>{total}</td></tr>
</table>
<p>Disponeras enligt följande:</p>
<table class="fin-table">
  <tr><th class="label">I ny räkning överföres</th><td>{carry}</td></tr>
</table>
"#,
            result = fmt_sek(pa.result_for_year),
            retained = fmt_sek(pa.retained_earnings),
            total = fmt_sek(pa.total_available),
            carry = fmt_sek(pa.carry_forward),
        )
        .unwrap();
    }
}

fn write_income_statement(doc: &mut String, report: &AnnualReport) {
    let is = &report.income_statement.current;
    let prev = report.income_statement.previous.as_ref();

    write!(doc, r#"
<h2>Resultaträkning</h2>
<table class="fin-table">
  <thead>
    <tr>
      <th class="label"></th>
      <th>{current_label}</th>"#,
        current_label = is.fiscal_year,
    ).unwrap();

    if let Some(p) = prev {
        write!(doc, r#"
      <th>{}</th>"#, p.fiscal_year).unwrap();
    }

    write!(doc, r#"
    </tr>
  </thead>
  <tbody>"#).unwrap();

    is_row(doc, "Nettoomsättning", "Nettoomsattning", is.net_revenue, prev.map(|p| p.net_revenue), false);
    is_row(doc, "Övriga rörelseintäkter", "OvrigaRorelseintakter", is.other_operating_income, prev.map(|p| p.other_operating_income), false);
    is_row(doc, "Råvaror och förnödenheter", "RavarorFornodenheter", -is.raw_materials, prev.map(|p| -p.raw_materials), false);
    is_row(doc, "Övriga externa kostnader", "OvrigaExternaKostnader", -is.other_external_costs, prev.map(|p| -p.other_external_costs), false);
    is_row(doc, "Personalkostnader", "Personalkostnader", -is.personnel_costs, prev.map(|p| -p.personnel_costs), false);
    is_row(doc, "Av- och nedskrivningar", "AvNedskrivningarMateriellaImmateriellaAnlaggningstillgangar", -is.depreciation, prev.map(|p| -p.depreciation), false);
    is_row(doc, "Rörelseresultat", "Rorelseresultat", is.operating_result, prev.map(|p| p.operating_result), true);
    is_row(doc, "Finansiella intäkter", "OvrigaRanteintakterLiknandeResultatposter", is.financial_income, prev.map(|p| p.financial_income), false);
    is_row(doc, "Finansiella kostnader", "RantekostnaderLiknandeResultatposter", -is.financial_costs, prev.map(|p| -p.financial_costs), false);
    is_row(doc, "Resultat efter finansiella poster", "ResultatEfterFinansiellaPoster", is.result_after_financial, prev.map(|p| p.result_after_financial), true);
    is_row(doc, "Skatt på årets resultat", "SkattAretsResultat", -is.tax, prev.map(|p| -p.tax), false);
    is_row(doc, "Årets resultat", "AretsResultat", is.net_result, prev.map(|p| p.net_result), true);

    write!(doc, r#"
  </tbody>
</table>
"#).unwrap();
}

fn write_balance_sheet(doc: &mut String, report: &AnnualReport) {
    let bs = &report.balance_sheet.current;
    let prev = report.balance_sheet.previous.as_ref();

    write!(doc, r#"
<h2>Balansräkning</h2>
<table class="fin-table">
  <thead>
    <tr>
      <th class="label"></th>
      <th>{current_label}</th>"#,
        current_label = bs.fiscal_year,
    ).unwrap();

    if let Some(p) = prev {
        write!(doc, r#"
      <th>{}</th>"#, p.fiscal_year).unwrap();
    }

    write!(doc, r#"
    </tr>
  </thead>
  <tbody>
    <tr><td class="section" colspan="3">TILLGÅNGAR</td></tr>
    <tr><td class="section" colspan="3">Anläggningstillgångar</td></tr>"#).unwrap();

    bs_row(doc, "Immateriella anläggningstillgångar", "ImmateriellaTillgangar", bs.assets.intangible_assets, prev.map(|p| p.assets.intangible_assets), false);
    bs_row(doc, "Materiella anläggningstillgångar", "MateriellaTillgangar", bs.assets.tangible_assets, prev.map(|p| p.assets.tangible_assets), false);
    bs_row(doc, "Finansiella anläggningstillgångar", "FinansiellaAnlaggningstillgangar", bs.assets.financial_fixed_assets, prev.map(|p| p.assets.financial_fixed_assets), false);
    bs_row(doc, "Summa anläggningstillgångar", "SummaAnlaggningstillgangar", bs.assets.total_fixed_assets, prev.map(|p| p.assets.total_fixed_assets), true);

    write!(doc, r#"
    <tr><td class="section" colspan="3">Omsättningstillgångar</td></tr>"#).unwrap();

    bs_row(doc, "Varulager", "Varulager", bs.assets.inventory, prev.map(|p| p.assets.inventory), false);
    bs_row(doc, "Kortfristiga fordringar", "KortfristigaFordringar", bs.assets.current_receivables, prev.map(|p| p.assets.current_receivables), false);
    bs_row(doc, "Kassa och bank", "KassaBank", bs.assets.cash_and_bank, prev.map(|p| p.assets.cash_and_bank), false);
    bs_row(doc, "Summa omsättningstillgångar", "SummaOmsattningstillgangar", bs.assets.total_current_assets, prev.map(|p| p.assets.total_current_assets), true);
    bs_row(doc, "SUMMA TILLGÅNGAR", "SummaTillgangar", bs.total_assets, prev.map(|p| p.total_assets), true);

    write!(doc, r#"
    <tr><td class="section" colspan="3">EGET KAPITAL OCH SKULDER</td></tr>
    <tr><td class="section" colspan="3">Eget kapital</td></tr>"#).unwrap();

    bs_row(doc, "Bundet eget kapital", "BundetEgetKapital", bs.equity_and_liabilities.restricted_equity, prev.map(|p| p.equity_and_liabilities.restricted_equity), false);
    bs_row(doc, "Fritt eget kapital", "FrittEgetKapital", bs.equity_and_liabilities.unrestricted_equity, prev.map(|p| p.equity_and_liabilities.unrestricted_equity), false);
    bs_row(doc, "Summa eget kapital", "SummaEgetKapital", bs.equity_and_liabilities.total_equity, prev.map(|p| p.equity_and_liabilities.total_equity), true);
    bs_row(doc, "Kortfristiga skulder", "SummaKortfristigaSkulder", bs.equity_and_liabilities.current_liabilities, prev.map(|p| p.equity_and_liabilities.current_liabilities), false);
    bs_row(doc, "SUMMA EGET KAPITAL OCH SKULDER", "SummaEgetKapitalSkulder", bs.total_equity_and_liabilities, prev.map(|p| p.total_equity_and_liabilities), true);

    write!(doc, r#"
  </tbody>
</table>
"#).unwrap();
}

fn write_notes(doc: &mut String, report: &AnnualReport) {
    write!(doc, r#"
<h2>Noter</h2>
"#).unwrap();

    for note in &report.notes.items {
        write!(
            doc,
            r#"
<div class="note">
  <p class="note-title">Not {num}: {title}</p>
  <p>{content}</p>
</div>
"#,
            num = note.number,
            title = escape_xml(&note.title),
            content = escape_xml(&note.content).replace('\n', "<br/>"),
        )
        .unwrap();
    }
}

fn write_footer(doc: &mut String) {
    write!(
        doc,
        r#"
</body>
</html>"#
    )
    .unwrap();
}

/// Write an income statement row with XBRL tagging.
fn is_row(doc: &mut String, label: &str, xbrl_name: &str, current: Money, previous: Option<Money>, is_total: bool) {
    let class = if is_total { "subtotal" } else { "" };

    write!(doc, r#"
    <tr class="{class}">
      <th class="label">{label}</th>
      <td><ix:nonFraction name="se-gen-base:{xbrl_name}" contextRef="current" unitRef="SEK" decimals="0" format="ixt:num-dot-decimal">{formatted}</ix:nonFraction></td>"#,
        formatted = fmt_sek(current),
    ).unwrap();

    if let Some(prev) = previous {
        write!(doc, r#"
      <td>{prev_fmt}</td>"#,
            prev_fmt = fmt_sek(prev),
        ).unwrap();
    }

    write!(doc, r#"
    </tr>"#).unwrap();
}

/// Write a balance sheet row with XBRL tagging.
fn bs_row(doc: &mut String, label: &str, xbrl_name: &str, current: Money, previous: Option<Money>, is_total: bool) {
    let class = if is_total { "subtotal" } else { "" };

    write!(doc, r#"
    <tr class="{class}">
      <th class="label">{label}</th>
      <td><ix:nonFraction name="se-gen-base:{xbrl_name}" contextRef="current_instant" unitRef="SEK" decimals="0" format="ixt:num-dot-decimal">{formatted}</ix:nonFraction></td>"#,
        formatted = fmt_sek(current),
    ).unwrap();

    if let Some(prev) = previous {
        write!(doc, r#"
      <td>{prev_fmt}</td>"#,
            prev_fmt = fmt_sek(prev),
        ).unwrap();
    }

    write!(doc, r#"
    </tr>"#).unwrap();
}

fn fmt_sek(m: Money) -> String {
    let ore = m.to_ore();
    if ore == 0 {
        return "-".to_string();
    }
    // Format as SEK with thousands separator
    let sek = ore / 100;
    let negative = sek < 0;
    let abs_sek = sek.unsigned_abs();
    let s = abs_sek.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ' ');
        }
        result.insert(0, c);
    }
    if negative {
        format!("-{result}")
    } else {
        result
    }
}

fn format_org_number(org: &str) -> String {
    if org.len() == 10 && !org.contains('-') {
        format!("{}-{}", &org[..6], &org[6..])
    } else {
        org.to_string()
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_sek() {
        // from_ore(15000) = 150.00 SEK → 150 SEK → "150"
        assert_eq!(fmt_sek(Money::from_ore(15000)), "150");
        // from_ore(10000000) = 100000.00 SEK → "100 000"
        assert_eq!(fmt_sek(Money::from_ore(10000000)), "100 000");
        assert_eq!(fmt_sek(Money::ZERO), "-");
        assert_eq!(fmt_sek(Money::from_ore(-500000)), "-5 000");
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
    }

    #[test]
    fn test_checksum() {
        let content = "test content";
        let hash = compute_checksum(content);
        assert_eq!(hash.len(), 64); // SHA-256 hex = 64 chars
    }
}
