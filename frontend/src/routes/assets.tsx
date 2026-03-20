import { createFileRoute } from '@tanstack/react-router';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { companiesApi, fiscalYearsApi, assetsApi } from '@/api/queries';
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

interface AssetsSearch {
  companyId?: string;
  fyId?: string;
  view?: 'register' | 'depreciation' | 'new';
}

export const Route = createFileRoute('/assets')({
  component: AssetsPage,
  validateSearch: (search: Record<string, unknown>): AssetsSearch => ({
    companyId: search.companyId as string | undefined,
    fyId: search.fyId as string | undefined,
    view: (search.view as AssetsSearch['view']) || 'register',
  }),
});

const ASSET_TYPES = [
  { value: 'computer', label: 'Dator', life: 36 },
  { value: 'equipment', label: 'Inventarie', life: 60 },
  { value: 'machinery', label: 'Maskin', life: 60 },
  { value: 'vehicle', label: 'Fordon', life: 72 },
  { value: 'building', label: 'Byggnad', life: 600 },
  { value: 'intangible', label: 'Immateriell', life: 60 },
];

const TYPE_LABELS: Record<string, string> = Object.fromEntries(
  ASSET_TYPES.map((t) => [t.value, t.label]),
);

function AssetsPage() {
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

  if (!activeCompanyId) {
    return <p className="text-muted-foreground">Skapa ett företag först.</p>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Anläggningsregister</h1>
        <div className="flex gap-2">
          <Button
            variant={view === 'register' ? 'default' : 'outline'}
            size="sm"
            onClick={() => navigate({ search: { companyId, fyId, view: 'register' } })}
          >
            Register
          </Button>
          <Button
            variant={view === 'depreciation' ? 'default' : 'outline'}
            size="sm"
            onClick={() => navigate({ search: { companyId, fyId, view: 'depreciation' } })}
          >
            Avskrivningar
          </Button>
          <Button
            variant={view === 'new' ? 'default' : 'outline'}
            size="sm"
            onClick={() => navigate({ search: { companyId, fyId, view: 'new' } })}
          >
            Ny tillgång
          </Button>
        </div>
      </div>

      {view === 'register' && <AssetList companyId={activeCompanyId} />}
      {view === 'depreciation' && activeFyId && (
        <DepreciationView companyId={activeCompanyId} fyId={activeFyId} />
      )}
      {view === 'new' && (
        <CreateAssetForm
          companyId={activeCompanyId}
          onSuccess={() => navigate({ search: { companyId, fyId, view: 'register' } })}
        />
      )}
    </div>
  );
}

