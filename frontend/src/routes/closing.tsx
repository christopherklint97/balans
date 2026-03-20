import { createFileRoute } from '@tanstack/react-router';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { companiesApi, fiscalYearsApi, closingApi } from '@/api/queries';
import type { ValidationResult, ClosingResult } from '@/api/types';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Separator } from '@/components/ui/separator';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

interface ClosingSearch {
  companyId?: string;
  fyId?: string;
}

export const Route = createFileRoute('/closing')({
  component: ClosingPage,
  validateSearch: (search: Record<string, unknown>): ClosingSearch => ({
    companyId: search.companyId as string | undefined,
    fyId: search.fyId as string | undefined,
  }),
});

function ClosingPage() {
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

  const activeFyId = fyId || fiscalYears?.find((fy) => !fy.is_closed)?.id;
  const activeFy = fiscalYears?.find((fy) => fy.id === activeFyId);

  if (!activeCompanyId || !activeFyId || !activeFy) {
    return (
      <p className="text-muted-foreground">
        Skapa ett företag och räkenskapsår först.
      </p>
    );
  }

  if (activeFy.is_closed) {
    return <ClosedView fyId={activeFyId} startDate={activeFy.start_date} endDate={activeFy.end_date} />;
  }

  return (
    <ClosingWizard
      companyId={activeCompanyId}
      fyId={activeFyId}
      startDate={activeFy.start_date}
      endDate={activeFy.end_date}
    />
  );
}

