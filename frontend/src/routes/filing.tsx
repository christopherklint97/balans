import { createFileRoute } from '@tanstack/react-router';
import { useQuery, useMutation } from '@tanstack/react-query';
import { useState } from 'react';
import { companiesApi, fiscalYearsApi, filingApi } from '@/api/queries';
import type { FilingResult } from '@/api/types';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';

interface FilingSearch {
  companyId?: string;
  fyId?: string;
}

export const Route = createFileRoute('/filing')({
  component: FilingPage,
  validateSearch: (search: Record<string, unknown>): FilingSearch => ({
    companyId: search.companyId as string | undefined,
    fyId: search.fyId as string | undefined,
  }),
});

function FilingPage() {
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

  const activeFyId = fyId || fiscalYears?.find((fy) => fy.is_closed)?.id;

  if (!activeCompanyId || !activeFyId) {
    return (
      <p className="text-muted-foreground">
        Stäng ett räkenskapsår för att kunna lämna in årsredovisningen.
      </p>
    );
  }

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Lämna in årsredovisning</h1>
      <p className="text-sm text-muted-foreground">
        Generera och lämna in årsredovisningen digitalt till Bolagsverket i iXBRL-format.
      </p>

      <IxbrlPreviewCard fyId={activeFyId} />
      <SubmissionCard fyId={activeFyId} />
    </div>
  );
}

function IxbrlPreviewCard({ fyId }: { fyId: string }) {
  const { data, isLoading } = useQuery({
    queryKey: ['ixbrl-preview', fyId],
    queryFn: () => filingApi.ixbrlPreview(fyId),
  });

  if (isLoading) return <p className="text-muted-foreground">Genererar iXBRL...</p>;
  if (!data) return null;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">iXBRL-dokument</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-2 gap-2 text-sm">
          <div>
            <span className="text-muted-foreground">Företag: </span>
            <span className="font-medium">{data.company_name}</span>
          </div>
          <div>
            <span className="text-muted-foreground">Org.nr: </span>
            <span className="font-mono">{formatOrgNr(data.org_number)}</span>
          </div>
          <div>
            <span className="text-muted-foreground">Räkenskapsår: </span>
            <span>{data.fiscal_year_start} — {data.fiscal_year_end}</span>
          </div>
          <div>
            <span className="text-muted-foreground">Status: </span>
            {data.is_closed ? (
              <Badge className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
                Stängt
              </Badge>
            ) : (
              <Badge variant="secondary">Öppet</Badge>
            )}
          </div>
          <div>
            <span className="text-muted-foreground">Dokumentstorlek: </span>
            <span className="font-mono">{(data.document_size_bytes / 1024).toFixed(1)} KB</span>
          </div>
          <div>
            <span className="text-muted-foreground">SHA-256: </span>
            <span className="font-mono text-xs">{data.checksum_sha256.slice(0, 16)}...</span>
          </div>
        </div>

        <Separator />

        <div className="flex gap-2">
          <a href={filingApi.ixbrlDownloadUrl(fyId)} download>
            <Button variant="outline" size="sm">
              Ladda ner iXBRL
            </Button>
          </a>
        </div>

        {!data.is_closed && (
          <p className="text-sm text-destructive">
            Räkenskapsåret måste vara stängt innan årsredovisningen kan lämnas in.
          </p>
        )}
      </CardContent>
    </Card>
  );
}

function SubmissionCard({ fyId }: { fyId: string }) {
  const [filingResult, setFilingResult] = useState<FilingResult | null>(null);
  const [error, setError] = useState('');
  const [useProduction, setUseProduction] = useState(false);

  const submitMutation = useMutation({
    mutationFn: () => filingApi.submit(fyId, useProduction),
    onSuccess: (data) => {
      setFilingResult(data);
      setError('');
    },
    onError: (err: Error) => setError(err.message),
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Skicka till Bolagsverket</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-muted-foreground">
          Årsredovisningen verifieras och lämnas in digitalt via Bolagsverkets API.
          Processen sker i tre steg: skapa token, verifiera, och lämna in.
        </p>

        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="production"
            checked={useProduction}
            onChange={(e) => setUseProduction(e.target.checked)}
            className="rounded"
          />
          <label htmlFor="production" className="text-sm">
            Använd produktionsmiljön (avmarkerat = testmiljö)
          </label>
        </div>

        {!useProduction && (
          <p className="text-xs text-muted-foreground">
            Testmiljön skickar inte in på riktigt — använd för att verifiera innan skarp inlämning.
          </p>
        )}

        <Button
          onClick={() => submitMutation.mutate()}
          disabled={submitMutation.isPending}
        >
          {submitMutation.isPending
            ? 'Skickar...'
            : useProduction
              ? 'Lämna in (produktion)'
              : 'Testa inlämning'}
        </Button>

        {error && <p className="text-sm text-destructive">{error}</p>}

        {filingResult && (
          <div
            className={`rounded-md border p-4 text-sm ${
              filingResult.success
                ? 'border-green-200 bg-green-50 dark:border-green-900 dark:bg-green-950'
                : 'border-red-200 bg-red-50 dark:border-red-900 dark:bg-red-950'
            }`}
          >
            <div className="flex items-center gap-2 mb-2">
              <Badge
                className={
                  filingResult.success
                    ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                    : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
                }
              >
                {filingResult.success ? 'Inlämnad' : 'Misslyckades'}
              </Badge>
              <span className="font-medium">{filingResult.message}</span>
            </div>

            {filingResult.submission_reference && (
              <p>
                <span className="text-muted-foreground">Ärendenummer: </span>
                <span className="font-mono font-medium">
                  {filingResult.submission_reference}
                </span>
              </p>
            )}

            {filingResult.verification_errors.length > 0 && (
              <div className="mt-2">
                <p className="font-semibold text-destructive">Fel:</p>
                {filingResult.verification_errors.map((e, i) => (
                  <p key={i} className="text-destructive">
                    {e.kod && <span className="font-mono">[{e.kod}] </span>}
                    {e.meddelande}
                  </p>
                ))}
              </div>
            )}

            {filingResult.verification_warnings.length > 0 && (
              <div className="mt-2">
                <p className="font-semibold text-yellow-700 dark:text-yellow-400">Varningar:</p>
                {filingResult.verification_warnings.map((w, i) => (
                  <p key={i} className="text-yellow-700 dark:text-yellow-400">
                    {w.kod && <span className="font-mono">[{w.kod}] </span>}
                    {w.meddelande}
                  </p>
                ))}
              </div>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function formatOrgNr(org: string): string {
  if (org.length === 10 && !org.includes('-')) {
    return `${org.slice(0, 6)}-${org.slice(6)}`;
  }
  return org;
}