function AssetList({ companyId }: { companyId: string }) {
  const { data: assets, isLoading } = useQuery({
    queryKey: ['assets', companyId],
    queryFn: () => assetsApi.list(companyId),
  });

  if (isLoading) return <p className="text-muted-foreground">Laddar...</p>;
  if (!assets?.length) return <p className="text-muted-foreground">Inga tillgångar registrerade.</p>;

  return (
    <Card>
      <CardContent className="p-0">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Namn</TableHead>
              <TableHead className="w-24">Typ</TableHead>
              <TableHead className="w-28">Inköpsdatum</TableHead>
              <TableHead className="w-28 text-right">Anskaffn.värde</TableHead>
              <TableHead className="w-20 text-right">Livslängd</TableHead>
              <TableHead className="w-20 text-center">Status</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {assets.map((asset) => (
              <TableRow key={asset.id} className={asset.is_disposed ? 'opacity-50' : ''}>
                <TableCell className="font-medium">{asset.name}</TableCell>
                <TableCell>
                  <Badge variant="secondary" className="text-xs">
                    {TYPE_LABELS[asset.asset_type] || asset.asset_type}
                  </Badge>
                </TableCell>
                <TableCell>{asset.acquisition_date}</TableCell>
                <TableCell className="text-right font-mono">{fmt(asset.acquisition_cost)}</TableCell>
                <TableCell className="text-right">{asset.useful_life_months} mån</TableCell>
                <TableCell className="text-center">
                  {asset.is_disposed ? (
                    <Badge variant="secondary" className="text-xs">Avyttrad</Badge>
                  ) : (
                    <Badge className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200 text-xs">
                      Aktiv
                    </Badge>
                  )}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}

function DepreciationView({ companyId, fyId }: { companyId: string; fyId: string }) {
  const queryClient = useQueryClient();

  const { data, isLoading } = useQuery({
    queryKey: ['depreciation', companyId, fyId],
    queryFn: () => assetsApi.depreciation(companyId, fyId),
  });

  const generateMutation = useMutation({
    mutationFn: () => assetsApi.generateDepreciation(companyId, fyId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vouchers'] });
      queryClient.invalidateQueries({ queryKey: ['depreciation'] });
    },
  });

  if (isLoading) return <p className="text-muted-foreground">Beräknar avskrivningar...</p>;
  if (!data?.assets.length) return <p className="text-muted-foreground">Inga tillgångar att skriva av.</p>;

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="pb-2">
          <div className="flex items-center justify-between">
            <CardTitle className="text-base">Avskrivningsplan</CardTitle>
            <Button
              size="sm"
              onClick={() => generateMutation.mutate()}
              disabled={generateMutation.isPending}
            >
              {generateMutation.isPending ? 'Skapar...' : 'Bokför avskrivningar'}
            </Button>
          </div>
        </CardHeader>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Tillgång</TableHead>
                <TableHead className="w-20">Typ</TableHead>
                <TableHead className="w-28 text-right">Anskaff.värde</TableHead>
                <TableHead className="w-28 text-right">Årets avskr.</TableHead>
                <TableHead className="w-28 text-right">Ack. avskr.</TableHead>
                <TableHead className="w-28 text-right">Bokfört värde</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.assets.map((a) => (
                <TableRow key={a.asset_id}>
                  <TableCell>{a.asset_name}</TableCell>
                  <TableCell>
                    <Badge variant="secondary" className="text-xs">
                      {TYPE_LABELS[a.asset_type] || a.asset_type}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-right font-mono">{fmt(a.acquisition_cost)}</TableCell>
                  <TableCell className="text-right font-mono">{fmt(a.depreciation_this_year)}</TableCell>
                  <TableCell className="text-right font-mono">{fmt(a.accumulated_depreciation)}</TableCell>
                  <TableCell className="text-right font-mono font-medium">{fmt(a.book_value)}</TableCell>
                </TableRow>
              ))}
              <TableRow className="font-semibold">
                <TableCell colSpan={3}>Totalt</TableCell>
                <TableCell className="text-right font-mono">{fmt(data.total_depreciation)}</TableCell>
                <TableCell colSpan={2}></TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      {generateMutation.isSuccess && (
        <p className="text-sm text-green-700 dark:text-green-400">
          Avskrivningar bokförda ({generateMutation.data.vouchers_created} verifikationer skapade).
        </p>
      )}
      {generateMutation.isError && (
        <p className="text-sm text-destructive">{(generateMutation.error as Error).message}</p>
      )}
    </div>
  );
}

function CreateAssetForm({ companyId, onSuccess }: { companyId: string; onSuccess: () => void }) {
  const queryClient = useQueryClient();
  const [name, setName] = useState('');
  const [assetType, setAssetType] = useState('computer');
  const [date, setDate] = useState(new Date().toISOString().slice(0, 10));
  const [cost, setCost] = useState('');
  const [lifeMonths, setLifeMonths] = useState(36);
  const [error, setError] = useState('');

  // Update life when type changes
  const handleTypeChange = (type: string) => {
    setAssetType(type);
    const found = ASSET_TYPES.find((t) => t.value === type);
    if (found) setLifeMonths(found.life);
  };

  const mutation = useMutation({
    mutationFn: () =>
      assetsApi.create(companyId, {
        name,
        asset_type: assetType,
        acquisition_date: date,
        acquisition_cost: (parseFloat(cost) || 0).toFixed(2),
        useful_life_months: lifeMonths,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['assets'] });
      onSuccess();
    },
    onError: (err: Error) => setError(err.message),
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Registrera ny tillgång</CardTitle>
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
              <Label htmlFor="name">Namn</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="MacBook Pro"
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="type">Typ</Label>
              <select
                id="type"
                value={assetType}
                onChange={(e) => handleTypeChange(e.target.value)}
                className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm"
              >
                {ASSET_TYPES.map((t) => (
                  <option key={t.value} value={t.value}>
                    {t.label} ({t.life} mån)
                  </option>
                ))}
              </select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="date">Inköpsdatum</Label>
              <Input
                id="date"
                type="date"
                value={date}
                onChange={(e) => setDate(e.target.value)}
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="cost">Anskaffningsvärde (SEK)</Label>
              <Input
                id="cost"
                type="number"
                step="0.01"
                min="0"
                value={cost}
                onChange={(e) => setCost(e.target.value)}
                placeholder="25000"
                className="font-mono"
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="life">Avskrivningstid (månader)</Label>
              <Input
                id="life"
                type="number"
                min="1"
                value={lifeMonths}
                onChange={(e) => setLifeMonths(parseInt(e.target.value) || 36)}
                className="font-mono"
              />
            </div>
          </div>

          {error && <p className="text-sm text-destructive">{error}</p>}

          <Button type="submit" disabled={mutation.isPending || !name || !cost}>
            {mutation.isPending ? 'Sparar...' : 'Registrera tillgång'}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}

function fmt(v: string): string {
  const n = parseFloat(v);
  if (n === 0) return '-';
  return n.toLocaleString('sv-SE', { minimumFractionDigits: 0, maximumFractionDigits: 0 });
}