function ClosedView({ fyId, startDate, endDate }: { fyId: string; startDate: string; endDate: string }) {
  const { data: status } = useQuery({
    queryKey: ['closing-status', fyId],
    queryFn: () => closingApi.status(fyId),
  });

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Årsbokslut</h1>
      <Card>
        <CardContent className="pt-6">
          <div className="flex items-center gap-3">
            <Badge className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
              Stängt
            </Badge>
            <span className="text-sm text-muted-foreground">
              Räkenskapsår {startDate} — {endDate}
            </span>
          </div>
          {status && (
            <div className="mt-4 text-sm text-muted-foreground space-y-1">
              <p>Stängt: {status.closed_at?.slice(0, 10)}</p>
              <p>{status.closing_voucher_count} bokslutsverifikationer, {status.total_voucher_count} totalt</p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function ClosingWizard({
  fyId,
  startDate,
  endDate,
}: {
  companyId: string; // reserved for future use
  fyId: string;
  startDate: string;
  endDate: string;
}) {
  const queryClient = useQueryClient();
  const [step, setStep] = useState<'validate' | 'review' | 'done'>('validate');
  const [validation, setValidation] = useState<ValidationResult | null>(null);
  const [closingResult, setClosingResult] = useState<ClosingResult | null>(null);
  const [customTax, setCustomTax] = useState('');
  const [carryForward, setCarryForward] = useState(true);
  const [error, setError] = useState('');

  const validateMutation = useMutation({
    mutationFn: () => closingApi.validate(fyId),
    onSuccess: (data) => {
      setValidation(data);
      setCustomTax(data.summary.estimated_tax);
      setStep('review');
      setError('');
    },
    onError: (err: Error) => setError(err.message),
  });

  const executeMutation = useMutation({
    mutationFn: () =>
      closingApi.execute(fyId, {
        tax_amount: customTax || undefined,
        carry_forward: carryForward,
      }),
    onSuccess: (data) => {
      setClosingResult(data);
      setStep('done');
      setError('');
      queryClient.invalidateQueries({ queryKey: ['fiscal-years'] });
      queryClient.invalidateQueries({ queryKey: ['vouchers'] });
      queryClient.invalidateQueries({ queryKey: ['closing-status'] });
    },
    onError: (err: Error) => setError(err.message),
  });

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Årsbokslut</h1>
        <span className="text-sm text-muted-foreground">
          {startDate} — {endDate}
        </span>
      </div>

      {/* Step indicators */}
      <div className="flex gap-2 text-sm">
        <Badge variant={step === 'validate' ? 'default' : 'secondary'}>1. Validera</Badge>
        <Badge variant={step === 'review' ? 'default' : 'secondary'}>2. Granska</Badge>
        <Badge variant={step === 'done' ? 'default' : 'secondary'}>3. Klart</Badge>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {step === 'validate' && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Steg 1: Validera bokföringen</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <p className="text-sm text-muted-foreground">
              Kontrollera att bokföringen är korrekt och redo för årsbokslut.
              Alla verifikationer måste vara balanserade.
            </p>
            <Button onClick={() => validateMutation.mutate()} disabled={validateMutation.isPending}>
              {validateMutation.isPending ? 'Validerar...' : 'Kör validering'}
            </Button>
          </CardContent>
        </Card>
      )}

      {step === 'review' && validation && (
        <>
          {/* Validation issues */}
          {(validation.errors.length > 0 || validation.warnings.length > 0) && (
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Valideringsresultat</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                {validation.errors.map((e, i) => (
                  <div key={i} className="flex items-start gap-2 text-sm text-destructive">
                    <span className="font-mono text-xs mt-0.5">FEL</span>
                    <span>{e.message}</span>
                  </div>
                ))}
                {validation.warnings.map((w, i) => (
                  <div key={i} className="flex items-start gap-2 text-sm text-yellow-600 dark:text-yellow-400">
                    <span className="font-mono text-xs mt-0.5">VARNING</span>
                    <span>{w.message}</span>
                  </div>
                ))}
              </CardContent>
            </Card>
          )}

          {/* Financial summary */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Resultatsammanställning</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2 text-sm">
                <SummaryRow label="Nettoomsättning" value={validation.summary.total_revenue} />
                <SummaryRow label="Rörelsekostnader" value={`-${validation.summary.total_expenses}`} negative />
                <Separator />
                <SummaryRow label="Rörelseresultat" value={validation.summary.operating_result} bold />
                <SummaryRow label="Finansiella intäkter" value={validation.summary.financial_income} />
                <SummaryRow label="Finansiella kostnader" value={`-${validation.summary.financial_expenses}`} negative />
                <Separator />
                <SummaryRow label="Resultat före skatt" value={validation.summary.result_before_tax} bold />
                <SummaryRow label="Beräknad bolagsskatt (20.6%)" value={`-${validation.summary.estimated_tax}`} negative />
                <Separator />
                <SummaryRow label="Årets resultat" value={validation.summary.net_result} bold />
              </div>
            </CardContent>
          </Card>

          {/* Closing parameters */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Steg 2: Bokslutsparametrar</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="tax">Bolagsskatt (SEK)</Label>
                <Input
                  id="tax"
                  type="number"
                  step="1"
                  min="0"
                  value={customTax}
                  onChange={(e) => setCustomTax(e.target.value)}
                  className="max-w-[200px] font-mono"
                />
                <p className="text-xs text-muted-foreground">
                  Justera om den beräknade skatten behöver korrigeras.
                </p>
              </div>

              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="carry"
                  checked={carryForward}
                  onChange={(e) => setCarryForward(e.target.checked)}
                  className="rounded"
                />
                <Label htmlFor="carry" className="text-sm font-normal">
                  Överför ingående balanser till nästa räkenskapsår
                </Label>
              </div>

              <Separator />

              <div className="flex gap-2">
                <Button
                  onClick={() => executeMutation.mutate()}
                  disabled={executeMutation.isPending || !validation.passed}
                >
                  {executeMutation.isPending ? 'Stänger...' : 'Genomför årsbokslut'}
                </Button>
                <Button
                  variant="outline"
                  onClick={() => {
                    setStep('validate');
                    setValidation(null);
                  }}
                >
                  Tillbaka
                </Button>
              </div>
            </CardContent>
          </Card>
        </>
      )}

      {step === 'done' && closingResult && (
        <Card>
          <CardContent className="pt-6 space-y-4">
            <div className="flex items-center gap-3">
              <Badge className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
                Klart
              </Badge>
              <span className="font-medium">Årsbokslutet är genomfört</span>
            </div>

            <div className="space-y-2 text-sm">
              <p className="text-muted-foreground">Skapade bokslutsverifikationer:</p>
              {closingResult.closing_vouchers.map((v) => (
                <div key={v.voucher_id} className="flex justify-between rounded-md border border-border p-2">
                  <span>
                    #{v.voucher_number} — {v.description}
                  </span>
                  <span className="font-mono">{v.total_amount} SEK</span>
                </div>
              ))}
            </div>

            {closingResult.next_fiscal_year_id && (
              <p className="text-sm text-muted-foreground">
                Nytt räkenskapsår skapat med ingående balanser.
              </p>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function SummaryRow({
  label,
  value,
  bold,
  negative,
}: {
  label: string;
  value: string;
  bold?: boolean;
  negative?: boolean;
}) {
  return (
    <div className={`flex justify-between ${bold ? 'font-semibold' : ''}`}>
      <span>{label}</span>
      <span className={`font-mono ${negative ? 'text-muted-foreground' : ''}`}>
        {value}
      </span>
    </div>
  );
}
