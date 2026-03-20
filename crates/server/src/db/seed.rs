use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Seed a company with the core BAS kontoplan accounts (~100 most common for a small AB).
pub async fn seed_bas_accounts(pool: &SqlitePool, company_id: &str) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();

    let accounts: Vec<(i32, &str, &str)> = vec![
        // Class 1: Tillgångar (Assets)
        // 10xx Immateriella anläggningstillgångar
        (1010, "Utvecklingsutgifter", "asset"),
        (1020, "Koncessioner", "asset"),
        (1030, "Patent", "asset"),
        (1050, "Goodwill", "asset"),
        (1070, "Pågående projekt immateriella", "asset"),
        (1080, "Ack avskrivningar immateriella", "asset"),
        // 11xx-12xx Materiella anläggningstillgångar
        (1110, "Byggnader", "asset"),
        (1119, "Ack avskrivningar byggnader", "asset"),
        (1210, "Maskiner och inventarier", "asset"),
        (1219, "Ack avskrivningar maskiner", "asset"),
        (1220, "Inventarier och verktyg", "asset"),
        (1229, "Ack avskrivningar inventarier", "asset"),
        (1240, "Bilar och transportmedel", "asset"),
        (1249, "Ack avskrivningar bilar", "asset"),
        (1250, "Datorer", "asset"),
        (1259, "Ack avskrivningar datorer", "asset"),
        // 13xx Finansiella anläggningstillgångar
        (1310, "Andelar i koncernföretag", "asset"),
        (1380, "Andra långfristiga fordringar", "asset"),
        // 14xx Lager
        (1400, "Lager av handelsvaror", "asset"),
        (1410, "Lager av råvaror", "asset"),
        (1460, "Lager av färdiga varor", "asset"),
        (1470, "Pågående arbeten", "asset"),
        // 15xx-17xx Kortfristiga fordringar
        (1510, "Kundfordringar", "asset"),
        (1610, "Fordringar hos anställda", "asset"),
        (1630, "Avräkning för skatter och avgifter", "asset"),
        (1650, "Momsfordran", "asset"),
        (1710, "Förutbetalda hyreskostnader", "asset"),
        (1790, "Övriga förutbetalda kostnader", "asset"),
        // 19xx Kassa och bank
        (1910, "Kassa", "asset"),
        (1920, "PlusGiro", "asset"),
        (1930, "Företagskonto", "asset"),
        (1940, "Övriga bankkonton", "asset"),

        // Class 2: Eget kapital och skulder
        // 20xx Eget kapital
        (2010, "Eget kapital", "equity"),
        (2011, "Aktiekapital", "equity"),
        (2013, "Överkursfond", "equity"),
        (2020, "Reservfond", "equity"),
        (2081, "Ackumulerade vinster/förluster", "equity"),
        (2091, "Balanserad vinst eller förlust", "equity"),
        (2098, "Vinst eller förlust från föregående år", "equity"),
        (2099, "Årets resultat", "equity"),
        // 21xx Obeskattade reserver
        (2110, "Periodiseringsfonder", "liability"),
        (2150, "Ackumulerade överavskrivningar", "liability"),
        // 23xx-29xx Skulder
        (2310, "Banklån", "liability"),
        (2330, "Checkräkningskredit", "liability"),
        (2395, "Övriga långfristiga skulder", "liability"),
        (2440, "Leverantörsskulder", "liability"),
        (2510, "Skatteskulder", "liability"),
        (2610, "Utgående moms 25%", "liability"),
        (2620, "Utgående moms 12%", "liability"),
        (2630, "Utgående moms 6%", "liability"),
        (2640, "Ingående moms", "liability"),
        (2650, "Redovisningskonto för moms", "liability"),
        (2710, "Personalskatt", "liability"),
        (2730, "Arbetsgivaravgifter", "liability"),
        (2790, "Övriga kortfristiga skulder", "liability"),
        (2820, "Kortfristig del av långfristiga skulder", "liability"),
        (2890, "Övriga upplupna kostnader", "liability"),
        (2910, "Upplupna löner", "liability"),
        (2920, "Upplupna semesterlöner", "liability"),
        (2940, "Upplupna arbetsgivaravgifter", "liability"),
        (2990, "Övriga upplupna kostnader", "liability"),

        // Class 3: Rörelseintäkter (Revenue)
        (3001, "Försäljning varor 25% moms", "revenue"),
        (3002, "Försäljning varor 12% moms", "revenue"),
        (3003, "Försäljning varor 6% moms", "revenue"),
        (3010, "Försäljning tjänster 25% moms", "revenue"),
        (3040, "Försäljning tjänster utanför Sverige", "revenue"),
        (3510, "Fakturerade kostnader", "revenue"),
        (3590, "Övriga sidointäkter", "revenue"),
        (3740, "Öres- och kronavsrundning", "revenue"),
        (3960, "Valutakursvinster", "revenue"),

        // Class 4: Material och varukostnader (Cost of goods)
        (4010, "Inköp varor och material", "expense"),
        (4110, "Inköp insatsvaror", "expense"),
        (4500, "Övriga kostnader för handelsvaror", "expense"),
        (4531, "Import av varor, EU, 25%", "expense"),
        (4600, "Legoarbeten och underentreprenader", "expense"),

        // Class 5: Övriga externa kostnader
        (5010, "Lokalhyra", "expense"),
        (5020, "El för lokal", "expense"),
        (5090, "Övriga lokalkostnader", "expense"),
        (5210, "Hyra av anläggningstillgångar", "expense"),
        (5410, "Förbrukningsinventarier", "expense"),
        (5420, "Programvaror", "expense"),
        (5460, "Förbrukningsmaterial", "expense"),
        (5500, "Reparation och underhåll", "expense"),
        (5600, "Kostnader för transportmedel", "expense"),
        (5610, "Personbilskostnader", "expense"),
        (5800, "Resekostnader", "expense"),
        (5810, "Biljetter", "expense"),
        (5831, "Kost och logi Sverige", "expense"),

        // Class 6: Övriga externa kostnader forts.
        (6071, "Representation avdragsgill", "expense"),
        (6072, "Representation ej avdragsgill", "expense"),
        (6110, "Kontorsmateriel", "expense"),
        (6200, "Tele och post", "expense"),
        (6211, "Telefon", "expense"),
        (6212, "Mobiltelefon", "expense"),
        (6230, "Datakommunikation", "expense"),
        (6250, "Postbefordran", "expense"),
        (6310, "Företagsförsäkringar", "expense"),
        (6530, "Redovisningstjänster", "expense"),
        (6540, "IT-tjänster", "expense"),
        (6550, "Konsultarvoden", "expense"),
        (6570, "Bankkostnader", "expense"),
        (6900, "Övriga externa kostnader", "expense"),

        // Class 7: Personalkostnader
        (7010, "Löner till tjänstemän", "expense"),
        (7082, "Sjuklöner", "expense"),
        (7090, "Förändring av semesterlöneskuld", "expense"),
        (7210, "Löner till kollektivanställda", "expense"),
        (7310, "Kontanta extraersättningar", "expense"),
        (7385, "Förmånsvärde fritt drivmedel", "expense"),
        (7510, "Arbetsgivaravgifter", "expense"),
        (7519, "Sociala avgifter semesterlöneskuld", "expense"),
        (7533, "Särskild löneskatt pensionskostn.", "expense"),
        (7570, "Premier för arbetsmarknadsförsäkr.", "expense"),
        (7610, "Utbildning", "expense"),
        (7631, "Personalrepresentation avdragsgill", "expense"),
        (7699, "Övriga personalkostnader", "expense"),

        // 77xx-78xx Avskrivningar (Depreciation)
        (7810, "Avskrivningar immateriella tillgångar", "expense"),
        (7820, "Avskrivningar byggnader", "expense"),
        (7831, "Avskrivningar maskiner", "expense"),
        (7832, "Avskrivningar inventarier", "expense"),
        (7833, "Avskrivningar bilar", "expense"),
        (7834, "Avskrivningar datorer", "expense"),
        (7840, "Nedskrivningar", "expense"),

        // Class 8: Finansiella poster, bokslutsdispositioner, skatt
        (8310, "Ränteintäkter", "revenue"),
        (8314, "Skattefria ränteintäkter", "revenue"),
        (8410, "Räntekostnader", "expense"),
        (8420, "Räntekostnader banklån", "expense"),
        (8491, "Räntekostnader leverantörsskulder", "expense"),
        (8810, "Förändring periodiseringsfonder", "expense"),
        (8850, "Förändring överavskrivningar", "expense"),
        (8910, "Skatt på årets resultat", "expense"),
        (8999, "Årets resultat", "revenue"),
    ];

    let mut tx = pool.begin().await?;

    for (number, name, account_type) in &accounts {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO accounts (id, company_id, number, name, account_type, is_active, created_at)
             VALUES (?, ?, ?, ?, ?, 1, ?)"
        )
        .bind(&id)
        .bind(company_id)
        .bind(number)
        .bind(name)
        .bind(account_type)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    // Seed SRU codes for common accounts
    seed_sru_codes(pool, company_id).await?;

    Ok(())
}

