use genpdf::{
    elements::{Break, Paragraph},
    fonts,
    style::Style,
    Alignment, Document, Element,
};

use super::annual_report::AnnualReport;
use crate::money::Money;

/// Generate a PDF for the annual report.
pub fn generate_pdf(report: &AnnualReport) -> Result<Vec<u8>, String> {
    let font =
        fonts::from_files("./fonts", "LiberationSans", None).unwrap_or_else(|_| {
            // Fallback: try system font paths
            fonts::from_files("/usr/share/fonts/truetype/liberation", "LiberationSans", None)
                .unwrap_or_else(|_| {
                    fonts::from_files(
                        "/usr/share/fonts/liberation-sans",
                        "LiberationSans",
                        None,
                    )
                    .unwrap_or_else(|_| {
                        // Use built-in font as last resort
                        genpdf::fonts::from_files("/usr/share/fonts", "DejaVuSans", None)
                            .expect("No suitable font found. Install liberation-fonts or dejavu-fonts.")
                    })
                })
        });

    let mut doc = Document::new(font);
    doc.set_title(format!(
        "Årsredovisning {} {}",
        report.company.name, report.fiscal_year.end_date
    ));
    doc.set_minimal_conformance();
    doc.set_line_spacing(1.5);

    let bold = Style::new().bold();
    let heading = Style::new().bold().with_font_size(16);
    let subheading = Style::new().bold().with_font_size(12);

    // === Title page ===
    doc.push(Break::new(3.0));
    doc.push(
        Paragraph::new("ÅRSREDOVISNING")
            .aligned(Alignment::Center)
            .styled(Style::new().bold().with_font_size(20)),
    );
    doc.push(Break::new(1.0));
    doc.push(
        Paragraph::new(&report.company.name)
            .aligned(Alignment::Center)
            .styled(heading),
    );
    doc.push(
        Paragraph::new(format!("Org.nr: {}", format_org_number(&report.company.org_number)))
            .aligned(Alignment::Center),
    );
    doc.push(Break::new(0.5));
    doc.push(
        Paragraph::new(format!(
            "Räkenskapsår: {} — {}",
            report.fiscal_year.start_date, report.fiscal_year.end_date
        ))
        .aligned(Alignment::Center),
    );

    doc.push(Break::new(4.0));

    // === Förvaltningsberättelse ===
    doc.push(Paragraph::new("FÖRVALTNINGSBERÄTTELSE").styled(heading));
    doc.push(Break::new(0.5));

    doc.push(Paragraph::new("Verksamheten").styled(subheading));
    doc.push(Paragraph::new(&report.directors_report.business_description));
    doc.push(Break::new(0.5));

    doc.push(Paragraph::new("Väsentliga händelser under räkenskapsåret").styled(subheading));
    doc.push(Paragraph::new(&report.directors_report.important_events));
    doc.push(Break::new(0.5));

    doc.push(Paragraph::new("Framtida utveckling").styled(subheading));
    doc.push(Paragraph::new(&report.directors_report.future_outlook));
    doc.push(Break::new(0.5));

    if let Some(ref pa) = report.directors_report.profit_allocation {
        doc.push(Paragraph::new("Förslag till vinstdisposition").styled(subheading));
        doc.push(Paragraph::new("Styrelsen föreslår att till förfogande stående medel:"));
        doc.push(Paragraph::new(format!("  Årets resultat: {} kr", pa.result_for_year)));
        doc.push(Paragraph::new(format!(
            "  Balanserat resultat: {} kr",
            pa.retained_earnings
        )));
        doc.push(Paragraph::new(format!(
            "  Summa: {} kr",
            pa.total_available
        )));
        doc.push(Break::new(0.3));
        doc.push(Paragraph::new("Disponeras enligt följande:"));
        doc.push(Paragraph::new(format!(
            "  I ny räkning överföres: {} kr",
            pa.carry_forward
        )));
        if !pa.dividend.is_zero() {
            doc.push(Paragraph::new(format!(
                "  Utdelning till aktieägarna: {} kr",
                pa.dividend
            )));
        }
    }

    doc.push(Break::new(1.5));

    // === Resultaträkning ===
    doc.push(Paragraph::new("RESULTATRÄKNING").styled(heading));
    doc.push(Break::new(0.5));

    let is = &report.income_statement.current;
    let prev = report.income_statement.previous.as_ref();

    let add_is_row = |doc: &mut Document, label: &str, current: Money, previous: Option<Money>, is_bold: bool| {
        let style = if is_bold { bold } else { Style::new() };
        let prev_str = previous.map(|p| format!("{}", p)).unwrap_or_default();
        doc.push(Paragraph::new(format!(
            "{:<45} {:>15} {:>15}",
            label, current, prev_str
        )).styled(style));
    };

    // Header row
    let prev_label = prev
        .map(|p| p.fiscal_year.clone())
        .unwrap_or_default();
    doc.push(
        Paragraph::new(format!(
            "{:<45} {:>15} {:>15}",
            "", &is.fiscal_year, prev_label
        ))
        .styled(bold),
    );

    add_is_row(&mut doc, "Nettoomsättning", is.net_revenue, prev.map(|p| p.net_revenue), false);
    if !is.other_operating_income.is_zero() {
        add_is_row(&mut doc, "Övriga rörelseintäkter", is.other_operating_income, prev.map(|p| p.other_operating_income), false);
    }
    add_is_row(&mut doc, "Råvaror och förnödenheter", -is.raw_materials, prev.map(|p| -p.raw_materials), false);
    add_is_row(&mut doc, "Övriga externa kostnader", -is.other_external_costs, prev.map(|p| -p.other_external_costs), false);
    add_is_row(&mut doc, "Personalkostnader", -is.personnel_costs, prev.map(|p| -p.personnel_costs), false);
    if !is.depreciation.is_zero() {
        add_is_row(&mut doc, "Av- och nedskrivningar", -is.depreciation, prev.map(|p| -p.depreciation), false);
    }
    doc.push(Paragraph::new(""));
    add_is_row(&mut doc, "Rörelseresultat", is.operating_result, prev.map(|p| p.operating_result), true);
    doc.push(Paragraph::new(""));
    if !is.financial_income.is_zero() || !is.financial_costs.is_zero() {
        add_is_row(&mut doc, "Finansiella intäkter", is.financial_income, prev.map(|p| p.financial_income), false);
        add_is_row(&mut doc, "Finansiella kostnader", -is.financial_costs, prev.map(|p| -p.financial_costs), false);
    }
    add_is_row(&mut doc, "Resultat efter finansiella poster", is.result_after_financial, prev.map(|p| p.result_after_financial), true);
    if !is.tax.is_zero() {
        add_is_row(&mut doc, "Skatt på årets resultat", -is.tax, prev.map(|p| -p.tax), false);
    }
    doc.push(Paragraph::new(""));
    add_is_row(&mut doc, "ÅRETS RESULTAT", is.net_result, prev.map(|p| p.net_result), true);

    doc.push(Break::new(1.5));

    // === Balansräkning ===
    doc.push(Paragraph::new("BALANSRÄKNING").styled(heading));
    doc.push(Break::new(0.5));

    let bs = &report.balance_sheet.current;
    let bsp = report.balance_sheet.previous.as_ref();

    let add_bs_row = |doc: &mut Document, label: &str, current: Money, previous: Option<Money>, is_bold: bool| {
        let style = if is_bold { bold } else { Style::new() };
        let prev_str = previous.map(|p| format!("{}", p)).unwrap_or_default();
        doc.push(Paragraph::new(format!(
            "{:<45} {:>15} {:>15}",
            label, current, prev_str
        )).styled(style));
    };

    doc.push(Paragraph::new("TILLGÅNGAR").styled(subheading));
    doc.push(Paragraph::new("Anläggningstillgångar").styled(bold));
    if !bs.assets.intangible_assets.is_zero() {
        add_bs_row(&mut doc, "  Immateriella anläggningstillgångar", bs.assets.intangible_assets, bsp.map(|p| p.assets.intangible_assets), false);
    }
    if !bs.assets.tangible_assets.is_zero() {
        add_bs_row(&mut doc, "  Materiella anläggningstillgångar", bs.assets.tangible_assets, bsp.map(|p| p.assets.tangible_assets), false);
    }
    if !bs.assets.financial_fixed_assets.is_zero() {
        add_bs_row(&mut doc, "  Finansiella anläggningstillgångar", bs.assets.financial_fixed_assets, bsp.map(|p| p.assets.financial_fixed_assets), false);
    }
    add_bs_row(&mut doc, "Summa anläggningstillgångar", bs.assets.total_fixed_assets, bsp.map(|p| p.assets.total_fixed_assets), true);

    doc.push(Break::new(0.3));
    doc.push(Paragraph::new("Omsättningstillgångar").styled(bold));
    if !bs.assets.inventory.is_zero() {
        add_bs_row(&mut doc, "  Varulager", bs.assets.inventory, bsp.map(|p| p.assets.inventory), false);
    }
    add_bs_row(&mut doc, "  Kortfristiga fordringar", bs.assets.current_receivables, bsp.map(|p| p.assets.current_receivables), false);
    add_bs_row(&mut doc, "  Kassa och bank", bs.assets.cash_and_bank, bsp.map(|p| p.assets.cash_and_bank), false);
    add_bs_row(&mut doc, "Summa omsättningstillgångar", bs.assets.total_current_assets, bsp.map(|p| p.assets.total_current_assets), true);
    doc.push(Break::new(0.3));
    add_bs_row(&mut doc, "SUMMA TILLGÅNGAR", bs.total_assets, bsp.map(|p| p.total_assets), true);

    doc.push(Break::new(1.0));

    doc.push(Paragraph::new("EGET KAPITAL OCH SKULDER").styled(subheading));
    doc.push(Paragraph::new("Eget kapital").styled(bold));
    add_bs_row(&mut doc, "  Bundet eget kapital", bs.equity_and_liabilities.restricted_equity, bsp.map(|p| p.equity_and_liabilities.restricted_equity), false);
    add_bs_row(&mut doc, "  Fritt eget kapital", bs.equity_and_liabilities.unrestricted_equity, bsp.map(|p| p.equity_and_liabilities.unrestricted_equity), false);
    add_bs_row(&mut doc, "Summa eget kapital", bs.equity_and_liabilities.total_equity, bsp.map(|p| p.equity_and_liabilities.total_equity), true);

    if !bs.equity_and_liabilities.untaxed_reserves.is_zero() {
        doc.push(Break::new(0.3));
        add_bs_row(&mut doc, "Obeskattade reserver", bs.equity_and_liabilities.untaxed_reserves, bsp.map(|p| p.equity_and_liabilities.untaxed_reserves), false);
    }

    if !bs.equity_and_liabilities.long_term_liabilities.is_zero() {
        doc.push(Break::new(0.3));
        add_bs_row(&mut doc, "Långfristiga skulder", bs.equity_and_liabilities.long_term_liabilities, bsp.map(|p| p.equity_and_liabilities.long_term_liabilities), false);
    }

    doc.push(Break::new(0.3));
    add_bs_row(&mut doc, "Kortfristiga skulder", bs.equity_and_liabilities.current_liabilities, bsp.map(|p| p.equity_and_liabilities.current_liabilities), false);

    doc.push(Break::new(0.3));
    add_bs_row(&mut doc, "SUMMA EGET KAPITAL OCH SKULDER", bs.total_equity_and_liabilities, bsp.map(|p| p.total_equity_and_liabilities), true);

    doc.push(Break::new(1.5));

    // === Noter ===
    doc.push(Paragraph::new("NOTER").styled(heading));
    doc.push(Break::new(0.5));

    for note in &report.notes.items {
        doc.push(Paragraph::new(format!("Not {}: {}", note.number, note.title)).styled(subheading));
        for line in note.content.lines() {
            doc.push(Paragraph::new(line));
        }
        doc.push(Break::new(0.5));
    }

    // Render to bytes
    let mut buf = Vec::new();
    doc.render(&mut buf)
        .map_err(|e| format!("PDF render error: {e}"))?;

    Ok(buf)
}

fn format_org_number(org: &str) -> String {
    if org.len() == 10 && !org.contains('-') {
        format!("{}-{}", &org[..6], &org[6..])
    } else {
        org.to_string()
    }
}
