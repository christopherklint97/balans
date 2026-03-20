use serde::Serialize;
use sqlx::SqlitePool;

/// K2 required notes per BFNAR 2016:10, chapter 18-19.
#[derive(Debug, Clone, Serialize)]
pub struct Notes {
    pub items: Vec<Note>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Note {
    pub number: i32,
    pub title: String,
    pub content: String,
}

/// Build the required notes for K2 årsredovisning.
pub async fn build_notes(
    pool: &SqlitePool,
    company_id: &str,
    fiscal_year_id: &str,
) -> Result<Notes, sqlx::Error> {
    let _company = sqlx::query_as::<_, crate::models::company::Company>(
        "SELECT * FROM companies WHERE id = ?",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await?;

    let _fy = sqlx::query_as::<_, crate::models::fiscal_year::FiscalYear>(
        "SELECT * FROM fiscal_years WHERE id = ?",
    )
    .bind(fiscal_year_id)
    .fetch_one(pool)
    .await?;

    let mut notes = Vec::new();

    // Note 1: Redovisningsprinciper (Accounting principles) — always required
    notes.push(Note {
        number: 1,
        title: "Redovisningsprinciper".into(),
        content: format!(
            "Årsredovisningen har upprättats i enlighet med årsredovisningslagen och \
             Bokföringsnämndens allmänna råd BFNAR 2016:10 Årsredovisning i mindre företag (K2).\n\n\
             Företaget tillämpar K2-regelverket i sin helhet.\n\n\
             Intäkter redovisas i den period de avser. Fordringar upptas till det belopp \
             som efter individuell bedömning beräknas bli betalt.\n\n\
             Tillgångar och skulder värderas till anskaffningsvärde om inget annat anges."
        ),
    });

    // Note 2: Medelantal anställda (average employees) — always required for K2
    notes.push(Note {
        number: 2,
        title: "Medelantal anställda".into(),
        content: "Medelantalet anställda under räkenskapsåret har uppgått till 0.".into(),
    });

    // Note 3: Ställda säkerheter och ansvarsförbindelser — always required
    notes.push(Note {
        number: 3,
        title: "Ställda säkerheter och ansvarsförbindelser".into(),
        content: "Ställda säkerheter: Inga\nAnsvarsförbindelser: Inga".into(),
    });

    Ok(Notes { items: notes })
}
