import { createFileRoute } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { accountsApi } from '@/api/queries';
import { useFiscalYear } from '@/hooks/use-fiscal-year';
import type { Account } from '@/api/types';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export const Route = createFileRoute('/accounts')({
  component: AccountsPage,
});

const ACCOUNT_CLASS_NAMES: Record<number, string> = {
  1: 'Tillgångar',
  2: 'Eget kapital och skulder',
  3: 'Rörelseintäkter',
  4: 'Material och varukostnader',
  5: 'Övriga externa kostnader',
  6: 'Övriga externa kostnader forts.',
  7: 'Personalkostnader',
  8: 'Finansiella poster',
};

const ACCOUNT_TYPE_LABELS: Record<string, string> = {
  asset: 'Tillgång',
  equity: 'Eget kapital',
  liability: 'Skuld',
  revenue: 'Intäkt',
  expense: 'Kostnad',
};

function AccountsPage() {
  const { activeCompanyId } = useFiscalYear();

  const { data: accounts, isLoading } = useQuery({
    queryKey: ['accounts', activeCompanyId],
    queryFn: () => accountsApi.list(activeCompanyId!),
    enabled: !!activeCompanyId,
  });

  if (!activeCompanyId) {
    return <p className="text-muted-foreground">Skapa ett företag först.</p>;
  }

  if (isLoading) {
    return <p className="text-muted-foreground">Laddar kontoplan...</p>;
  }

  // Group by account class
  const grouped = (accounts || []).reduce<Record<number, Account[]>>((acc, account) => {
    const cls = Math.floor(account.number / 1000);
    if (!acc[cls]) acc[cls] = [];
    acc[cls].push(account);
    return acc;
  }, {});

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Kontoplan (BAS)</h1>
      <p className="text-sm text-muted-foreground">
        {accounts?.length || 0} konton
      </p>

      {Object.entries(grouped)
        .sort(([a], [b]) => Number(a) - Number(b))
        .map(([cls, classAccounts]) => (
          <Card key={cls}>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">
                Klass {cls}: {ACCOUNT_CLASS_NAMES[Number(cls)] || 'Övrigt'}
              </CardTitle>
            </CardHeader>
            <CardContent className="p-0">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-24">Konto</TableHead>
                    <TableHead>Namn</TableHead>
                    <TableHead className="w-24">Typ</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {classAccounts.map((account) => (
                    <TableRow key={account.id} className={!account.is_active ? 'opacity-50' : ''}>
                      <TableCell className="font-mono text-sm">{account.number}</TableCell>
                      <TableCell>{account.name}</TableCell>
                      <TableCell>
                        <Badge variant="secondary" className="text-xs">
                          {ACCOUNT_TYPE_LABELS[account.account_type] || account.account_type}
                        </Badge>
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
