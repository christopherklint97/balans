import { createFileRoute } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { complianceApi } from '@/api/queries';
import { useFiscalYear } from '@/hooks/use-fiscal-year';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';

interface ComplianceSearch {
  tab?: 'eligibility' | 'multiyear' | 'audit';
}

export const Route = createFileRoute('/compliance')({
  component: CompliancePage,
  validateSearch: (search: Record<string, unknown>): ComplianceSearch => ({
    tab: (search.tab as ComplianceSearch['tab']) || 'eligibility',
  }),
});

function CompliancePage() {
  const { tab } = Route.useSearch();
  const navigate = Route.useNavigate();
  const { activeCompanyId, activeFyId } = useFiscalYear();

  if (!activeCompanyId) {
    return <p className="text-muted-foreground">Skapa ett företag först.</p>;
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <h1 className="text-2xl font-semibold">Compliance</h1>
        <div className="flex flex-wrap gap-2">
          <Button
            variant={tab === 'eligibility' ? 'default' : 'outline'}
            size="sm"
            onClick={() => navigate({ search: { tab: 'eligibility' } })}
          >
            K2-behörighet
          </Button>
          <Button
            variant={tab === 'multiyear' ? 'default' : 'outline'}
            size="sm"
            onClick={() => navigate({ search: { tab: 'multiyear' } })}
          >
            Flerårsöversikt
          </Button>
          <Button
            variant={tab === 'audit' ? 'default' : 'outline'}
            size="sm"
            onClick={() => navigate({ search: { tab: 'audit' } })}
          >
            Ändringslogg
          </Button>
        </div>
      </div>

      {tab === 'eligibility' && activeFyId && (
        <K2Eligibility companyId={activeCompanyId} fyId={activeFyId} />
      )}
      {tab === 'multiyear' && <MultiYearView companyId={activeCompanyId} />}
      {tab === 'audit' && <AuditLogView companyId={activeCompanyId} />}
    </div>
  );
}

