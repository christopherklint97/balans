use genpdf::{
    elements::{Break, LinearLayout, PageBreak, Paragraph, TableLayout},
    fonts,
    style::Style,
    Alignment, Document, Element, Margins,
};

use super::annual_report::AnnualReport;
use crate::money::Money;

const BODY_SIZE: u8 = 10;
const SMALL_SIZE: u8 = 9;
const H1_SIZE: u8 = 18;
const H2_SIZE: u8 = 13;
const H3_SIZE: u8 = 11;

/// Generate a PDF for the annual report (K2 format, BFNAR 2016:10).
pub fn generate_pdf(report: &AnnualReport) -> Result<Vec<u8>, String> {
    let font = fonts::from_files("./fonts", "LiberationSans", None)
        .or_else(|_| fonts::from_files("/usr/share/fonts/truetype/liberation", "LiberationSans", None))
        .or_else(|_| fonts::from_files("/usr/share/fonts/liberation-sans", "LiberationSans", None))
        .or_else(|_| fonts::from_files("/usr/share/fonts", "DejaVuSans", None))
        .map_err(|e| format!("No suitable font found. Install fonts-liberation or fonts-dejavu: {e}"))?;

    let mut doc = Document::new(font);
    doc.set_title(format!(
        "Årsredovisning {} {}",
        report.company.name, report.fiscal_year.end_date
    ));
    doc.set_minimal_conformance();
    doc.set_line_spacing(1.2);
    doc.set_font_size(BODY_SIZE);

    // A4 defaults to 210×297mm. Standard Swedish business-document margins:
    // 25mm top/bottom, 25mm left, 20mm right.
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(Margins::trbl(25, 20, 25, 25));
    let company_name = report.company.name.clone();
    let org_nr = format_org_number(&report.company.org_number);
    decorator.set_header(move |page| {
        let mut layout = LinearLayout::vertical();
        if page > 1 {
            layout.push(
                Paragraph::default()
                    .styled_string(
                        format!("{}  ·  {}", company_name, org_nr),
                        Style::new().with_font_size(SMALL_SIZE),
                    )
                    .aligned(Alignment::Left),
            );
            layout.push(
                Paragraph::default()
                    .styled_string(
                        format!("Sida {}", page),
                        Style::new().with_font_size(SMALL_SIZE),
                    )
                    .aligned(Alignment::Right),
            );
            layout.push(Break::new(1.0));
        }
        layout
    });
    doc.set_page_decorator(decorator);

    let bold = Style::new().bold();
    let h1 = Style::new().bold().with_font_size(H1_SIZE);
    let h2 = Style::new().bold().with_font_size(H2_SIZE);
    let h3 = Style::new().bold().with_font_size(H3_SIZE);

    // === Title page ===
    doc.push(Break::new(6.0));
    doc.push(
        Paragraph::new("ÅRSREDOVISNING")
            .aligned(Alignment::Center)
            .styled(h1),
    );
    doc.push(Break::new(0.8));
    doc.push(
        Paragraph::new("för")
            .aligned(Alignment::Center)
            .styled(Style::new().with_font_size(BODY_SIZE)),
    );
    doc.push(Break::new(0.4));
    doc.push(
        Paragraph::new(&report.company.name)
            .aligned(Alignment::Center)
            .styled(h2),
    );
    doc.push(Break::new(0.3));
    doc.push(
        Paragraph::new(format!("Org.nr {}", format_org_number(&report.company.org_number)))
            .aligned(Alignment::Center),
    );
    doc.push(Break::new(1.0));
    doc.push(
        Paragraph::new(format!(
            "Räkenskapsår {} – {}",
            report.fiscal_year.start_date, report.fiscal_year.end_date
        ))
        .aligned(Alignment::Center),
    );
    doc.push(Break::new(3.0));
    doc.push(
        Paragraph::new("Styrelsen avger följande årsredovisning.")
            .aligned(Alignment::Center)
            .styled(Style::new().italic().with_font_size(SMALL_SIZE)),
    );

    // === Förvaltningsberättelse ===
    doc.push(PageBreak::new());
    doc.push(Paragraph::new("Förvaltningsberättelse").styled(h1));
    doc.push(Break::new(0.8));

    push_section(&mut doc, "Allmänt om verksamheten", &report.directors_report.business_description, h3);
    push_section(&mut doc, "Väsentliga händelser under räkenskapsåret", &report.directors_report.important_events, h3);
    push_section(&mut doc, "Framtida utveckling", &report.directors_report.future_outlook, h3);

    if let Some(ref pa) = report.directors_report.profit_allocation {
        doc.push(Paragraph::new("Förslag till resultatdisposition").styled(h3));
        doc.push(Break::new(0.3));
        doc.push(Paragraph::new("Styrelsen föreslår att till förfogande stående medel:"));
        doc.push(Break::new(0.2));

        let mut tbl = money_table();
        push_money_row(&mut tbl, "Årets resultat", pa.result_for_year, false);
        push_money_row(&mut tbl, "Balanserat resultat", pa.retained_earnings, false);
        push_money_row(&mut tbl, "Summa", pa.total_available, true);
        doc.push(tbl);
        doc.push(Break::new(0.5));

        doc.push(Paragraph::new("disponeras enligt följande:"));
        doc.push(Break::new(0.2));

        let mut tbl = money_table();
        if !pa.dividend.is_zero() {
            push_money_row(&mut tbl, "Utdelning till aktieägarna", pa.dividend, false);
        }
        push_money_row(&mut tbl, "I ny räkning överföres", pa.carry_forward, false);
        push_money_row(&mut tbl, "Summa", pa.total_available, true);
        doc.push(tbl);
    }

    // === Resultaträkning ===
    doc.push(PageBreak::new());
    doc.push(Paragraph::new("Resultaträkning").styled(h1));
    doc.push(Break::new(0.2));
    doc.push(
        Paragraph::default()
            .styled_string("Belopp i kronor", Style::new().italic().with_font_size(SMALL_SIZE)),
    );
    doc.push(Break::new(0.5));

    let is = &report.income_statement.current;
    let prev = report.income_statement.previous.as_ref();

    let prev_label = prev
        .map(|p| p.fiscal_year.clone())
        .unwrap_or_else(|| "—".to_string());

    let mut tbl = statement_table();
    push_header_row(&mut tbl, "", &is.fiscal_year, &prev_label);
    push_stmt_row(&mut tbl, "Nettoomsättning", is.net_revenue, prev.map(|p| p.net_revenue), false);
    if !is.other_operating_income.is_zero() {
        push_stmt_row(&mut tbl, "Övriga rörelseintäkter", is.other_operating_income, prev.map(|p| p.other_operating_income), false);
    }
    push_stmt_row(&mut tbl, "Råvaror och förnödenheter", -is.raw_materials, prev.map(|p| -p.raw_materials), false);
    push_stmt_row(&mut tbl, "Övriga externa kostnader", -is.other_external_costs, prev.map(|p| -p.other_external_costs), false);
    push_stmt_row(&mut tbl, "Personalkostnader", -is.personnel_costs, prev.map(|p| -p.personnel_costs), false);
    if !is.depreciation.is_zero() {
        push_stmt_row(&mut tbl, "Av- och nedskrivningar", -is.depreciation, prev.map(|p| -p.depreciation), false);
    }
    push_stmt_row(&mut tbl, "Rörelseresultat", is.operating_result, prev.map(|p| p.operating_result), true);
    if !is.financial_income.is_zero() || !is.financial_costs.is_zero() {
        push_stmt_row(&mut tbl, "Finansiella intäkter", is.financial_income, prev.map(|p| p.financial_income), false);
        push_stmt_row(&mut tbl, "Finansiella kostnader", -is.financial_costs, prev.map(|p| -p.financial_costs), false);
    }
    push_stmt_row(&mut tbl, "Resultat efter finansiella poster", is.result_after_financial, prev.map(|p| p.result_after_financial), true);
    if !is.tax.is_zero() {
        push_stmt_row(&mut tbl, "Skatt på årets resultat", -is.tax, prev.map(|p| -p.tax), false);
    }
    push_stmt_row(&mut tbl, "Årets resultat", is.net_result, prev.map(|p| p.net_result), true);
    doc.push(tbl);

    // === Balansräkning ===
    doc.push(PageBreak::new());
    doc.push(Paragraph::new("Balansräkning").styled(h1));
    doc.push(Break::new(0.2));
    doc.push(
        Paragraph::default()
            .styled_string("Belopp i kronor", Style::new().italic().with_font_size(SMALL_SIZE)),
    );
    doc.push(Break::new(0.5));

    let bs = &report.balance_sheet.current;
    let bsp = report.balance_sheet.previous.as_ref();
    let bs_prev_label = bsp
        .map(|p| p.fiscal_year.clone())
        .unwrap_or_else(|| "—".to_string());

    doc.push(Paragraph::new("Tillgångar").styled(h2));
    doc.push(Break::new(0.3));

    let mut tbl = statement_table();
    push_header_row(&mut tbl, "", &bs.fiscal_year, &bs_prev_label);
    push_subhead_row(&mut tbl, "Anläggningstillgångar");
    if !bs.assets.intangible_assets.is_zero() {
        push_stmt_row(&mut tbl, "  Immateriella anläggningstillgångar", bs.assets.intangible_assets, bsp.map(|p| p.assets.intangible_assets), false);
    }
    if !bs.assets.tangible_assets.is_zero() {
        push_stmt_row(&mut tbl, "  Materiella anläggningstillgångar", bs.assets.tangible_assets, bsp.map(|p| p.assets.tangible_assets), false);
    }
    if !bs.assets.financial_fixed_assets.is_zero() {
        push_stmt_row(&mut tbl, "  Finansiella anläggningstillgångar", bs.assets.financial_fixed_assets, bsp.map(|p| p.assets.financial_fixed_assets), false);
    }
    push_stmt_row(&mut tbl, "Summa anläggningstillgångar", bs.assets.total_fixed_assets, bsp.map(|p| p.assets.total_fixed_assets), true);
    push_subhead_row(&mut tbl, "Omsättningstillgångar");
    if !bs.assets.inventory.is_zero() {
        push_stmt_row(&mut tbl, "  Varulager", bs.assets.inventory, bsp.map(|p| p.assets.inventory), false);
    }
    push_stmt_row(&mut tbl, "  Kortfristiga fordringar", bs.assets.current_receivables, bsp.map(|p| p.assets.current_receivables), false);
    push_stmt_row(&mut tbl, "  Kassa och bank", bs.assets.cash_and_bank, bsp.map(|p| p.assets.cash_and_bank), false);
    push_stmt_row(&mut tbl, "Summa omsättningstillgångar", bs.assets.total_current_assets, bsp.map(|p| p.assets.total_current_assets), true);
    push_stmt_row(&mut tbl, "Summa tillgångar", bs.total_assets, bsp.map(|p| p.total_assets), true);
    doc.push(tbl);

    doc.push(Break::new(1.0));
    doc.push(Paragraph::new("Eget kapital och skulder").styled(h2));
    doc.push(Break::new(0.3));

    let mut tbl = statement_table();
    push_header_row(&mut tbl, "", &bs.fiscal_year, &bs_prev_label);
    push_subhead_row(&mut tbl, "Eget kapital");
    push_stmt_row(&mut tbl, "  Bundet eget kapital", bs.equity_and_liabilities.restricted_equity, bsp.map(|p| p.equity_and_liabilities.restricted_equity), false);
    push_stmt_row(&mut tbl, "  Fritt eget kapital", bs.equity_and_liabilities.unrestricted_equity, bsp.map(|p| p.equity_and_liabilities.unrestricted_equity), false);
    push_stmt_row(&mut tbl, "Summa eget kapital", bs.equity_and_liabilities.total_equity, bsp.map(|p| p.equity_and_liabilities.total_equity), true);
    if !bs.equity_and_liabilities.untaxed_reserves.is_zero() {
        push_stmt_row(&mut tbl, "Obeskattade reserver", bs.equity_and_liabilities.untaxed_reserves, bsp.map(|p| p.equity_and_liabilities.untaxed_reserves), false);
    }
    if !bs.equity_and_liabilities.long_term_liabilities.is_zero() {
        push_stmt_row(&mut tbl, "Långfristiga skulder", bs.equity_and_liabilities.long_term_liabilities, bsp.map(|p| p.equity_and_liabilities.long_term_liabilities), false);
    }
    push_stmt_row(&mut tbl, "Kortfristiga skulder", bs.equity_and_liabilities.current_liabilities, bsp.map(|p| p.equity_and_liabilities.current_liabilities), false);
    push_stmt_row(&mut tbl, "Summa eget kapital och skulder", bs.total_equity_and_liabilities, bsp.map(|p| p.total_equity_and_liabilities), true);
    doc.push(tbl);

    // === Noter ===
    doc.push(PageBreak::new());
    doc.push(Paragraph::new("Noter").styled(h1));
    doc.push(Break::new(0.8));

    for note in &report.notes.items {
        doc.push(
            Paragraph::new(format!("Not {}  {}", note.number, note.title)).styled(h3),
        );
        doc.push(Break::new(0.3));
        for line in note.content.lines() {
            if line.is_empty() {
                doc.push(Break::new(0.3));
            } else {
                doc.push(Paragraph::new(line));
            }
        }
        doc.push(Break::new(0.8));
    }

    // === Signatures ===
    doc.push(PageBreak::new());
    doc.push(Paragraph::new("Underskrifter").styled(h1));
    doc.push(Break::new(1.5));
    doc.push(Paragraph::new(format!(
        "{}, den _______________",
        report.company.name
    )));
    doc.push(Break::new(2.5));
    doc.push(Paragraph::new("_________________________________").styled(bold));
    doc.push(Paragraph::new("Styrelseledamot"));
    doc.push(Break::new(2.0));
    doc.push(Paragraph::new("_________________________________").styled(bold));
    doc.push(Paragraph::new("Styrelseledamot"));
    doc.push(Break::new(2.5));
    doc.push(
        Paragraph::default().styled_string(
            "Vår revisionsberättelse har lämnats denna dag.",
            Style::new().italic().with_font_size(SMALL_SIZE),
        ),
    );

    let mut buf = Vec::new();
    doc.render(&mut buf)
        .map_err(|e| format!("PDF render error: {e}"))?;
    Ok(buf)
}

