import { createFileRoute } from '@tanstack/react-router';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { vouchersApi, accountsApi, companiesApi, fiscalYearsApi, reportsApi } from '@/api/queries';
import type { CreateVoucherLine } from '@/api/types';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Separator } from '@/components/ui/separator';

interface VouchersSearch {
  companyId?: string;
  fyId?: string;
  view?: 'list' | 'new' | 'balance';
}

export const Route = createFileRoute('/vouchers')({
  component: VouchersPage,
  validateSearch: (search: Record<string, unknown>): VouchersSearch => ({
    companyId: search.companyId as string | undefined,
    fyId: search.fyId as string | undefined,
    view: (search.view as VouchersSearch['view']) || 'list',
  }),
});

function VouchersPage() {
  const { companyId, fyId, view } = Route.useSearch();
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

  const activeFyId = fyId || fiscalYears?.find((fy) => !fy.is_closed)?.id;

  if (!activeCompanyId || !activeFyId) {
    return <p className="text-muted-foreground">Skapa ett företag och räkenskapsår först.</p>;
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <h1 className="text-2xl font-semibold">Verifikationer</h1>
        <div className="flex flex-wrap gap-2">
          <Button
            variant={view === 'list' ? 'default' : 'outline'}
            size="sm"
            onClick={() => navigate({ search: { companyId, fyId, view: 'list' } })}
          >
            Lista
          </Button>
          <Button
            variant={view === 'new' ? 'default' : 'outline'}
            size="sm"
            onClick={() => navigate({ search: { companyId, fyId, view: 'new' } })}
          >
            Ny verifikation
          </Button>
          <Button
            variant={view === 'balance' ? 'default' : 'outline'}
            size="sm"
            onClick={() => navigate({ search: { companyId, fyId, view: 'balance' } })}
          >
            Saldobalans
          </Button>
        </div>
      </div>

      {view === 'new' ? (
        <VoucherForm
          companyId={activeCompanyId}
          fyId={activeFyId}
          onSuccess={() => navigate({ search: { companyId, fyId, view: 'list' } })}
        />
      ) : view === 'balance' ? (
        <TrialBalance fyId={activeFyId} />
      ) : (
        <VoucherList fyId={activeFyId} />
      )}
    </div>
  );
}

