use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Seed a company with the complete BAS kontoplan 2026 (all huvudkonton).
/// Source: https://www.bas.se/wp-content/uploads/2026/01/BAS_kontoplan_2026.pdf
pub async fn seed_bas_accounts(pool: &SqlitePool, company_id: &str) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();

    // # after account number in comments = Ej K2
    let accounts: Vec<(i32, &str, &str)> = vec![
        // ═══════════════════════════════════════════════════════════════
        // KONTOKLASS 1 — TILLGÅNGAR (Assets)
        // ═══════════════════════════════════════════════════════════════

        // 10 Immateriella anläggningstillgångar
        (1010, "Utvecklingsutgifter", "asset"),                    // #
        (1020, "Koncessioner m.m.", "asset"),
        (1030, "Patent", "asset"),
        (1040, "Licenser", "asset"),
        (1050, "Varumärken", "asset"),
        (1060, "Hyresrätter och liknande", "asset"),
        (1070, "Goodwill", "asset"),
        (1080, "Pågående projekt och förskott för immateriella anläggningstillgångar", "asset"),
        (1090, "Övriga immateriella anläggningstillgångar", "asset"),

        // 11 Byggnader och mark
        (1110, "Byggnader", "asset"),
        (1120, "Förbättringsutgifter på annans fastighet", "asset"),
        (1130, "Mark", "asset"),
        (1140, "Tomter och obebyggda markområden", "asset"),
        (1150, "Markanläggningar", "asset"),
        (1180, "Pågående nyanläggningar och förskott för byggnader och mark", "asset"),

        // 12 Maskiner respektive inventarier
        (1210, "Maskiner och andra tekniska anläggningar", "asset"),
        (1220, "Inventarier, verktyg och installationer", "asset"),
        (1230, "Maskiner och andra tekniska anläggningar (fritt konto)", "asset"),
        (1240, "Maskiner och andra tekniska anläggningar (fritt konto 2)", "asset"),
        (1250, "Inventarier, verktyg och installationer (fritt konto)", "asset"),
        (1260, "Inventarier, verktyg och installationer (fritt konto 2)", "asset"),
        (1280, "Pågående nyanläggningar och förskott för maskiner respektive inventarier", "asset"),
        (1290, "Övriga materiella anläggningstillgångar", "asset"),

        // 13 Finansiella anläggningstillgångar
        (1310, "Andelar i koncernföretag", "asset"),
        (1320, "Långfristiga fordringar hos koncernföretag", "asset"),
        (1330, "Andelar i intresseföretag och gemensamt styrda företag samt övriga företag som det finns ett ägarintresse i", "asset"),
        (1340, "Långfristiga fordringar hos intresseföretag och gemensamt styrda företag samt övriga företag som det finns ett ägarintresse i", "asset"),
        (1350, "Andra långfristiga värdepappersinnehav", "asset"),
        (1360, "Lån till delägare eller närstående, långfristig del", "asset"),
        (1370, "Uppskjuten skattefordran", "asset"),               // #
        (1380, "Andra långfristiga fordringar", "asset"),

        // 14 Lager, produkter i arbete och pågående arbeten
        (1410, "Lager av råvaror", "asset"),
        (1420, "Lager av tillsatsmaterial och förnödenheter", "asset"),
        (1440, "Produkter i arbete", "asset"),
        (1450, "Lager av färdiga varor", "asset"),
        (1460, "Lager av handelsvaror", "asset"),
        (1470, "Pågående arbeten", "asset"),
        (1480, "Förskott för varor och tjänster", "asset"),
        (1490, "Övriga lagertillgångar", "asset"),

        // 15 Kundfordringar
        (1510, "Kundfordringar", "asset"),
        (1520, "Växelfordringar", "asset"),
        (1530, "Kontraktsfordringar", "asset"),
        (1550, "Konsignationsfordringar", "asset"),
        (1560, "Kundfordringar hos koncernföretag", "asset"),
        (1570, "Kundfordringar hos intresseföretag, gemensamt styrda företag och övriga företag som det finns ett ägarintresse i", "asset"),

        // 16 Övriga kortfristiga fordringar
        (1610, "Kortfristiga fordringar hos anställda", "asset"),
        (1620, "Upparbetad men ej fakturerad intäkt", "asset"),
        (1630, "Avräkning för skatter och avgifter (skattekonto)", "asset"),
        (1640, "Skattefordringar", "asset"),
        (1650, "Momsfordran", "asset"),
        (1660, "Kortfristiga fordringar hos koncernföretag", "asset"),
        (1670, "Kortfristiga fordringar hos intresseföretag, gemensamt styrda företag och övriga företag som det finns ett ägarintresse i", "asset"),
        (1680, "Andra kortfristiga fordringar", "asset"),
        (1690, "Fordringar för tecknat men ej inbetalt aktiekapital", "asset"),

        // 17 Förutbetalda kostnader och upplupna intäkter
        (1710, "Förutbetalda hyreskostnader", "asset"),
        (1720, "Förutbetalda leasingavgifter", "asset"),
        (1730, "Förutbetalda försäkringspremier", "asset"),
        (1740, "Förutbetalda räntekostnader", "asset"),
        (1750, "Upplupna hyresintäkter", "asset"),
        (1760, "Upplupna ränteintäkter", "asset"),
        (1770, "Tillgångar av kostnadsnatur", "asset"),
        (1780, "Upplupna avtalsintäkter", "asset"),
        (1790, "Övriga förutbetalda kostnader och upplupna intäkter", "asset"),

        // 18 Kortfristiga placeringar
        (1810, "Andelar i börsnoterade företag", "asset"),
        (1820, "Obligationer", "asset"),
        (1830, "Konvertibla skuldebrev", "asset"),
        (1860, "Andelar i koncernföretag, kortfristigt", "asset"),
        (1880, "Andra kortfristiga placeringar", "asset"),
        (1890, "Nedskrivning av kortfristiga placeringar", "asset"),

        // 19 Kassa och bank
        (1910, "Kassa", "asset"),
        (1920, "PlusGiro", "asset"),
        (1930, "Företagskonto", "asset"),
        (1940, "Övriga bankkonton", "asset"),
        (1950, "Bankcertifikat", "asset"),
        (1960, "Koncernkonto moderföretag", "asset"),
        (1970, "Särskilda bankkonton", "asset"),
        (1980, "Valutakonton", "asset"),
        (1990, "Redovisningsmedel", "asset"),

        // ═══════════════════════════════════════════════════════════════
        // KONTOKLASS 2 — EGET KAPITAL OCH SKULDER
        // ═══════════════════════════════════════════════════════════════

        // 20 Eget kapital
        (2010, "Eget kapital", "equity"),
        (2020, "Eget kapital", "equity"),
        (2030, "Eget kapital", "equity"),
        (2040, "Eget kapital", "equity"),
        (2050, "Avsättning till expansionsfond", "equity"),
        (2060, "Eget kapital i ideella föreningar, stiftelser och registrerade trossamfund", "equity"),
        (2070, "Ändamålsbestämda medel", "equity"),
        (2080, "Bundet eget kapital", "equity"),
        (2090, "Fritt eget kapital", "equity"),

        // 20 Underkonton – vanliga för aktiebolag
        (2081, "Aktiekapital", "equity"),
        (2082, "Ej registrerat aktiekapital", "equity"),
        (2083, "Medlemsinsatser", "equity"),
        (2084, "Förlagsinsatser", "equity"),
        (2085, "Uppskrivningsfond", "equity"),
        (2086, "Reservfond", "equity"),
        (2087, "Bunden överkursfond", "equity"),
        (2088, "Fond för yttre underhåll", "equity"),
        (2089, "Fond för utvecklingsutgifter", "equity"),
        (2091, "Balanserad vinst eller förlust", "equity"),
        (2093, "Erhållna aktieägartillskott", "equity"),
        (2094, "Egna aktier", "equity"),
        (2095, "Fusionsresultat", "equity"),
        (2097, "Fri överkursfond", "equity"),
        (2098, "Vinst eller förlust från föregående år", "equity"),
        (2099, "Årets resultat", "equity"),

        // 21 Obeskattade reserver
        (2110, "Periodiseringsfonder", "liability"),
        (2120, "Periodiseringsfond 2020", "liability"),
        (2130, "Periodiseringsfond 2020 – nr 2", "liability"),
        (2150, "Ackumulerade överavskrivningar", "liability"),
        (2160, "Ersättningsfond", "liability"),
        (2190, "Övriga obeskattade reserver", "liability"),

        // 22 Avsättningar
        (2210, "Avsättningar för pensioner enligt tryggandelagen", "liability"),
        (2220, "Avsättningar för garantier", "liability"),
        (2230, "Övriga avsättningar för pensioner och liknande förpliktelser", "liability"),
        (2240, "Avsättningar för uppskjutna skatter", "liability"), // #
        (2250, "Övriga avsättningar för skatter", "liability"),
        (2290, "Övriga avsättningar", "liability"),

        // 23 Långfristiga skulder
        (2310, "Obligations- och förlagslån", "liability"),
        (2320, "Konvertibla lån och liknande", "liability"),
        (2330, "Kontokredit", "liability"),
        (2340, "Byggnadskreditiv", "liability"),
        (2350, "Andra långfristiga skulder till kreditinstitut", "liability"),
        (2360, "Långfristiga skulder till koncernföretag", "liability"),
        (2370, "Långfristiga skulder till intresseföretag, gemensamt styrda företag och övriga företag som det finns ett ägarintresse i", "liability"),
        (2390, "Övriga långfristiga skulder", "liability"),

        // 24 Kortfristiga skulder till kreditinstitut, kunder och leverantörer
        (2410, "Andra kortfristiga låneskulder till kreditinstitut", "liability"),
        (2420, "Förskott från kunder", "liability"),
        (2430, "Pågående arbeten", "liability"),
        (2440, "Leverantörsskulder", "liability"),
        (2450, "Fakturerad men ej upparbetad intäkt", "liability"),
        (2460, "Leverantörsskulder till koncernföretag", "liability"),
        (2470, "Leverantörsskulder till intresseföretag, gemensamt styrda företag och övriga företag som det finns ett ägarintresse i", "liability"),
        (2480, "Kontokredit, kortfristig", "liability"),
        (2490, "Övriga kortfristiga skulder till kreditinstitut, kunder och leverantörer", "liability"),

        // 25 Skatteskulder
        (2510, "Skatteskulder", "liability"),

        // 26 Moms och punktskatter
        (2610, "Utgående moms, 25 %", "liability"),
        (2620, "Utgående moms, 12 %", "liability"),
        (2630, "Utgående moms, 6 %", "liability"),
        (2640, "Ingående moms", "liability"),
        (2650, "Redovisningskonto för moms", "liability"),
        (2660, "Punktskatter", "liability"),
        (2670, "Utgående moms på försäljning inom EU, OSS", "liability"),

        // 27 Personalens skatter, avgifter och löneavdrag
        (2710, "Personalskatt", "liability"),
        (2730, "Lagstadgade sociala avgifter och särskild löneskatt", "liability"),
        (2740, "Avtalade sociala avgifter", "liability"),
        (2750, "Utmätning i lön m.m.", "liability"),
        (2760, "Semestermedel", "liability"),
        (2790, "Övriga löneavdrag", "liability"),

        // 28 Övriga kortfristiga skulder
        (2810, "Avräkning för factoring och belånade kontraktsfordringar", "liability"),
        (2820, "Kortfristiga skulder till anställda", "liability"),
        (2830, "Avräkning för annans räkning", "liability"),
        (2840, "Kortfristiga låneskulder", "liability"),
        (2850, "Avräkning för skatter och avgifter (skattekonto)", "liability"),
        (2860, "Kortfristiga skulder till koncernföretag", "liability"),
        (2870, "Kortfristiga skulder till intresseföretag, gemensamt styrda företag och övriga företag som det finns ett ägarintresse i", "liability"),
        (2880, "Skuld erhållna bidrag", "liability"),
        (2890, "Övriga kortfristiga skulder", "liability"),

        // 29 Upplupna kostnader och förutbetalda intäkter
        (2910, "Upplupna löner", "liability"),
        (2920, "Upplupna semesterlöner", "liability"),
        (2930, "Upplupna pensionskostnader", "liability"),
        (2940, "Upplupna lagstadgade sociala och andra avgifter", "liability"),
        (2950, "Upplupna avtalade sociala avgifter", "liability"),
        (2960, "Upplupna räntekostnader", "liability"),
        (2970, "Förutbetalda intäkter", "liability"),
        (2980, "Upplupna avtalskostnader", "liability"),
        (2990, "Övriga upplupna kostnader och förutbetalda intäkter", "liability"),

        // ═══════════════════════════════════════════════════════════════
        // KONTOKLASS 3 — RÖRELSEINTÄKTER (Revenue)
        // ═══════════════════════════════════════════════════════════════

        // 30 Huvudintäkter
        (3000, "Försäljning inom Sverige", "revenue"),
        (3001, "Försäljning inom Sverige, 25 % moms", "revenue"),
        (3002, "Försäljning inom Sverige, 12 % moms", "revenue"),
        (3003, "Försäljning inom Sverige, 6 % moms", "revenue"),
        (3004, "Försäljning inom Sverige, momsfri", "revenue"),
        (3100, "Försäljning av varor utanför Sverige", "revenue"),
        (3200, "Försäljning VMB och omvänd moms", "revenue"),
        (3300, "Försäljning av tjänster utanför Sverige", "revenue"),
        (3400, "Försäljning, egna uttag", "revenue"),

        // 35 Fakturerade kostnader
        (3500, "Fakturerade kostnader (gruppkonto)", "revenue"),
        (3510, "Fakturerat emballage", "revenue"),
        (3520, "Fakturerade frakter", "revenue"),
        (3530, "Fakturerade tull- och speditionskostnader m.m.", "revenue"),
        (3540, "Faktureringsavgifter", "revenue"),
        (3550, "Fakturerade resekostnader", "revenue"),
        (3560, "Fakturerade kostnader till koncernföretag", "revenue"),
        (3570, "Fakturerade kostnader till intresseföretag, gemensamt styrda företag och övriga företag som det finns ett ägarintresse i", "revenue"),
        (3590, "Övriga fakturerade kostnader", "revenue"),

        // 36 Rörelsens sidointäkter
        (3600, "Rörelsens sidointäkter (gruppkonto)", "revenue"),
        (3610, "Försäljning av material", "revenue"),
        (3620, "Tillfällig uthyrning av personal", "revenue"),
        (3630, "Tillfällig uthyrning av transportmedel", "revenue"),
        (3670, "Intäkter från värdepapper", "revenue"),
        (3680, "Management fees", "revenue"),
        (3690, "Övriga sidointäkter", "revenue"),

        // 37 Intäktskorrigeringar
        (3700, "Intäktskorrigeringar (gruppkonto)", "revenue"),
        (3710, "Ofördelade intäktsreduktioner", "revenue"),
        (3730, "Lämnade rabatter", "revenue"),
        (3740, "Öres- och kronutjämning", "revenue"),
        (3750, "Punktskatter", "revenue"),
        (3790, "Övriga intäktskorrigeringar", "revenue"),

        // 38 Aktiverat arbete för egen räkning
        (3800, "Aktiverat arbete för egen räkning (gruppkonto)", "revenue"),
        (3840, "Aktiverat arbete (material)", "revenue"),
        (3850, "Aktiverat arbete (omkostnader)", "revenue"),
        (3870, "Aktiverat arbete (personal)", "revenue"),

        // 39 Övriga rörelseintäkter
        (3900, "Övriga rörelseintäkter (gruppkonto)", "revenue"),
        (3910, "Hyres- och arrendeintäkter", "revenue"),
        (3920, "Provisionsintäkter, licensintäkter och royalties", "revenue"),
        (3940, "Orealiserade negativa/positiva värdeförändringar på säkringsinstrument", "revenue"), // #
        (3950, "Återvunna, tidigare avskrivna kundfordringar", "revenue"),
        (3960, "Valutakursvinster på fordringar och skulder av rörelsekaraktär", "revenue"),
        (3970, "Vinst vid avyttring av immateriella och materiella anläggningstillgångar", "revenue"),
        (3980, "Erhållna offentliga bidrag", "revenue"),
        (3990, "Övriga ersättningar, bidrag och intäkter", "revenue"),

        // ═══════════════════════════════════════════════════════════════
        // KONTOKLASS 4 — MATERIAL OCH VARUKOSTNADER (Cost of goods)
        // ═══════════════════════════════════════════════════════════════

        // 40 Inköp av handelsvaror
        (4000, "Inköp av handelsvaror (gruppkonto)", "expense"),
        (4010, "Inköp av handelsvaror i Sverige", "expense"),
        (4060, "Inköp av handelsvaror i Sverige, omvänd betalningsskyldighet", "expense"),
        (4070, "Inköp av handelsvaror från annat EU-land", "expense"),
        (4080, "Import av handelsvaror", "expense"),
        (4090, "Erhållna rabatter (Handelsvaror)", "expense"),

        // 42 Sålda handelsvaror VMB
        (4200, "Sålda handelsvaror VMB (gruppkonto)", "expense"),
        (4210, "Sålda handelsvaror VMB", "expense"),

        // 43 Inköp av råvaror och material i Sverige
        (4300, "Inköp av råvaror och material i Sverige (gruppkonto)", "expense"),
        (4310, "Inköp av råvaror och material i Sverige", "expense"),

        // 44 Inköp av råvaror och material, tjänster m.m. i Sverige, omvänd betalningsskyldighet
        (4400, "Inköp av råvaror och material, tjänster m.m. i Sverige, omvänd betalningsskyldighet (gruppkonto)", "expense"),
        (4410, "Inköp av råvaror och material i Sverige, omvänd betalningsskyldighet", "expense"),
        (4420, "Inköp av tjänster i Sverige, omvänd betalningsskyldighet", "expense"),

        // 45 Inköp av råvaror och material, tjänster m.m. från utlandet
        (4500, "Inköp av råvaror och material, tjänster m.m. från utlandet (gruppkonto)", "expense"),
        (4510, "Inköp av råvaror och material från annat EU-land", "expense"),
        (4530, "Inköp av tjänster m.m. från utlandet", "expense"),
        (4540, "Import av råvaror och material", "expense"),

        // 46 Inköp av tjänster, underentreprenader och legoarbeten i Sverige
        (4600, "Inköp av tjänster, underentreprenader och legoarbeten i Sverige (gruppkonto)", "expense"),
        (4610, "Inköp av tjänster och underentreprenader", "expense"),
        (4670, "Inköp av legoarbeten", "expense"),

        // 47 Reduktion av inköpspriser
        (4700, "Reduktion av inköpspriser (gruppkonto)", "expense"),
        (4730, "Erhållna rabatter (Råvaror och förnödenheter)", "expense"),

        // 48 Andra produktionskostnader
        (4800, "Andra produktionskostnader (gruppkonto)", "expense"),
        (4810, "Kostnader för energi (Råvaror och förnödenheter)", "expense"),
        (4820, "Kostnader för drivmedel (Råvaror och förnödenheter)", "expense"),
        (4830, "Kostnader för resor (Råvaror och förnödenheter)", "expense"),
        (4840, "Kostnader för hyra av utrustning (Råvaror och förnödenheter)", "expense"),
        (4890, "Övriga produktionskostnader (Råvaror och förnödenheter)", "expense"),

        // 49 Förändring av lager, produkter i arbete och pågående arbeten
        (4900, "Förändring av lager (gruppkonto)", "expense"),
        (4910, "Förändring av lager av råvaror", "expense"),
        (4920, "Förändring av lager av tillsatsmaterial och förnödenheter", "expense"),
        (4940, "Förändring av produkter i arbete", "expense"),
        (4950, "Förändring av lager av färdiga varor", "expense"),
        (4960, "Förändring av lager av handelsvaror", "expense"),
        (4970, "Förändring av pågående arbeten, nedlagda kostnader", "expense"),
        (4980, "Förändring av lager av värdepapper (Handelsvaror)", "expense"),

        // ═══════════════════════════════════════════════════════════════
        // KONTOKLASS 5 — ÖVRIGA EXTERNA KOSTNADER
        // ═══════════════════════════════════════════════════════════════

        // 50 Lokalkostnader
        (5000, "Lokalkostnader (gruppkonto)", "expense"),
        (5010, "Lokalhyra", "expense"),
        (5020, "El", "expense"),
        (5030, "Värme", "expense"),
        (5040, "Vatten och avlopp", "expense"),
        (5050, "Lokaltillbehör", "expense"),
        (5060, "Städning och renhållning", "expense"),
        (5070, "Reparation och underhåll av lokaler", "expense"),
        (5090, "Övriga lokalkostnader", "expense"),

        // 51 Fastighetskostnader
        (5100, "Fastighetskostnader (gruppkonto)", "expense"),
        (5110, "Tomträttsavgäld/arrende", "expense"),
        (5120, "El", "expense"),
        (5130, "Värme", "expense"),
        (5140, "Vatten och avlopp", "expense"),
        (5160, "Städning och renhållning", "expense"),
        (5170, "Reparation och underhåll av fastighet", "expense"),
        (5190, "Övriga fastighetskostnader", "expense"),

        // 52 Hyra av anläggningstillgångar
        (5200, "Hyra av anläggningstillgångar (gruppkonto)", "expense"),
        (5210, "Hyra av maskiner och andra tekniska anläggningar, ej datorer och fordon", "expense"),
        (5220, "Hyra av inventarier och verktyg, ej datorer och fordon", "expense"),
        (5250, "Hyra av datorer", "expense"),
        (5290, "Hyra av övriga anläggningstillgångar, ej datorer och fordon", "expense"),

        // 53 Energikostnader för drift (ej råvaror och förnödenheter)
        (5300, "Energikostnader för drift (gruppkonto)", "expense"),
        (5310, "El för drift (ej råvaror och förnödenheter)", "expense"),
        (5320, "Gas för drift (ej råvaror och förnödenheter)", "expense"),
        (5330, "Eldningsolja för drift (ej råvaror och förnödenheter)", "expense"),
        (5340, "Stenkol och koks för drift (ej råvaror och förnödenheter)", "expense"),
        (5350, "Torv, träkol, ved, m.m. för drift (ej råvaror och förnödenheter)", "expense"),
        (5360, "Bensin, fotogen och motorbrännolja för drift (ej råvaror och förnödenheter)", "expense"),
        (5370, "Fjärrvärme, kyla och ånga för drift (ej råvaror och förnödenheter)", "expense"),
        (5380, "Vatten för drift (ej råvaror och förnödenheter)", "expense"),
        (5390, "Övriga energikostnader för drift (ej råvaror och förnödenheter)", "expense"),

        // 54 Förbrukningsinventarier och förbrukningsmaterial
        (5400, "Förbrukningsinventarier och förbrukningsmaterial (gruppkonto)", "expense"),
        (5410, "Förbrukningsinventarier", "expense"),
        (5420, "Programvaror", "expense"),
        (5430, "Transportinventarier", "expense"),
        (5440, "Förbrukningsemballage", "expense"),
        (5460, "Förbrukningsmaterial", "expense"),
        (5480, "Arbetskläder och skyddsmaterial", "expense"),

        // 55 Reparation och underhåll
        (5500, "Reparation och underhåll (gruppkonto)", "expense"),
        (5510, "Reparation och underhåll av maskiner och andra tekniska anläggningar", "expense"),
        (5520, "Reparation och underhåll av inventarier, verktyg och datorer m.m.", "expense"),
        (5530, "Reparation och underhåll byggnads- och markinventarier", "expense"),
        (5550, "Reparation och underhåll av förbrukningsinventarier", "expense"),
        (5580, "Underhåll och tvätt av arbetskläder", "expense"),
        (5590, "Övriga kostnader för reparation och underhåll", "expense"),

        // 56 Kostnader för transportmedel
        (5600, "Kostnader för transportmedel (gruppkonto)", "expense"),
        (5610, "Personbils- och mc-kostnader, m.m.", "expense"),
        (5620, "Lastbils- och busskostnader, m.m.", "expense"),
        (5630, "Truckkostnader", "expense"),
        (5640, "Kostnader för arbetsmaskiner", "expense"),
        (5650, "Traktorkostnader", "expense"),
        (5670, "Kostnader för fartyg och luftfartyg", "expense"),
        (5680, "Kostnader för rälsfordon", "expense"),
        (5690, "Kostnader för övriga transportmedel", "expense"),

        // 57 Frakter och transporter
        (5700, "Frakter och transporter (gruppkonto)", "expense"),
        (5710, "Frakter och försäkringar vid varudistribution", "expense"),
        (5720, "Tull- och speditionskostnader m.m.", "expense"),
        (5730, "Arbetstransporter", "expense"),
        (5790, "Övriga kostnader för frakter och transporter", "expense"),

        // 58 Resekostnader
        (5800, "Resekostnader (gruppkonto)", "expense"),
        (5810, "Biljetter", "expense"),
        (5820, "Hyrbilskostnader", "expense"),
        (5830, "Kost och logi", "expense"),
        (5890, "Övriga resekostnader", "expense"),

        // 59 Reklam och PR
        (5900, "Reklam och PR (gruppkonto)", "expense"),
        (5910, "Annonsering", "expense"),
        (5920, "Utomhus- och trafikreklam", "expense"),
        (5930, "Reklamtrycksaker och direktreklam", "expense"),
        (5940, "Utställningar och mässor", "expense"),
        (5950, "Butiksreklam och återförsäljarreklam", "expense"),
        (5960, "Varuprover, reklamgåvor, presentreklam och tävlingar", "expense"),
        (5970, "Film-, radio-, TV- och Internetreklam", "expense"),
        (5980, "Sponsring", "expense"),
        (5990, "Övriga kostnader för reklam och PR", "expense"),

        // ═══════════════════════════════════════════════════════════════
        // KONTOKLASS 6 — ÖVRIGA EXTERNA KOSTNADER (forts.)
        // ═══════════════════════════════════════════════════════════════

        // 60 Övriga försäljningskostnader
        (6000, "Övriga försäljningskostnader (gruppkonto)", "expense"),
        (6010, "Kataloger, prislistor m.m.", "expense"),
        (6020, "Egna facktidskrifter", "expense"),
        (6030, "Speciella orderkostnader", "expense"),
        (6040, "Kontokortsavgifter", "expense"),
        (6050, "Försäljningsprovisioner", "expense"),
        (6060, "Kreditförsäljningskostnader", "expense"),
        (6070, "Representation", "expense"),
        (6071, "Representation, avdragsgill", "expense"),
        (6072, "Representation, ej avdragsgill", "expense"),
        (6080, "Bankgarantier", "expense"),
        (6090, "Övriga försäljningskostnader", "expense"),

        // 61 Kontorsmaterial och trycksaker
        (6100, "Kontorsmateriel och trycksaker (gruppkonto)", "expense"),
        (6110, "Kontorsmateriel", "expense"),
        (6150, "Trycksaker", "expense"),

        // 62 Tele, data och post
        (6200, "Tele, data och post (gruppkonto)", "expense"),
        (6210, "Telekommunikation", "expense"),
        (6211, "Fast telefoni", "expense"),
        (6212, "Mobiltelefon", "expense"),
        (6230, "Datakommunikation", "expense"),
        (6250, "Porto", "expense"),
        (6290, "Övriga tele-, data- och postkostnader", "expense"),

        // 63 Företagsförsäkringar och övriga riskkostnader
        (6300, "Företagsförsäkringar och övriga riskkostnader (gruppkonto)", "expense"),
        (6310, "Företagsförsäkringar", "expense"),
        (6320, "Självrisker vid skada", "expense"),
        (6330, "Förluster i pågående arbeten", "expense"),
        (6340, "Lämnade skadestånd", "expense"),
        (6350, "Förluster på kundfordringar", "expense"),
        (6360, "Garantikostnader", "expense"),
        (6370, "Kostnader för bevakning och larm", "expense"),
        (6380, "Förluster på övriga kortfristiga fordringar", "expense"),
        (6390, "Övriga riskkostnader", "expense"),

        // 64 Förvaltningskostnader
        (6400, "Förvaltningskostnader (gruppkonto)", "expense"),
        (6420, "Ersättningar till revisor", "expense"),
        (6430, "Management fees", "expense"),
        (6440, "Årsredovisning och delårsrapporter", "expense"),
        (6450, "Bolagsstämma/års- eller föreningsstämma", "expense"),
        (6490, "Övriga förvaltningskostnader", "expense"),

        // 65 Övriga externa tjänster
        (6500, "Övriga externa tjänster (gruppkonto)", "expense"),
        (6510, "Mätningskostnader", "expense"),
        (6520, "Ritnings- och kopieringskostnader", "expense"),
        (6530, "Redovisningstjänster", "expense"),
        (6540, "IT-tjänster", "expense"),
        (6550, "Konsultarvoden", "expense"),
        (6560, "Serviceavgifter till branschorganisationer", "expense"),
        (6570, "Bankkostnader", "expense"),
        (6580, "Advokat- och rättegångskostnader", "expense"),
        (6590, "Övriga externa tjänster", "expense"),

        // 67 Särskilt för ideella föreningar och stiftelser
        (6700, "Särskilt för ideella föreningar och stiftelser (gruppkonto)", "expense"),
        (6710, "Lämnade bidrag", "expense"),

        // 68 Inhyrd personal
        (6800, "Inhyrd personal (gruppkonto)", "expense"),
        (6810, "Inhyrd produktionspersonal", "expense"),
        (6820, "Inhyrd lagerpersonal", "expense"),
        (6830, "Inhyrd transportpersonal", "expense"),
        (6840, "Inhyrd kontors- och ekonomipersonal", "expense"),
        (6850, "Inhyrd IT-personal", "expense"),
        (6860, "Inhyrd marknads- och försäljningspersonal", "expense"),
        (6870, "Inhyrd restaurang- och butikspersonal", "expense"),
        (6880, "Inhyrda företagsledare", "expense"),
        (6890, "Övrig inhyrd personal", "expense"),

        // 69 Övriga externa kostnader
        (6900, "Övriga externa kostnader (gruppkonto)", "expense"),
        (6910, "Licensavgifter och royalties", "expense"),
        (6920, "Kostnader för egna patent", "expense"),
        (6930, "Kostnader för varumärken m.m.", "expense"),
        (6940, "Kontroll-, provnings- och stämpelavgifter", "expense"),
        (6950, "Tillsynsavgifter myndigheter", "expense"),
        (6970, "Tidningar, facklitteratur, m.m.", "expense"),
        (6980, "Föreningsavgifter", "expense"),
        (6990, "Övriga externa kostnader", "expense"),

        // ═══════════════════════════════════════════════════════════════
        // KONTOKLASS 7 — PERSONALKOSTNADER
        // ═══════════════════════════════════════════════════════════════

        // 70 Löner till kollektivanställda
        (7000, "Löner till kollektivanställda (gruppkonto)", "expense"),
        (7010, "Löner till kollektivanställda", "expense"),
        (7030, "Löner till kollektivanställda (utlandsanställda)", "expense"),
        (7080, "Löner till kollektivanställda för ej arbetad tid", "expense"),
        (7090, "Förändring av semesterlöneskuld", "expense"),

        // 72 Löner till tjänstemän och företagsledare
        (7200, "Löner till tjänstemän och företagsledare (gruppkonto)", "expense"),
        (7210, "Löner till tjänstemän", "expense"),
        (7220, "Löner till företagsledare", "expense"),
        (7230, "Löner till tjänstemän och ftgsledare (utlandsanställda)", "expense"),
        (7240, "Styrelsearvoden", "expense"),
        (7280, "Löner till tjänstemän och företagsledare för ej arbetad tid", "expense"),
        (7290, "Förändring av semesterlöneskuld", "expense"),

        // 73 Kostnadsersättningar och förmåner
        (7300, "Kostnadsersättningar och förmåner (gruppkonto)", "expense"),
        (7310, "Kontanta extraersättningar", "expense"),
        (7320, "Traktamenten vid tjänsteresa", "expense"),
        (7330, "Bilersättningar", "expense"),
        (7350, "Ersättningar för föreskrivna arbetskläder", "expense"),
        (7370, "Representationsersättningar", "expense"),
        (7380, "Kostnader för förmåner till anställda", "expense"),
        (7390, "Övriga kostnadsersättningar och förmåner", "expense"),

        // 74 Pensionskostnader
        (7400, "Pensionskostnader (gruppkonto)", "expense"),
        (7410, "Pensionsförsäkringspremier", "expense"),
        (7420, "Förändring av pensionsskuld", "expense"),
        (7430, "Avdrag för räntedel i pensionskostnad", "expense"),
        (7440, "Förändring av pensionsstiftelsekapital", "expense"),
        (7460, "Pensionsutbetalningar", "expense"),
        (7470, "Förvaltnings- och kreditförsäkringsavgifter", "expense"),
        (7490, "Övriga pensionskostnader", "expense"),

        // 75 Sociala och andra avgifter enligt lag och avtal
        (7500, "Sociala och andra avgifter enligt lag och avtal (gruppkonto)", "expense"),
        (7510, "Arbetsgivaravgifter 31,42 %", "expense"),
        (7530, "Särskild löneskatt", "expense"),
        (7550, "Avkastningsskatt på pensionsmedel", "expense"),
        (7570, "Premier för arbetsmarknadsförsäkringar", "expense"),
        (7580, "Gruppförsäkringspremier", "expense"),
        (7590, "Övriga sociala och andra avgifter enligt lag och avtal", "expense"),

        // 76 Övriga personalkostnader
        (7600, "Övriga personalkostnader (gruppkonto)", "expense"),
        (7610, "Utbildning", "expense"),
        (7620, "Sjuk- och hälsovård", "expense"),
        (7630, "Personalrepresentation", "expense"),
        (7650, "Sjuklöneförsäkring", "expense"),
        (7670, "Förändring av personalstiftelsekapital", "expense"),
        (7690, "Övriga personalkostnader", "expense"),

        // 77 Nedskrivningar och återföring av nedskrivningar
        (7710, "Nedskrivningar av immateriella anläggningstillgångar", "expense"),
        (7720, "Nedskrivningar av byggnader och mark", "expense"),
        (7730, "Nedskrivningar av maskiner respektive inventarier", "expense"),
        (7740, "Nedskrivningar av vissa omsättningstillgångar", "expense"),
        (7760, "Återföring av nedskrivningar av immateriella anläggningstillgångar", "expense"),
        (7770, "Återföring av nedskrivningar av byggnader och mark", "expense"),
        (7780, "Återföring av nedskrivningar av maskiner respektive inventarier", "expense"),
        (7790, "Återföring av nedskrivningar av vissa omsättningstillgångar", "expense"),

        // 78 Avskrivningar enligt plan
        (7810, "Avskrivningar på immateriella anläggningstillgångar", "expense"),
        (7820, "Avskrivningar på byggnader och markanläggningar", "expense"),
        (7830, "Avskrivningar på maskiner respektive inventarier", "expense"),
        (7840, "Avskrivningar på förbättringsutgifter på annans fastighet", "expense"),

        // 79 Övriga rörelsekostnader
        (7940, "Orealiserade positiva/negativa värdeförändringar på säkringsinstrument", "expense"), // #
        (7960, "Valutakursförluster på fordringar och skulder av rörelsekaraktär", "expense"),
        (7970, "Förlust vid avyttring av immateriella och materiella anläggningstillgångar", "expense"),
        (7990, "Övriga rörelsekostnader", "expense"),

        // ═══════════════════════════════════════════════════════════════
        // KONTOKLASS 8 — FINANSIELLA POSTER, BOKSLUTSDISPOSITIONER, SKATT
        // ═══════════════════════════════════════════════════════════════

        // 80 Resultat från andelar i koncernföretag
        (8010, "Utdelning på andelar i koncernföretag", "revenue"),
        (8020, "Resultat vid försäljning av andelar i koncernföretag", "revenue"),
        (8030, "Resultatandelar från handelsbolag (dotterföretag)", "revenue"),
        (8070, "Nedskrivningar av andelar i och långfristiga fordringar hos koncernföretag", "expense"),
        (8080, "Återföringar av nedskrivningar av andelar i och långfristiga fordringar hos koncernföretag", "revenue"),

        // 81 Resultat från andelar i intresseföretag och gemensamt styrda företag samt övriga företag som det finns ett ägarintresse i
        (8110, "Utdelningar på andelar i intresseföretag och gemensamt styrda företag samt övriga företag som det finns ett ägarintresse i", "revenue"),
        (8120, "Resultat vid försäljning av andelar i intresseföretag och gemensamt styrda företag samt övriga företag som det finns ett ägarintresse i", "revenue"),
        (8130, "Resultatandelar från handelsbolag (intresseföretag och gemensamt styrda företag samt övriga företag som det finns ett ägarintresse i)", "revenue"),
        (8170, "Nedskrivningar av andelar i och långfristiga fordringar hos intresseföretag och gemensamt styrda företag samt övriga företag som det finns ett ägarintresse i", "expense"),
        (8180, "Återföringar av nedskrivningar av andelar i och långfristiga fordringar hos intresseföretag och gemensamt styrda företag samt övriga företag som det finns ett ägarintresse i", "revenue"),

        // 82 Resultat från övriga värdepapper och långfristiga fordringar (anläggningstillgångar)
        (8210, "Utdelningar på andelar i andra företag", "revenue"),
        (8220, "Resultat vid försäljning av värdepapper i och långfristiga fordringar hos andra företag", "revenue"),
        (8230, "Valutakursdifferenser på långfristiga fordringar", "revenue"),
        (8240, "Resultatandelar från handelsbolag (andra företag)", "revenue"),
        (8250, "Ränteintäkter från långfristiga fordringar hos och värdepapper i andra företag", "revenue"),
        (8260, "Ränteintäkter från långfristiga fordringar hos koncernföretag", "revenue"),
        (8270, "Nedskrivningar av innehav av andelar i och fordringar hos andra företag", "expense"),
        (8280, "Återföringar av nedskrivningar av andelar i och långfristiga fordringar hos andra företag", "revenue"),
        (8290, "Värdering till verkligt värde, anläggningstillgångar", "revenue"), // #

        // 83 Övriga ränteintäkter och liknande resultatposter
        (8310, "Ränteintäkter från omsättningstillgångar", "revenue"),
        (8320, "Värdering till verkligt värde, omsättningstillgångar", "revenue"), // #
        (8330, "Valutakursdifferenser på kortfristiga fordringar och placeringar", "revenue"),
        (8340, "Utdelningar på kortfristiga placeringar", "revenue"),
        (8350, "Resultat vid försäljning av kortfristiga placeringar", "revenue"),
        (8360, "Övriga ränteintäkter från koncernföretag", "revenue"),
        (8370, "Nedskrivningar av kortfristiga placeringar", "expense"),
        (8380, "Återföringar av nedskrivningar av kortfristiga placeringar", "revenue"),
        (8390, "Övriga finansiella intäkter", "revenue"),

        // 84 Räntekostnader och liknande resultatposter
        (8400, "Räntekostnader (gruppkonto)", "expense"),
        (8410, "Räntekostnader för långfristiga skulder", "expense"),
        (8420, "Räntekostnader för kortfristiga skulder", "expense"),
        (8430, "Valutakursdifferenser på skulder", "expense"),
        (8440, "Erhållna räntebidrag", "revenue"),
        (8450, "Orealiserade värdeförändringar på skulder", "expense"), // #
        (8460, "Räntekostnader till koncernföretag", "expense"),
        (8480, "Aktiverade ränteutgifter", "expense"),                 // #
        (8490, "Övriga skuldrelaterade poster", "expense"),

        // 88 Bokslutsdispositioner
        (8810, "Förändring av periodiseringsfond", "expense"),
        (8820, "Mottagna koncernbidrag", "revenue"),
        (8830, "Lämnade koncernbidrag", "expense"),
        (8840, "Lämnade gottgörelser", "expense"),
        (8850, "Förändring av överavskrivningar", "expense"),
        (8860, "Förändring av ersättningsfond", "expense"),
        (8890, "Övriga bokslutsdispositioner", "expense"),

        // 89 Skatter och årets resultat
        (8910, "Skatt som belastar årets resultat", "expense"),
        (8920, "Skatt på grund av ändrad beskattning", "expense"),
        (8930, "Restituerad skatt", "revenue"),
        (8940, "Uppskjuten skatt", "expense"),                        // #
        (8980, "Övriga skatter", "expense"),
        (8990, "Resultat", "revenue"),
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
        (1040, "7202"), // Licenser
        (1050, "7202"), // Varumärken
        (1060, "7202"), // Hyresrätter
        (1070, "7203"), // Goodwill
        (1080, "7207"), // Pågående arbeten immateriella
        (1090, "7202"), // Övriga immateriella

        // Materiella anläggningstillgångar — Byggnader & mark
        (1110, "7210"), // Byggnader
        (1120, "7215"), // Förbättringsutgifter på annans fastighet
        (1130, "7211"), // Mark
        (1140, "7211"), // Tomter
        (1150, "7212"), // Markanläggningar
        (1180, "7216"), // Pågående nyanläggningar byggnader/mark

        // Materiella anläggningstillgångar — Maskiner & inventarier
        (1210, "7214"), // Maskiner och andra tekniska anläggningar
        (1220, "7215"), // Inventarier, verktyg och installationer
        (1230, "7214"), // Maskiner (fritt konto)
        (1240, "7214"), // Maskiner (fritt konto 2)
        (1250, "7215"), // Inventarier (fritt konto)
        (1260, "7215"), // Inventarier (fritt konto 2)
        (1280, "7216"), // Pågående nyanläggningar maskiner/inventarier
        (1290, "7215"), // Övriga materiella

        // Finansiella anläggningstillgångar
        (1310, "7220"), // Andelar i koncernföretag
        (1320, "7221"), // Långfristiga fordringar hos koncernföretag
        (1330, "7222"), // Andelar i intresseföretag m.fl.
        (1340, "7223"), // Långfristiga fordringar intresseföretag m.fl.
        (1350, "7224"), // Andra långfristiga värdepappersinnehav
        (1360, "7225"), // Lån till delägare
        (1370, "7225"), // Uppskjuten skattefordran
        (1380, "7225"), // Andra långfristiga fordringar

        // Varulager
        (1410, "7230"), // Lager av råvaror
        (1420, "7230"), // Lager av tillsatsmaterial
        (1440, "7230"), // Produkter i arbete
        (1450, "7230"), // Lager av färdiga varor
        (1460, "7230"), // Lager av handelsvaror
        (1470, "7234"), // Pågående arbeten
        (1480, "7235"), // Förskott
        (1490, "7230"), // Övriga lagertillgångar

        // Kortfristiga fordringar
        (1510, "7240"), // Kundfordringar
        (1520, "7240"), // Växelfordringar
        (1530, "7240"), // Kontraktsfordringar
        (1550, "7240"), // Konsignationsfordringar
        (1560, "7241"), // Kundfordringar koncernföretag
        (1570, "7242"), // Kundfordringar intresseföretag
        (1610, "7243"), // Fordringar hos anställda
        (1620, "7249"), // Upparbetad ej fakturerad intäkt
        (1630, "7248"), // Skattefordringar
        (1640, "7248"), // Skattefordringar
        (1650, "7248"), // Momsfordran
        (1660, "7244"), // Kortfr. fordringar koncernföretag
        (1670, "7245"), // Kortfr. fordringar intresseföretag
        (1680, "7249"), // Andra kortfristiga fordringar
        (1690, "7249"), // Fordringar ej inbetalt aktiekapital
        (1710, "7249"), // Förutbetalda kostnader
        (1720, "7249"), // Förutbetalda leasingavgifter
        (1730, "7249"), // Förutbetalda försäkringspremier
        (1740, "7249"), // Förutbetalda räntekostnader
        (1750, "7249"), // Upplupna hyresintäkter
        (1760, "7249"), // Upplupna ränteintäkter
        (1770, "7249"), // Tillgångar av kostnadsnatur
        (1780, "7249"), // Upplupna avtalsintäkter
        (1790, "7249"), // Övriga förutbetalda

        // Kortfristiga placeringar
        (1810, "7250"), // Andelar börsnoterade
        (1820, "7250"), // Obligationer
        (1830, "7250"), // Konvertibla skuldebrev
        (1860, "7250"), // Andelar koncernföretag kortfristigt
        (1880, "7250"), // Andra kortfristiga placeringar
        (1890, "7250"), // Nedskrivning kortfristiga placeringar

        // Kassa och bank
        (1910, "7260"), // Kassa
        (1920, "7260"), // PlusGiro
        (1930, "7260"), // Företagskonto
        (1940, "7260"), // Övriga bankkonton
        (1950, "7260"), // Bankcertifikat
        (1960, "7260"), // Koncernkonto moderföretag
        (1970, "7260"), // Särskilda bankkonton
        (1980, "7260"), // Valutakonton
        (1990, "7260"), // Redovisningsmedel

        // ═══ BALANCE SHEET — EQUITY & LIABILITIES ═══

        // Bundet eget kapital
        (2080, "7300"), // Bundet eget kapital
        (2081, "7300"), // Aktiekapital
        (2082, "7300"), // Ej registrerat aktiekapital
        (2085, "7301"), // Uppskrivningsfond
        (2086, "7303"), // Reservfond
        (2087, "7302"), // Bunden överkursfond

        // Fritt eget kapital
        (2090, "7310"), // Fritt eget kapital
        (2091, "7310"), // Balanserad vinst eller förlust
        (2097, "7310"), // Fri överkursfond
        (2098, "7310"), // Vinst/förlust föregående år
        (2099, "7312"), // Årets resultat

        // Obeskattade reserver
        (2110, "7320"), // Periodiseringsfonder
        (2120, "7320"), // Periodiseringsfond 2020
        (2130, "7320"), // Periodiseringsfond 2020 – nr 2
        (2150, "7321"), // Ackumulerade överavskrivningar
        (2160, "7322"), // Ersättningsfond
        (2190, "7329"), // Övriga obeskattade reserver

        // Avsättningar
        (2210, "7330"), // Avsättningar för pensioner
        (2220, "7332"), // Avsättningar för garantier
        (2230, "7330"), // Övriga avsättningar pensioner
        (2240, "7331"), // Avsättningar uppskjutna skatter
        (2250, "7332"), // Övriga avsättningar skatter
        (2290, "7332"), // Övriga avsättningar

        // Långfristiga skulder
        (2310, "7340"), // Obligations- och förlagslån
        (2320, "7340"), // Konvertibla lån
        (2330, "7340"), // Kontokredit
        (2340, "7340"), // Byggnadskreditiv
        (2350, "7340"), // Andra långfristiga skulder kreditinstitut
        (2360, "7341"), // Långfristiga skulder koncernföretag
        (2370, "7342"), // Långfristiga skulder intresseföretag
        (2390, "7349"), // Övriga långfristiga skulder

        // Kortfristiga skulder
        (2410, "7361"), // Kortfristiga låneskulder kreditinstitut
        (2420, "7369"), // Förskott från kunder
        (2430, "7369"), // Pågående arbeten
        (2440, "7360"), // Leverantörsskulder
        (2450, "7369"), // Fakturerad ej upparbetad intäkt
        (2460, "7364"), // Leverantörsskulder koncernföretag
        (2470, "7365"), // Leverantörsskulder intresseföretag
        (2480, "7361"), // Kontokredit kortfristig
        (2490, "7369"), // Övriga kortfristiga skulder
        (2510, "7362"), // Skatteskulder
        (2610, "7363"), // Utgående moms 25%
        (2620, "7363"), // Utgående moms 12%
        (2630, "7363"), // Utgående moms 6%
        (2640, "7363"), // Ingående moms
        (2650, "7363"), // Redovisningskonto moms
        (2660, "7363"), // Punktskatter
        (2670, "7363"), // Utgående moms EU/OSS
        (2710, "7365"), // Personalskatt
        (2730, "7365"), // Lagstadgade sociala avgifter
        (2740, "7365"), // Avtalade sociala avgifter
        (2790, "7369"), // Övriga löneavdrag
        (2810, "7369"), // Avräkning factoring
        (2820, "7366"), // Kortfristiga skulder anställda
        (2840, "7361"), // Kortfristiga låneskulder
        (2850, "7362"), // Avräkning skatter (skattekonto)
        (2860, "7367"), // Kortfristiga skulder koncernföretag
        (2870, "7367"), // Kortfristiga skulder intresseföretag
        (2880, "7369"), // Skuld erhållna bidrag
        (2890, "7369"), // Övriga kortfristiga skulder
        (2910, "7368"), // Upplupna löner
        (2920, "7368"), // Upplupna semesterlöner
        (2930, "7368"), // Upplupna pensionskostnader
        (2940, "7368"), // Upplupna lagstadgade sociala avgifter
        (2950, "7368"), // Upplupna avtalade sociala avgifter
        (2960, "7368"), // Upplupna räntekostnader
        (2970, "7368"), // Förutbetalda intäkter
        (2980, "7368"), // Upplupna avtalskostnader
        (2990, "7368"), // Övriga upplupna kostnader

        // ═══ INCOME STATEMENT (Resultaträkning) ═══

        // Nettoomsättning
        (3000, "7410"), // Försäljning inom Sverige
        (3001, "7410"), // Försäljning Sverige 25%
        (3002, "7410"), // Försäljning Sverige 12%
        (3003, "7410"), // Försäljning Sverige 6%
        (3004, "7410"), // Försäljning Sverige momsfri
        (3100, "7410"), // Försäljning varor utanför Sverige
        (3200, "7410"), // Försäljning VMB
        (3300, "7410"), // Försäljning tjänster utanför Sverige
        (3400, "7410"), // Försäljning egna uttag
        (3500, "7410"), // Fakturerade kostnader
        (3510, "7410"), // Fakturerat emballage
        (3520, "7410"), // Fakturerade frakter
        (3540, "7410"), // Faktureringsavgifter
        (3590, "7410"), // Övriga fakturerade kostnader

        // Övriga rörelseintäkter
        (3600, "7412"), // Rörelsens sidointäkter
        (3610, "7412"), // Försäljning av material
        (3690, "7412"), // Övriga sidointäkter
        (3730, "7410"), // Lämnade rabatter (minskar nettoomsättning)
        (3740, "7412"), // Öresavrundning
        (3900, "7412"), // Övriga rörelseintäkter
        (3960, "7412"), // Valutakursvinster
        (3970, "7412"), // Vinst avyttring anläggningstillgångar
        (3980, "7412"), // Erhållna offentliga bidrag
        (3990, "7412"), // Övriga ersättningar

        // Råvaror och förnödenheter + handelsvaror
        (4000, "7420"), // Inköp handelsvaror
        (4010, "7420"), // Inköp handelsvaror Sverige
        (4060, "7420"), // Inköp handelsvaror omvänd moms
        (4070, "7420"), // Inköp handelsvaror EU
        (4080, "7420"), // Import handelsvaror
        (4090, "7420"), // Erhållna rabatter
        (4300, "7420"), // Inköp råvaror Sverige
        (4310, "7420"), // Inköp råvaror Sverige
        (4410, "7420"), // Inköp råvaror omvänd moms
        (4420, "7420"), // Inköp tjänster omvänd moms
        (4510, "7420"), // Inköp råvaror EU
        (4530, "7420"), // Inköp tjänster utlandet
        (4540, "7420"), // Import råvaror
        (4600, "7420"), // Inköp tjänster/underentreprenader
        (4610, "7420"), // Inköp tjänster
        (4670, "7420"), // Inköp legoarbeten
        (4900, "7420"), // Förändring av lager
        (4910, "7420"), // Förändring lager råvaror
        (4960, "7420"), // Förändring lager handelsvaror

        // Övriga externa kostnader
        (5000, "7430"), // Lokalkostnader
        (5010, "7430"), // Lokalhyra
        (5020, "7430"), // El
        (5030, "7430"), // Värme
        (5040, "7430"), // Vatten och avlopp
        (5060, "7430"), // Städning
        (5090, "7430"), // Övriga lokalkostnader
        (5200, "7430"), // Hyra anläggningstillgångar
        (5210, "7430"), // Hyra maskiner
        (5220, "7430"), // Hyra inventarier
        (5250, "7430"), // Hyra datorer
        (5400, "7430"), // Förbrukningsinventarier
        (5410, "7430"), // Förbrukningsinventarier
        (5420, "7430"), // Programvaror
        (5460, "7430"), // Förbrukningsmaterial
        (5500, "7430"), // Reparation och underhåll
        (5510, "7430"), // Reparation maskiner
        (5520, "7430"), // Reparation inventarier
        (5600, "7430"), // Kostnader transportmedel
        (5610, "7430"), // Personbilskostnader
        (5800, "7430"), // Resekostnader
        (5810, "7430"), // Biljetter
        (5830, "7430"), // Kost och logi
        (5900, "7430"), // Reklam och PR
        (5910, "7430"), // Annonsering
        (6000, "7430"), // Övriga försäljningskostnader
        (6070, "7430"), // Representation
        (6071, "7430"), // Representation avdragsgill
        (6072, "7430"), // Representation ej avdragsgill
        (6100, "7430"), // Kontorsmateriel
        (6110, "7430"), // Kontorsmateriel
        (6200, "7430"), // Tele och post
        (6210, "7430"), // Telekommunikation
        (6211, "7430"), // Fast telefoni
        (6212, "7430"), // Mobiltelefon
        (6230, "7430"), // Datakommunikation
        (6250, "7430"), // Porto
        (6300, "7430"), // Företagsförsäkringar
        (6310, "7430"), // Företagsförsäkringar
        (6350, "7430"), // Förluster på kundfordringar
        (6400, "7430"), // Förvaltningskostnader
        (6420, "7430"), // Ersättningar till revisor
        (6500, "7430"), // Övriga externa tjänster
        (6530, "7430"), // Redovisningstjänster
        (6540, "7430"), // IT-tjänster
        (6550, "7430"), // Konsultarvoden
        (6570, "7430"), // Bankkostnader
        (6580, "7430"), // Advokat- och rättegångskostnader
        (6590, "7430"), // Övriga externa tjänster
        (6800, "7430"), // Inhyrd personal
        (6900, "7430"), // Övriga externa kostnader
        (6990, "7430"), // Övriga externa kostnader

        // Personalkostnader
        (7000, "7440"), // Löner kollektivanställda
        (7010, "7440"), // Löner kollektivanställda
        (7080, "7440"), // Löner ej arbetad tid
        (7090, "7440"), // Förändring semesterlöneskuld
        (7200, "7440"), // Löner tjänstemän
        (7210, "7440"), // Löner tjänstemän
        (7220, "7440"), // Löner företagsledare
        (7240, "7440"), // Styrelsearvoden
        (7280, "7440"), // Löner ej arbetad tid
        (7290, "7440"), // Förändring semesterlöneskuld
        (7300, "7440"), // Kostnadsersättningar
        (7310, "7440"), // Kontanta extraersättningar
        (7320, "7440"), // Traktamenten
        (7330, "7440"), // Bilersättningar
        (7380, "7440"), // Förmåner
        (7500, "7441"), // Sociala avgifter
        (7510, "7441"), // Arbetsgivaravgifter
        (7530, "7441"), // Särskild löneskatt
        (7550, "7441"), // Avkastningsskatt
        (7570, "7441"), // Arbetsmarknadsförsäkringar
        (7580, "7441"), // Gruppförsäkringspremier
        (7600, "7440"), // Övriga personalkostnader
        (7610, "7440"), // Utbildning
        (7630, "7440"), // Personalrepresentation
        (7690, "7440"), // Övriga personalkostnader

        // Avskrivningar och nedskrivningar
        (7710, "7450"), // Nedskrivningar immateriella
        (7720, "7450"), // Nedskrivningar byggnader
        (7730, "7450"), // Nedskrivningar maskiner/inventarier
        (7810, "7450"), // Avskrivningar immateriella
        (7820, "7450"), // Avskrivningar byggnader
        (7830, "7450"), // Avskrivningar maskiner/inventarier
        (7840, "7450"), // Avskrivningar förbättringsutgifter

        // Övriga rörelsekostnader
        (7960, "7459"), // Valutakursförluster
        (7970, "7459"), // Förlust avyttring anläggningstillgångar
        (7990, "7459"), // Övriga rörelsekostnader

        // Finansiella poster
        (8010, "7510"), // Utdelning koncernföretag
        (8110, "7510"), // Utdelning intresseföretag
        (8210, "7510"), // Utdelning andra företag
        (8250, "7510"), // Ränteintäkter långfristiga fordringar
        (8260, "7510"), // Ränteintäkter koncernföretag
        (8310, "7510"), // Ränteintäkter omsättningstillgångar
        (8330, "7510"), // Valutakursdifferenser kortfristiga
        (8340, "7510"), // Utdelningar kortfristiga
        (8350, "7510"), // Resultat försäljning kortfristiga
        (8360, "7510"), // Övriga ränteintäkter koncern
        (8390, "7510"), // Övriga finansiella intäkter

        (8070, "7511"), // Nedskrivningar andelar koncernföretag
        (8170, "7511"), // Nedskrivningar intresseföretag
        (8270, "7511"), // Nedskrivningar andra företag
        (8370, "7511"), // Nedskrivningar kortfristiga

        (8400, "7511"), // Räntekostnader
        (8410, "7511"), // Räntekostnader långfristiga
        (8420, "7511"), // Räntekostnader kortfristiga
        (8430, "7511"), // Valutakursdifferenser skulder
        (8460, "7511"), // Räntekostnader koncernföretag
        (8490, "7511"), // Övriga skuldrelaterade poster

        // Bokslutsdispositioner
        (8810, "7520"), // Förändring periodiseringsfonder
        (8820, "7521"), // Mottagna koncernbidrag
        (8830, "7522"), // Lämnade koncernbidrag
        (8850, "7521"), // Förändring överavskrivningar
        (8860, "7521"), // Förändring ersättningsfond
        (8890, "7521"), // Övriga bokslutsdispositioner

        // Skatt
        (8910, "7600"), // Skatt som belastar årets resultat
        (8920, "7600"), // Skatt på grund av ändrad beskattning
        (8930, "7600"), // Restituerad skatt
        (8980, "7600"), // Övriga skatter
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