fn push_section(doc: &mut Document, title: &str, body: &str, heading_style: Style) {
    doc.push(Paragraph::new(title).styled(heading_style));
    doc.push(Break::new(0.3));
    let text = if body.trim().is_empty() { "—" } else { body };
    for para in text.split("\n\n") {
        let joined: String = para
            .lines()
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join(" ");
        if !joined.trim().is_empty() {
            doc.push(Paragraph::new(joined));
            doc.push(Break::new(0.3));
        }
    }
    doc.push(Break::new(0.6));
}

fn money_table() -> TableLayout {
    TableLayout::new(vec![7, 3])
}

fn statement_table() -> TableLayout {
    TableLayout::new(vec![8, 3, 3])
}

fn push_money_row(tbl: &mut TableLayout, label: &str, amount: Money, bold_row: bool) {
    let style = if bold_row { Style::new().bold() } else { Style::new() };
    tbl.row()
        .element(Paragraph::default().styled_string(label, style))
        .element(
            Paragraph::default()
                .styled_string(format_sek(amount), style)
                .aligned(Alignment::Right),
        )
        .push()
        .ok();
}

fn push_header_row(tbl: &mut TableLayout, label: &str, current: &str, previous: &str) {
    let style = Style::new().bold().with_font_size(SMALL_SIZE);
    tbl.row()
        .element(Paragraph::default().styled_string(label, style))
        .element(
            Paragraph::default()
                .styled_string(current, style)
                .aligned(Alignment::Right),
        )
        .element(
            Paragraph::default()
                .styled_string(previous, style)
                .aligned(Alignment::Right),
        )
        .push()
        .ok();
}