function VoucherList({ fyId }: { fyId: string }) {
  const { data: vouchers, isLoading } = useQuery({
    queryKey: ['vouchers', fyId],
    queryFn: () => vouchersApi.list(fyId),
  });

  if (isLoading) return <p className="text-muted-foreground">Laddar...</p>;
  if (!vouchers?.length) return <p className="text-muted-foreground">Inga verifikationer ännu.</p>;

  return (
    <Card>
      <CardContent className="p-0">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-16">Nr</TableHead>
              <TableHead className="w-28">Datum</TableHead>
              <TableHead>Beskrivning</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {vouchers.map((v) => (
              <TableRow key={v.id}>
                <TableCell className="font-mono">{v.voucher_number}</TableCell>
                <TableCell>{v.date}</TableCell>
                <TableCell>{v.description}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}

interface VoucherLineInput {
  account_number: string;
  debit: string;
  credit: string;
}

function VoucherForm({
  companyId,
  fyId,
  onSuccess,
}: {
  companyId: string;
  fyId: string;
  onSuccess: () => void;
}) {
  const queryClient = useQueryClient();
  const [date, setDate] = useState(new Date().toISOString().slice(0, 10));
  const [description, setDescription] = useState('');
  const [lines, setLines] = useState<VoucherLineInput[]>([
    { account_number: '', debit: '', credit: '' },
    { account_number: '', debit: '', credit: '' },
  ]);
  const [error, setError] = useState('');

  const { data: accounts } = useQuery({
    queryKey: ['accounts', companyId],
    queryFn: () => accountsApi.list(companyId),
  });

  const totalDebit = lines.reduce((sum, l) => sum + (parseFloat(l.debit) || 0), 0);
  const totalCredit = lines.reduce((sum, l) => sum + (parseFloat(l.credit) || 0), 0);
  const isBalanced = Math.abs(totalDebit - totalCredit) < 0.005 && totalDebit > 0;

  const mutation = useMutation({
    mutationFn: () => {
      const voucherLines: CreateVoucherLine[] = lines
        .filter((l) => l.account_number && (l.debit || l.credit))
        .map((l) => ({
          account_number: parseInt(l.account_number, 10),
          debit: (parseFloat(l.debit) || 0).toFixed(2),
          credit: (parseFloat(l.credit) || 0).toFixed(2),
        }));
      return vouchersApi.create(fyId, { date, description, lines: voucherLines });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vouchers', fyId] });
      onSuccess();
    },
    onError: (err: Error) => setError(err.message),
  });

  const updateLine = (index: number, field: keyof VoucherLineInput, value: string) => {
    setLines((prev) => prev.map((l, i) => (i === index ? { ...l, [field]: value } : l)));
  };

  const addLine = () => {
    setLines((prev) => [...prev, { account_number: '', debit: '', credit: '' }]);
  };

  const removeLine = (index: number) => {
    if (lines.length > 2) {
      setLines((prev) => prev.filter((_, i) => i !== index));
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Ny verifikation</CardTitle>
      </CardHeader>
      <CardContent>
        <form
          className="space-y-4"
          onSubmit={(e) => {
            e.preventDefault();
            setError('');
            mutation.mutate();
          }}
        >
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="date">Datum</Label>
              <Input
                id="date"
                type="date"
                value={date}
                onChange={(e) => setDate(e.target.value)}
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="desc">Beskrivning</Label>
              <Input
                id="desc"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="T.ex. Kundbetalning faktura 1001"
                required
              />
            </div>
          </div>

          <Separator />

          <div className="space-y-2">
            {/* Desktop header */}
            <div className="hidden sm:grid grid-cols-[1fr_2fr_100px_100px_40px] gap-2 text-sm font-medium text-muted-foreground">
              <span>Konto</span>
              <span>Kontonamn</span>
              <span>Debet</span>
              <span>Kredit</span>
              <span></span>
            </div>

            {lines.map((line, i) => {
              const matchedAccount = accounts?.find(
                (a) => a.number === parseInt(line.account_number, 10),
              );
              return (
                <div key={i}>
                  {/* Desktop row */}
                  <div className="hidden sm:grid grid-cols-[1fr_2fr_100px_100px_40px] gap-2">
                    <Input
                      value={line.account_number}
                      onChange={(e) => updateLine(i, 'account_number', e.target.value)}
                      placeholder="1910"
                      className="font-mono"
                    />
                    <span className="flex items-center text-sm text-muted-foreground truncate">
                      {matchedAccount?.name || ''}
                    </span>
                    <Input
                      value={line.debit}
                      onChange={(e) => {
                        updateLine(i, 'debit', e.target.value);
                        if (e.target.value) updateLine(i, 'credit', '');
                      }}
                      placeholder="0.00"
                      className="font-mono text-right"
                      type="number"
                      step="0.01"
                      min="0"
                    />
                    <Input
                      value={line.credit}
                      onChange={(e) => {
                        updateLine(i, 'credit', e.target.value);
                        if (e.target.value) updateLine(i, 'debit', '');
                      }}
                      placeholder="0.00"
                      className="font-mono text-right"
                      type="number"
                      step="0.01"
                      min="0"
                    />
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={() => removeLine(i)}
                      disabled={lines.length <= 2}
                      className="text-muted-foreground"
                    >
                      x
                    </Button>
                  </div>

                  {/* Mobile card */}
                  <div className="sm:hidden rounded-md border border-border p-3 space-y-2">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2 flex-1 min-w-0">
                        <Input
                          value={line.account_number}
                          onChange={(e) => updateLine(i, 'account_number', e.target.value)}
                          placeholder="Konto"
                          className="font-mono w-20"
                        />
                        <span className="text-sm text-muted-foreground truncate">
                          {matchedAccount?.name || ''}
                        </span>
                      </div>
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={() => removeLine(i)}
                        disabled={lines.length <= 2}
                        className="text-muted-foreground shrink-0"
                      >
                        x
                      </Button>
                    </div>
                    <div className="grid grid-cols-2 gap-2">
                      <div className="space-y-1">
                        <span className="text-xs text-muted-foreground">Debet</span>
                        <Input
                          value={line.debit}
                          onChange={(e) => {
                            updateLine(i, 'debit', e.target.value);
                            if (e.target.value) updateLine(i, 'credit', '');
                          }}
                          placeholder="0.00"
                          className="font-mono text-right"
                          type="number"
                          step="0.01"
                          min="0"
                        />
                      </div>
                      <div className="space-y-1">
                        <span className="text-xs text-muted-foreground">Kredit</span>
                        <Input
                          value={line.credit}
                          onChange={(e) => {
                            updateLine(i, 'credit', e.target.value);
                            if (e.target.value) updateLine(i, 'debit', '');
                          }}
                          placeholder="0.00"
                          className="font-mono text-right"
                          type="number"
                          step="0.01"
                          min="0"
                        />
                      </div>
                    </div>
                  </div>
                </div>
              );
            })}

            <Button type="button" variant="outline" size="sm" onClick={addLine}>
              + Lägg till rad
            </Button>
          </div>

          <Separator />

          <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex flex-wrap gap-x-4 gap-y-1 text-sm">
              <span>
                Debet: <span className="font-mono font-medium">{totalDebit.toFixed(2)}</span>
              </span>
              <span>
                Kredit: <span className="font-mono font-medium">{totalCredit.toFixed(2)}</span>
              </span>
              <span className={isBalanced ? 'text-green-600' : 'text-destructive'}>
                Diff: {(totalDebit - totalCredit).toFixed(2)}
              </span>
            </div>
            <Button type="submit" disabled={!isBalanced || mutation.isPending || !description} className="w-full sm:w-auto">
              {mutation.isPending ? 'Sparar...' : 'Bokför'}
            </Button>
          </div>

          {error && <p className="text-sm text-destructive">{error}</p>}
        </form>
      </CardContent>
    </Card>
  );
}

function TrialBalance({ fyId }: { fyId: string }) {
  const { data: rows, isLoading } = useQuery({
    queryKey: ['trial-balance', fyId],
    queryFn: () => reportsApi.trialBalance(fyId),
  });

  if (isLoading) return <p className="text-muted-foreground">Laddar saldobalans...</p>;
  if (!rows?.length) return <p className="text-muted-foreground">Inga transaktioner ännu.</p>;

  const totalDebit = rows.reduce((s, r) => s + parseFloat(r.debit_total), 0);
  const totalCredit = rows.reduce((s, r) => s + parseFloat(r.credit_total), 0);

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Saldobalans</CardTitle>
      </CardHeader>
      <CardContent className="p-0">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-20">Konto</TableHead>
              <TableHead>Namn</TableHead>
              <TableHead className="text-right w-28">Debet</TableHead>
              <TableHead className="text-right w-28">Kredit</TableHead>
              <TableHead className="text-right w-28">Saldo</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {rows.map((r) => (
              <TableRow key={r.account_number}>
                <TableCell className="font-mono">{r.account_number}</TableCell>
                <TableCell>{r.account_name}</TableCell>
                <TableCell className="text-right font-mono">{r.debit_total}</TableCell>
                <TableCell className="text-right font-mono">{r.credit_total}</TableCell>
                <TableCell className="text-right font-mono">{r.balance}</TableCell>
              </TableRow>
            ))}
            <TableRow className="font-semibold">
              <TableCell></TableCell>
              <TableCell>Summa</TableCell>
              <TableCell className="text-right font-mono">{totalDebit.toFixed(2)}</TableCell>
              <TableCell className="text-right font-mono">{totalCredit.toFixed(2)}</TableCell>
              <TableCell className="text-right font-mono">
                {(totalDebit - totalCredit).toFixed(2)}
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}