function K2Eligibility({ companyId, fyId }: { companyId: string; fyId: string }) {
  const [employees, setEmployees] = useState('0');

  const { data, isLoading } = useQuery({
    queryKey: ['k2-eligibility', companyId, fyId, employees],
    queryFn: () => complianceApi.k2Eligibility(companyId, fyId, parseInt(employees) || 0),
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">K2-behörighetskontroll</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-muted-foreground">
          Kontrollera att företaget uppfyller kraven för att tillämpa K2 (BFNAR 2016:10).
          Ett mindre företag får inte överskrida mer än ett av tre gränsvärden under två
          på varandra följande räkenskapsår.
        </p>

        <div className="space-y-2">
          <Label htmlFor="employees">Medelantal anställda</Label>
          <Input
            id="employees"
            type="number"
            min="0"
            value={employees}
            onChange={(e) => setEmployees(e.target.value)}
            className="max-w-[200px]"
          />
        </div>

        {isLoading && <p className="text-sm text-muted-foreground">Kontrollerar...</p>}

        {data && (
          <div className="space-y-3">
            <div className="flex items-center gap-2">
              <Badge
                className={
                  data.is_eligible
                    ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                    : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
                }
              >
                {data.is_eligible ? 'K2 tillåtet' : 'K2 ej tillåtet'}
              </Badge>
              {data.reason && (
                <span className="text-sm text-muted-foreground">{data.reason}</span>
              )}
            </div>

            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Kriterium</TableHead>
                  <TableHead className="text-right">Värde</TableHead>
                  <TableHead className="text-right">Gränsvärde</TableHead>
                  <TableHead className="text-center">Status</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow>
                  <TableCell>Medelantal anställda</TableCell>
                  <TableCell className="text-right font-mono">
                    {data.checks.average_employees}
                  </TableCell>
                  <TableCell className="text-right font-mono">
                    {data.thresholds.max_employees}
                  </TableCell>
                  <TableCell className="text-center">
                    {data.checks.employees_exceeded ? '❌' : '✓'}
                  </TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>Balansomslutning</TableCell>
                  <TableCell className="text-right font-mono">
                    {fmt(data.checks.balance_sheet_total)}
                  </TableCell>
                  <TableCell className="text-right font-mono">
                    {fmt(data.thresholds.max_balance_sheet)}
                  </TableCell>
                  <TableCell className="text-center">
                    {data.checks.balance_exceeded ? '❌' : '✓'}
                  </TableCell>
                </TableRow>
                <TableRow>
                  <TableCell>Nettoomsättning</TableCell>
                  <TableCell className="text-right font-mono">
                    {fmt(data.checks.net_revenue)}
                  </TableCell>
                  <TableCell className="text-right font-mono">
                    {fmt(data.thresholds.max_net_revenue)}
                  </TableCell>
                  <TableCell className="text-center">
                    {data.checks.revenue_exceeded ? '❌' : '✓'}
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>

            <p className="text-xs text-muted-foreground">
              Överskridna gränsvärden: {data.checks.thresholds_exceeded}/3 (max 1 tillåtet)
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function MultiYearView({ companyId }: { companyId: string }) {
  const { data, isLoading } = useQuery({
    queryKey: ['multi-year', companyId],
    queryFn: () => complianceApi.multiYear(companyId),
  });

  if (isLoading) return <p className="text-muted-foreground">Laddar flerårsöversikt...</p>;
  if (!data?.years.length)
    return <p className="text-muted-foreground">Inga räkenskapsår att visa.</p>;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Flerårsöversikt</CardTitle>
      </CardHeader>
      <CardContent className="p-0">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Räkenskapsår</TableHead>
              <TableHead className="text-right">Nettoomsättning</TableHead>
              <TableHead className="text-right">Rörelseresultat</TableHead>
              <TableHead className="text-right">Res. efter fin.</TableHead>
              <TableHead className="text-right">Balansomslutning</TableHead>
              <TableHead className="text-right">Soliditet</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data.years.map((y) => (
              <TableRow key={y.fiscal_year}>
                <TableCell className="text-xs">{y.fiscal_year}</TableCell>
                <TableCell className="text-right font-mono">{fmt(y.net_revenue)}</TableCell>
                <TableCell className="text-right font-mono">{fmt(y.operating_result)}</TableCell>
                <TableCell className="text-right font-mono">
                  {fmt(y.result_after_financial)}
                </TableCell>
                <TableCell className="text-right font-mono">{fmt(y.total_assets)}</TableCell>
                <TableCell className="text-right font-mono">{y.equity_ratio}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}

function AuditLogView({ companyId }: { companyId: string }) {
  const { data, isLoading } = useQuery({
    queryKey: ['audit-log', companyId],
    queryFn: () => complianceApi.auditLog(companyId),
  });

  if (isLoading) return <p className="text-muted-foreground">Laddar ändringslogg...</p>;
  if (!data?.length)
    return <p className="text-muted-foreground">Inga loggposter ännu.</p>;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Ändringslogg</CardTitle>
      </CardHeader>
      <CardContent className="p-0">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-40">Tidpunkt</TableHead>
              <TableHead className="w-24">Typ</TableHead>
              <TableHead className="w-20">Åtgärd</TableHead>
              <TableHead>Detaljer</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data.map((entry) => (
              <TableRow key={entry.id}>
                <TableCell className="text-xs font-mono">
                  {entry.created_at.slice(0, 19).replace('T', ' ')}
                </TableCell>
                <TableCell>
                  <Badge variant="secondary" className="text-xs">
                    {entry.entity_type}
                  </Badge>
                </TableCell>
                <TableCell className="text-xs">{entry.action}</TableCell>
                <TableCell className="text-xs text-muted-foreground truncate max-w-[300px]">
                  {entry.details}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}

function fmt(v: string): string {
  const n = parseFloat(v);
  if (n === 0) return '-';
  return n.toLocaleString('sv-SE', { minimumFractionDigits: 0, maximumFractionDigits: 0 });
}