fn push_subhead_row(tbl: &mut TableLayout, label: &str) {
    let style = Style::new().bold();
    tbl.row()
        .element(Paragraph::default().styled_string(label, style))
        .element(Paragraph::new(""))
        .element(Paragraph::new(""))
        .push()
        .ok();
}

fn push_stmt_row(
    tbl: &mut TableLayout,
    label: &str,
    current: Money,
    previous: Option<Money>,
    bold_row: bool,
) {
    let style = if bold_row { Style::new().bold() } else { Style::new() };
    let prev_str = previous.map(format_sek).unwrap_or_default();
    tbl.row()
        .element(Paragraph::default().styled_string(label, style))
        .element(
            Paragraph::default()
                .styled_string(format_sek(current), style)
                .aligned(Alignment::Right),
        )
        .element(
            Paragraph::default()
                .styled_string(prev_str, style)
                .aligned(Alignment::Right),
        )
        .push()
        .ok();
}

/// Format as Swedish kronor (whole kronor, space thousands separator).
/// Example: -12345 → "-12 345", 1234567 → "1 234 567".
fn format_sek(m: Money) -> String {
    let rounded = m.inner().round().to_string();
    let negative = rounded.starts_with('-');
    let digits: &str = if negative { &rounded[1..] } else { &rounded };
    let digits = digits.trim_start_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };

    let mut out = String::new();
    let bytes = digits.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        let from_right = bytes.len() - i;
        if i > 0 && from_right % 3 == 0 {
            out.push(' ');
        }
        out.push(*b as char);
    }
    if negative {
        format!("-{out}")
    } else {
        out
    }
}

fn format_org_number(org: &str) -> String {
    if org.len() == 10 && !org.contains('-') {
        format!("{}-{}", &org[..6], &org[6..])
    } else {
        org.to_string()
    }
}
