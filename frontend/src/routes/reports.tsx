import { createFileRoute } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { companiesApi, fiscalYearsApi, annualReportApi } from '@/api/queries';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Separator } from '@/components/ui/separator';
import { Badge } from '@/components/ui/badge';

interface ReportsSearch {
  companyId?: string;
  fyId?: string;
  tab?: 'income' | 'balance' | 'full';
}

export const Route = createFileRoute('/reports')({
  component: ReportsPage,
  validateSearch: (search: Record<string, unknown>): ReportsSearch => ({
    companyId: search.companyId as string | undefined,
    fyId: search.fyId as string | undefined,
    tab: (search.tab as ReportsSearch['tab']) || 'income',
  }),
});

function ReportsPage() {
  const { companyId, fyId, tab } = Route.useSearch();
  const navigate = Route.useNavigate();

  const { data: companies } = useQuery({
    queryKey: ['companies'],
    queryFn: companiesApi.list,
  });

  const activeCompanyId = companyId || companies?.[0]?.id;

  const { data: fiscalYears } = useQuery({
    queryKey: ['fiscal-years', activeCompanyId],
    queryFn: () => fiscalYearsApi.list(activeCompanyId!),
    enabled: !!activeCompanyId,
  });

  // Prefer closed fiscal year for reports, fall back to open one
  const activeFyId =
    fyId || fiscalYears?.find((fy) => fy.is_closed)?.id || fiscalYears?.[0]?.id;

  if (!activeCompanyId || !activeFyId) {
    return <p className="text-muted-foreground">Skapa ett företag och räkenskapsår först.</p>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Rapporter</h1>
        <div className="flex gap-2">
          <Button
            variant={tab === 'income' ? 'default' : 'outline'}
            size="sm"
            onClick={() => navigate({ search: { companyId, fyId, tab: 'income' } })}
          >
            Resultaträkning
          </Button>
          <Button
            variant={tab === 'balance' ? 'default' : 'outline'}
            size="sm"
            onClick={() => navigate({ search: { companyId, fyId, tab: 'balance' } })}
          >
            Balansräkning
          </Button>
          <Button
            variant={tab === 'full' ? 'default' : 'outline'}
            size="sm"
            onClick={() => navigate({ search: { companyId, fyId, tab: 'full' } })}
          >
            Årsredovisning
          </Button>
        </div>
      </div>

      {tab === 'income' && <IncomeStatementView fyId={activeFyId} />}
      {tab === 'balance' && <BalanceSheetView fyId={activeFyId} />}
      {tab === 'full' && <AnnualReportView fyId={activeFyId} />}
    </div>
  );
}

function IncomeStatementView({ fyId }: { fyId: string }) {
  const { data, isLoading } = useQuery({
    queryKey: ['income-statement', fyId],
    queryFn: () => annualReportApi.incomeStatement(fyId),
  });

  if (isLoading) return <p className="text-muted-foreground">Laddar...</p>;
  if (!data) return null;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Resultaträkning (K2)</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="font-mono text-sm space-y-0.5">
          <ISHeader current={data.current.fiscal_year} previous={data.previous?.fiscal_year} />
          <Separator className="my-2" />
          <ISRow label="Nettoomsättning" c={data.current.net_revenue} p={data.previous?.net_revenue} />
          <ISRow label="Övriga rörelseintäkter" c={data.current.other_operating_income} p={data.previous?.other_operating_income} hide />
          <Separator className="my-1" />
          <ISRow label="Råvaror och förnödenheter" c={neg(data.current.raw_materials)} p={data.previous ? neg(data.previous.raw_materials) : undefined} hide />
          <ISRow label="Handelsvaror" c={neg(data.current.goods_for_resale)} p={data.previous ? neg(data.previous.goods_for_resale) : undefined} hide />
          <ISRow label="Övriga externa kostnader" c={neg(data.current.other_external_costs)} p={data.previous ? neg(data.previous.other_external_costs) : undefined} />
          <ISRow label="Personalkostnader" c={neg(data.current.personnel_costs)} p={data.previous ? neg(data.previous.personnel_costs) : undefined} />
          <ISRow label="Av- och nedskrivningar" c={neg(data.current.depreciation)} p={data.previous ? neg(data.previous.depreciation) : undefined} hide />
          <Separator className="my-1" />
          <ISRow label="Rörelseresultat" c={data.current.operating_result} p={data.previous?.operating_result} bold />
          <Separator className="my-1" />
          <ISRow label="Finansiella intäkter" c={data.current.financial_income} p={data.previous?.financial_income} hide />
          <ISRow label="Finansiella kostnader" c={neg(data.current.financial_costs)} p={data.previous ? neg(data.previous.financial_costs) : undefined} hide />
          <ISRow label="Resultat efter finansiella poster" c={data.current.result_after_financial} p={data.previous?.result_after_financial} bold />
          <ISRow label="Bokslutsdispositioner" c={data.current.appropriations} p={data.previous?.appropriations} hide />
          <Separator className="my-1" />
          <ISRow label="Resultat före skatt" c={data.current.result_before_tax} p={data.previous?.result_before_tax} bold />
          <ISRow label="Skatt på årets resultat" c={neg(data.current.tax)} p={data.previous ? neg(data.previous.tax) : undefined} />
          <Separator className="my-1" />
          <ISRow label="ÅRETS RESULTAT" c={data.current.net_result} p={data.previous?.net_result} bold />
        </div>
      </CardContent>
    </Card>
  );
}

