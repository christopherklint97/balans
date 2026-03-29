import { createFileRoute } from '@tanstack/react-router';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState, useRef } from 'react';
import { sieApi } from '@/api/queries';
import { useFiscalYear } from '@/hooks/use-fiscal-year';
import type { SieImportPreview, SieImportResult } from '@/api/types';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Separator } from '@/components/ui/separator';
import { Badge } from '@/components/ui/badge';

export const Route = createFileRoute('/sie')({
  component: SiePage,
});

function SiePage() {
  const { activeCompanyId, activeFyId } = useFiscalYear();

  if (!activeCompanyId) {
    return <p className="text-muted-foreground">Skapa ett företag först.</p>;
  }

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">SIE Import/Export</h1>

      <div className="grid gap-6 md:grid-cols-2">
        <SieImport companyId={activeCompanyId} />
        <SieExport fyId={activeFyId} />
      </div>
    </div>
  );
}

function SieImport({ companyId }: { companyId: string }) {
  const queryClient = useQueryClient();
  const fileRef = useRef<HTMLInputElement>(null);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [preview, setPreview] = useState<SieImportPreview | null>(null);
  const [result, setResult] = useState<SieImportResult | null>(null);
  const [error, setError] = useState('');

  const previewMutation = useMutation({
    mutationFn: (file: File) => sieApi.preview(companyId, file),
    onSuccess: (data) => {
      setPreview(data);
      setError('');
    },
    onError: (err: Error) => {
      setError(err.message);
      setPreview(null);
    },
  });

  const importMutation = useMutation({
    mutationFn: (file: File) => sieApi.import(companyId, file),
    onSuccess: (data) => {
      setResult(data);
      setPreview(null);
      setSelectedFile(null);
      setError('');
      queryClient.invalidateQueries({ queryKey: ['accounts'] });
      queryClient.invalidateQueries({ queryKey: ['vouchers'] });
      queryClient.invalidateQueries({ queryKey: ['fiscal-years'] });
    },
    onError: (err: Error) => setError(err.message),
  });

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setSelectedFile(file);
      setPreview(null);
      setResult(null);
      setError('');
      previewMutation.mutate(file);
    }
  };

  const reset = () => {
    setSelectedFile(null);
    setPreview(null);
    setResult(null);
    setError('');
    if (fileRef.current) fileRef.current.value = '';
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Importera SIE-fil</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-muted-foreground">
          Ladda upp en SIE-fil (typ 1-4) från ett annat bokföringsprogram.
        </p>

        <input
          ref={fileRef}
          type="file"
          accept=".se,.si,.sie"
          onChange={handleFileChange}
          className="block w-full text-sm file:mr-4 file:rounded-md file:border-0 file:bg-secondary file:px-4 file:py-2 file:text-sm file:font-medium hover:file:bg-secondary/80"
        />

        {previewMutation.isPending && (
          <p className="text-sm text-muted-foreground">Analyserar fil...</p>
        )}

        {error && <p className="text-sm text-destructive">{error}</p>}

        {preview && (
          <div className="space-y-3 rounded-md border border-border p-4">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Förhandsgranskning</span>
              <Badge variant="secondary">SIE typ {preview.sie_type}</Badge>
            </div>

            {preview.company_name && (
              <p className="text-sm">
                <span className="text-muted-foreground">Företag: </span>
                {preview.company_name}
              </p>
            )}
            {preview.org_number && (
              <p className="text-sm">
                <span className="text-muted-foreground">Orgnr: </span>
                {preview.org_number}
              </p>
            )}

            {preview.fiscal_years.length > 0 && (
              <div className="text-sm">
                <span className="text-muted-foreground">Räkenskapsår: </span>
                {preview.fiscal_years.map((fy) => (
                  <span key={fy.index} className="mr-2">
                    {fy.start_date} — {fy.end_date}
                  </span>
                ))}
              </div>
            )}

            <Separator />

            <div className="grid grid-cols-2 gap-2 text-sm">
              <div>
                <span className="text-muted-foreground">Konton: </span>
                <span className="font-medium">{preview.account_count}</span>
              </div>
              <div>
                <span className="text-muted-foreground">Verifikationer: </span>
                <span className="font-medium">{preview.voucher_count}</span>
              </div>
              <div>
                <span className="text-muted-foreground">Transaktioner: </span>
                <span className="font-medium">{preview.transaction_count}</span>
              </div>
              <div>
                <span className="text-muted-foreground">IB/UB: </span>
                <span className="font-medium">
                  {preview.opening_balances} / {preview.closing_balances}
                </span>
              </div>
            </div>

            <div className="flex gap-2 pt-2">
              <Button
                size="sm"
                onClick={() => selectedFile && importMutation.mutate(selectedFile)}
                disabled={importMutation.isPending}
              >
                {importMutation.isPending ? 'Importerar...' : 'Importera'}
              </Button>
              <Button size="sm" variant="outline" onClick={reset}>
                Avbryt
              </Button>
            </div>
          </div>
        )}

        {result && (
          <div className="rounded-md border border-green-200 bg-green-50 p-4 text-sm dark:border-green-900 dark:bg-green-950">
            <p className="font-medium text-green-800 dark:text-green-200">Import klar</p>
            <p className="text-green-700 dark:text-green-300">
              {result.accounts_imported} nya konton, {result.vouchers_imported} verifikationer
              importerade.
            </p>
            <Button size="sm" variant="outline" className="mt-2" onClick={reset}>
              Importera en till
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function SieExport({ fyId }: { fyId: string | undefined }) {
  if (!fyId) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Exportera SIE-fil</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">
            Skapa ett räkenskapsår för att kunna exportera.
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Exportera SIE-fil</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-muted-foreground">
          Exportera bokföringsdata i SIE-format för import i andra program.
        </p>

        <div className="space-y-3">
          <div className="flex items-center justify-between rounded-md border border-border p-3">
            <div>
              <p className="text-sm font-medium">SIE Typ 1 — Årssaldon</p>
              <p className="text-xs text-muted-foreground">
                Utgående balanser per konto
              </p>
            </div>
            <a href={sieApi.exportUrl(fyId, '1')} download>
              <Button size="sm" variant="outline">
                Ladda ner
              </Button>
            </a>
          </div>

          <div className="flex items-center justify-between rounded-md border border-border p-3">
            <div>
              <p className="text-sm font-medium">SIE Typ 4 — Transaktioner</p>
              <p className="text-xs text-muted-foreground">
                Alla verifikationer med kontoplan och saldon
              </p>
            </div>
            <a href={sieApi.exportUrl(fyId, '4')} download>
              <Button size="sm" variant="outline">
                Ladda ner
              </Button>
            </a>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
