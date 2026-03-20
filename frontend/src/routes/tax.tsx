import { createFileRoute } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { companiesApi, fiscalYearsApi, taxApi } from '@/api/queries';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';

interface TaxSearch {
  companyId?: string;
  fyId?: string;
}

export const Route = createFileRoute('/tax')({
  component: TaxPage,
  validateSearch: (search: Record<string, unknown>): TaxSearch => ({
    companyId: search.companyId as string | undefined,
    fyId: search.fyId as string | undefined,
  }),
});

function TaxPage() {
  const { companyId, fyId } = Route.useSearch();

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

  const activeFyId = fyId || fiscalYears?.find((fy) => fy.is_closed)?.id || fiscalYears?.[0]?.id;

  if (!activeCompanyId || !activeFyId) {
    return <p className="text-muted-foreground">Skapa ett företag och räkenskapsår först.</p>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold">INK2 Skattedeklaration</h1>
        <a href={taxApi.sruDownloadUrl(activeCompanyId, activeFyId)} download>
          <Button variant="outline" size="sm">
            Ladda ner SRU-fil
          </Button>
        </a>
      </div>
      <p className="text-sm text-muted-foreground">
        Underlag för Inkomstdeklaration 2 (INK2) med SRU-koder för Skatteverket.
      </p>

      <Ink2View companyId={activeCompanyId} fyId={activeFyId} />
    </div>
  );
}

function Ink2View({ companyId, fyId }: { companyId: string; fyId: string }) {
  const { data, isLoading } = useQuery({
    queryKey: ['ink2', companyId, fyId],
    queryFn: () => taxApi.ink2(companyId, fyId),
  });

  if (isLoading) return <p className="text-muted-foreground">Laddar INK2-data...</p>;
  if (!data) return null;

  return (
    <div className="space-y-4">
      <Card>
        <CardContent className="pt-4">
          <div className="grid grid-cols-2 gap-2 text-sm">
            <div>
              <span className="text-muted-foreground">Företag: </span>
              {data.company_name}
            </div>
            <div>
              <span className="text-muted-foreground">Org.nr: </span>
              <span className="font-mono">{formatOrgNr(data.org_number)}</span>
            </div>
            <div>
              <span className="text-muted-foreground">Räkenskapsår: </span>
              {data.fiscal_year_start} — {data.fiscal_year_end}
            </div>
            <div>
              <span className="text-muted-foreground">SRU-fält: </span>
              {data.fields.length} st
            </div>
          </div>
        </CardContent>
      </Card>

      {data.sections.map((section) => (
        <Card key={section.title}>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">{section.title}</CardTitle>
          </CardHeader>
          <CardContent className="p-0">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-20">SRU</TableHead>
                  <TableHead>Fält</TableHead>
                  <TableHead className="w-16 text-right">Konton</TableHead>
                  <TableHead className="w-32 text-right">Belopp (SEK)</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {section.fields.map((field) => (
                  <TableRow key={field.sru_code}>
                    <TableCell>
                      <Badge variant="secondary" className="font-mono text-xs">
                        {field.sru_code}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-sm">{field.label}</TableCell>
                    <TableCell className="text-right text-xs text-muted-foreground font-mono">
                      {field.accounts.join(', ')}
                    </TableCell>
                    <TableCell className="text-right font-mono font-medium">
                      {formatAmount(field.amount)}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      ))}
    </div>
  );
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