function BalanceSheetView({ fyId }: { fyId: string }) {
  const { data, isLoading } = useQuery({
    queryKey: ['balance-sheet', fyId],
    queryFn: () => annualReportApi.balanceSheet(fyId),
  });

  if (isLoading) return <p className="text-muted-foreground">Laddar...</p>;
  if (!data) return null;

  const c = data.current;
  const p = data.previous;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Balansräkning (K2)</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="font-mono text-sm space-y-0.5">
          <ISHeader current={c.fiscal_year} previous={p?.fiscal_year} />
          <Separator className="my-2" />

          <div className="font-semibold pt-1">TILLGÅNGAR</div>
          <div className="font-semibold text-muted-foreground text-xs pt-1">Anläggningstillgångar</div>
          <ISRow label="  Immateriella" c={c.assets.intangible_assets} p={p?.assets.intangible_assets} hide />
          <ISRow label="  Materiella" c={c.assets.tangible_assets} p={p?.assets.tangible_assets} hide />
          <ISRow label="  Finansiella" c={c.assets.financial_fixed_assets} p={p?.assets.financial_fixed_assets} hide />
          <ISRow label="Summa anläggningstillgångar" c={c.assets.total_fixed_assets} p={p?.assets.total_fixed_assets} bold />

          <div className="font-semibold text-muted-foreground text-xs pt-1">Omsättningstillgångar</div>
          <ISRow label="  Varulager" c={c.assets.inventory} p={p?.assets.inventory} hide />
          <ISRow label="  Kortfristiga fordringar" c={c.assets.current_receivables} p={p?.assets.current_receivables} />
          <ISRow label="  Kassa och bank" c={c.assets.cash_and_bank} p={p?.assets.cash_and_bank} />
          <ISRow label="Summa omsättningstillgångar" c={c.assets.total_current_assets} p={p?.assets.total_current_assets} bold />
          <Separator className="my-1" />
          <ISRow label="SUMMA TILLGÅNGAR" c={c.total_assets} p={p?.total_assets} bold />

          <Separator className="my-2" />

          <div className="font-semibold pt-1">EGET KAPITAL OCH SKULDER</div>
          <div className="font-semibold text-muted-foreground text-xs pt-1">Eget kapital</div>
          <ISRow label="  Bundet eget kapital" c={c.equity_and_liabilities.restricted_equity} p={p?.equity_and_liabilities.restricted_equity} />
          <ISRow label="  Fritt eget kapital" c={c.equity_and_liabilities.unrestricted_equity} p={p?.equity_and_liabilities.unrestricted_equity} />
          <ISRow label="Summa eget kapital" c={c.equity_and_liabilities.total_equity} p={p?.equity_and_liabilities.total_equity} bold />
          <ISRow label="Obeskattade reserver" c={c.equity_and_liabilities.untaxed_reserves} p={p?.equity_and_liabilities.untaxed_reserves} hide />
          <ISRow label="Långfristiga skulder" c={c.equity_and_liabilities.long_term_liabilities} p={p?.equity_and_liabilities.long_term_liabilities} hide />
          <ISRow label="Kortfristiga skulder" c={c.equity_and_liabilities.current_liabilities} p={p?.equity_and_liabilities.current_liabilities} />
          <Separator className="my-1" />
          <ISRow label="SUMMA EGET KAPITAL OCH SKULDER" c={c.total_equity_and_liabilities} p={p?.total_equity_and_liabilities} bold />
        </div>
      </CardContent>
    </Card>
  );
}