/// Seed SRU codes for Skatteverket INK2 tax return mapping.
/// Covers balance sheet (INK2 R), income statement (INK2 R), and tax calculation fields.
/// SRU codes map BAS accounts to specific lines in the INK2 form.
async fn seed_sru_codes(pool: &SqlitePool, company_id: &str) -> Result<(), sqlx::Error> {
    let sru_mappings: Vec<(i32, &str)> = vec![
        // ═══ BALANCE SHEET — ASSETS (Tillgångar) ═══

        // Immateriella anläggningstillgångar
        (1010, "7201"), // Balanserade utgifter för utvecklingsarbeten
        (1020, "7202"), // Koncessioner, patent, licenser
        (1030, "7202"), // Patent
        (1050, "7203"), // Goodwill
        (1070, "7207"), // Pågående arbeten immateriella
        (1080, "7201"), // Ack avskr immateriella (nets against 7201)

        // Materiella anläggningstillgångar — Byggnader & mark
        (1110, "7210"), // Byggnader
        (1119, "7210"), // Ack avskr byggnader

        // Materiella anläggningstillgångar — Maskiner & inventarier
        (1210, "7214"), // Maskiner och andra tekniska anläggningar
        (1219, "7214"),
        (1220, "7215"), // Inventarier, verktyg och installationer
        (1229, "7215"),
        (1240, "7215"), // Bilar och transportmedel
        (1249, "7215"),
        (1250, "7215"), // Datorer
        (1259, "7215"),

        // Finansiella anläggningstillgångar
        (1310, "7220"), // Andelar i koncernföretag
        (1380, "7225"), // Andra långfristiga fordringar

        // Varulager
        (1400, "7230"), // Lager av handelsvaror
        (1410, "7230"), // Lager av råvaror
        (1460, "7230"), // Lager av färdiga varor
        (1470, "7234"), // Pågående arbeten

        // Kortfristiga fordringar
        (1510, "7240"), // Kundfordringar
        (1610, "7243"), // Fordringar hos anställda
        (1630, "7248"), // Skattefordringar
        (1650, "7248"), // Momsfordran
        (1710, "7249"), // Förutbetalda kostnader och upplupna intäkter
        (1790, "7249"),

        // Kortfristiga placeringar
        // (18xx would be 7250 if used)

        // Kassa och bank
        (1910, "7260"), // Kassa
        (1920, "7260"), // PlusGiro
        (1930, "7260"), // Företagskonto
        (1940, "7260"), // Övriga bankkonton

        // ═══ BALANCE SHEET — EQUITY & LIABILITIES (Eget kapital och skulder) ═══

        // Bundet eget kapital
        (2011, "7300"), // Aktiekapital
        (2013, "7302"), // Överkursfond
        (2020, "7303"), // Reservfond

        // Fritt eget kapital
        (2081, "7310"), // Balanserat resultat
        (2091, "7310"),
        (2098, "7310"), // Vinst/förlust föregående år
        (2099, "7312"), // Årets resultat

        // Obeskattade reserver
        (2110, "7320"), // Periodiseringsfonder
        (2150, "7321"), // Ackumulerade överavskrivningar

        // Långfristiga skulder
        (2310, "7340"), // Skulder till kreditinstitut (lång)
        (2330, "7340"), // Checkräkningskredit
        (2395, "7349"), // Övriga långfristiga skulder

        // Kortfristiga skulder
        (2440, "7360"), // Leverantörsskulder
        (2510, "7362"), // Skatteskulder
        (2610, "7363"), // Utgående moms 25%
        (2620, "7363"), // Utgående moms 12%
        (2630, "7363"), // Utgående moms 6%
        (2640, "7363"), // Ingående moms
        (2650, "7363"), // Redovisningskonto moms
        (2710, "7365"), // Personalskatt
        (2730, "7365"), // Arbetsgivaravgifter
        (2790, "7369"), // Övriga kortfristiga skulder
        (2820, "7361"), // Kortfristig del av långfristiga skulder
        (2890, "7368"), // Upplupna kostnader och förutbetalda intäkter
        (2910, "7368"),
        (2920, "7368"),
        (2940, "7368"),
        (2990, "7368"),

        // ═══ INCOME STATEMENT (Resultaträkning) ═══

        // Nettoomsättning
        (3001, "7410"), // Försäljning varor 25%
        (3002, "7410"), // Försäljning varor 12%
        (3003, "7410"), // Försäljning varor 6%
        (3010, "7410"), // Försäljning tjänster
        (3040, "7410"), // Försäljning utanför Sverige
        (3510, "7410"), // Fakturerade kostnader
        (3590, "7412"), // Övriga sidointäkter → övriga rörelseintäkter
        (3740, "7412"), // Öresavrundning
        (3960, "7412"), // Valutakursvinster

        // Råvaror och förnödenheter + handelsvaror
        (4010, "7420"), // Inköp varor och material
        (4110, "7420"), // Inköp insatsvaror
        (4500, "7420"), // Övriga kostnader handelsvaror
        (4531, "7420"), // Import varor EU
        (4600, "7420"), // Legoarbeten

        // Övriga externa kostnader
        (5010, "7430"), // Lokalhyra
        (5020, "7430"), // El
        (5090, "7430"), // Övriga lokalkostnader
        (5210, "7430"), // Hyra anläggningstillgångar
        (5410, "7430"), // Förbrukningsinventarier
        (5420, "7430"), // Programvaror
        (5460, "7430"), // Förbrukningsmaterial
        (5500, "7430"), // Reparation och underhåll
        (5600, "7430"), // Kostnader transportmedel
        (5610, "7430"), // Personbilskostnader
        (5800, "7430"), // Resekostnader
        (5810, "7430"), // Biljetter
        (5831, "7430"), // Kost och logi
        (6071, "7430"), // Representation avdragsgill
        (6072, "7430"), // Representation ej avdragsgill
        (6110, "7430"), // Kontorsmateriel
        (6200, "7430"), // Tele och post
        (6211, "7430"), // Telefon
        (6212, "7430"), // Mobiltelefon
        (6230, "7430"), // Datakommunikation
        (6250, "7430"), // Postbefordran
        (6310, "7430"), // Företagsförsäkringar
        (6530, "7430"), // Redovisningstjänster
        (6540, "7430"), // IT-tjänster
        (6550, "7430"), // Konsultarvoden
        (6570, "7430"), // Bankkostnader
        (6900, "7430"), // Övriga externa kostnader

        // Personalkostnader
        (7010, "7440"), // Löner tjänstemän
        (7082, "7440"), // Sjuklöner
        (7090, "7440"), // Förändring semesterlöneskuld
        (7210, "7440"), // Löner kollektivanställda
        (7310, "7440"), // Kontanta extraersättningar
        (7385, "7440"), // Förmånsvärde drivmedel
        (7510, "7441"), // Arbetsgivaravgifter (social costs)
        (7519, "7441"),
        (7533, "7441"), // Särskild löneskatt
        (7570, "7441"), // Arbetsmarknadsförsäkringar
        (7610, "7440"), // Utbildning
        (7631, "7440"), // Personalrepresentation
        (7699, "7440"), // Övriga personalkostnader

        // Avskrivningar
        (7810, "7450"), // Avskrivningar immateriella
        (7820, "7450"), // Avskrivningar byggnader
        (7831, "7450"), // Avskrivningar maskiner
        (7832, "7450"), // Avskrivningar inventarier
        (7833, "7450"), // Avskrivningar bilar
        (7834, "7450"), // Avskrivningar datorer
        (7840, "7450"), // Nedskrivningar

        // Finansiella poster
        (8310, "7510"), // Ränteintäkter
        (8314, "7510"), // Skattefria ränteintäkter
        (8410, "7511"), // Räntekostnader
        (8420, "7511"), // Räntekostnader banklån
        (8491, "7511"), // Räntekostnader leverantörsskulder

        // Bokslutsdispositioner
        (8810, "7520"), // Förändring periodiseringsfonder
        (8850, "7521"), // Förändring överavskrivningar

        // Skatt
        (8910, "7600"), // Skatt på årets resultat
    ];

    for (account_number, sru_code) in &sru_mappings {
        sqlx::query(
            "UPDATE accounts SET sru_code = ? WHERE company_id = ? AND number = ?",
        )
        .bind(sru_code)
        .bind(company_id)
        .bind(account_number)
        .execute(pool)
        .await?;
    }

    Ok(())
}