function AnnualReportView({ fyId }: { fyId: string }) {
  const { data, isLoading } = useQuery({
    queryKey: ['annual-report', fyId],
    queryFn: () => annualReportApi.full(fyId),
  });

  if (isLoading) return <p className="text-muted-foreground">Laddar årsredovisning...</p>;
  if (!data) return null;

  const dr = data.directors_report;

  return (
    <div className="space-y-6">
      {/* PDF download */}
      <div className="flex justify-end">
        <a href={annualReportApi.pdfUrl(fyId)} download>
          <Button variant="outline" size="sm">
            Ladda ner PDF
          </Button>
        </a>
      </div>

      {/* Header */}
      <Card>
        <CardContent className="pt-6 text-center space-y-1">
          <p className="text-xs text-muted-foreground uppercase tracking-widest">Årsredovisning</p>
          <p className="text-xl font-semibold">{data.company.name}</p>
          <p className="text-sm text-muted-foreground">
            Org.nr: {formatOrgNr(data.company.org_number)}
          </p>
          <p className="text-sm text-muted-foreground">
            Räkenskapsår: {data.fiscal_year.start_date} — {data.fiscal_year.end_date}
          </p>
          {!data.fiscal_year.is_closed && (
            <Badge variant="secondary">Ej stängt</Badge>
          )}
        </CardContent>
      </Card>

      {/* Förvaltningsberättelse */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Förvaltningsberättelse</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4 text-sm">
          <div>
            <p className="font-semibold">Verksamheten</p>
            <p className="text-muted-foreground">{dr.business_description}</p>
          </div>
          <div>
            <p className="font-semibold">Väsentliga händelser</p>
            <p className="text-muted-foreground">{dr.important_events}</p>
          </div>
          <div>
            <p className="font-semibold">Framtida utveckling</p>
            <p className="text-muted-foreground">{dr.future_outlook}</p>
          </div>
          {dr.profit_allocation && (
            <div>
              <p className="font-semibold">Förslag till vinstdisposition</p>
              <div className="text-muted-foreground font-mono text-xs mt-1 space-y-0.5">
                <p>Årets resultat: {dr.profit_allocation.result_for_year} kr</p>
                <p>Balanserat resultat: {dr.profit_allocation.retained_earnings} kr</p>
                <p className="font-semibold">Summa: {dr.profit_allocation.total_available} kr</p>
                <p className="pt-1">I ny räkning överföres: {dr.profit_allocation.carry_forward} kr</p>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Inline IS + BS */}
      <IncomeStatementView fyId={fyId} />
      <BalanceSheetView fyId={fyId} />

      {/* Notes */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Noter</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4 text-sm">
          {data.notes.items.map((note) => (
            <div key={note.number}>
              <p className="font-semibold">
                Not {note.number}: {note.title}
              </p>
              <p className="text-muted-foreground whitespace-pre-line">{note.content}</p>
            </div>
          ))}
        </CardContent>
      </Card>
    </div>
  );
}

// --- Shared components ---

function ISHeader({ current, previous }: { current: string; previous?: string }) {
  return (
    <div className="flex justify-between text-xs text-muted-foreground font-semibold">
      <span className="w-1/2"></span>
      <span className="w-1/4 text-right">{current}</span>
      {previous && <span className="w-1/4 text-right">{previous}</span>}
    </div>
  );
}

function ISRow({
  label,
  c,
  p,
  bold,
  hide,
}: {
  label: string;
  c: string;
  p?: string;
  bold?: boolean;
  hide?: boolean;
}) {
  if (hide && isZero(c) && (!p || isZero(p))) return null;

  return (
    <div className={`flex justify-between ${bold ? 'font-semibold' : ''}`}>
      <span className="w-1/2 truncate">{label}</span>
      <span className="w-1/4 text-right">{formatAmount(c)}</span>
      {p !== undefined && <span className="w-1/4 text-right">{formatAmount(p)}</span>}
    </div>
  );
}

function isZero(v: string): boolean {
  return parseFloat(v) === 0;
}

function neg(v: string): string {
  const n = parseFloat(v);
  if (n === 0) return '0.00';
  return (-n).toFixed(2);
}

function formatAmount(v: string): string {
  const n = parseFloat(v);
  if (n === 0) return '-';
  return n.toLocaleString('sv-SE', { minimumFractionDigits: 0, maximumFractionDigits: 0 });
}

function formatOrgNr(org: string): string {
  if (org.length === 10 && !org.includes('-')) {
    return `${org.slice(0, 6)}-${org.slice(6)}`;
  }
  return org;
}
