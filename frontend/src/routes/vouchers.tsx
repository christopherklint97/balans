import { createFileRoute } from '@tanstack/react-router';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState, useRef, useEffect, useCallback } from 'react';
import { vouchersApi, accountsApi, companiesApi, fiscalYearsApi, reportsApi } from '@/api/queries';
import type { Account, CreateVoucherLine } from '@/api/types';
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
import { formatSEK, parseSEK, normalizeAmountInput } from '@/lib/format';

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

// --- Account Autocomplete ---

function AccountAutocomplete({
  value,
  onChange,
  accounts,
  placeholder,
  className,
}: {
  value: string;
  onChange: (value: string) => void;
  accounts: Account[] | undefined;
  placeholder?: string;
  className?: string;
}) {
  const [open, setOpen] = useState(false);
  const [highlightIndex, setHighlightIndex] = useState(0);
  const wrapperRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const filtered = accounts?.filter((a) => {
    if (!value) return true;
    const q = value.toLowerCase();
    return a.number.toString().startsWith(q) || a.name.toLowerCase().includes(q);
  }).slice(0, 10) ?? [];

  useEffect(() => {
    setHighlightIndex(0);
  }, [value]);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (wrapperRef.current && !wrapperRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const selectAccount = useCallback((account: Account) => {
    onChange(account.number.toString());
    setOpen(false);
  }, [onChange]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!open || filtered.length === 0) return;

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setHighlightIndex((prev) => Math.min(prev + 1, filtered.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setHighlightIndex((prev) => Math.max(prev - 1, 0));
    } else if (e.key === 'Enter' || e.key === 'Tab') {
      if (filtered[highlightIndex]) {
        e.preventDefault();
        selectAccount(filtered[highlightIndex]);
      }
    } else if (e.key === 'Escape') {
      setOpen(false);
    }
  };

  return (
    <div ref={wrapperRef} className="relative">
      <Input
        ref={inputRef}
        value={value}
        onChange={(e) => {
          onChange(e.target.value);
          setOpen(true);
        }}
        onFocus={() => setOpen(true)}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        className={className}
        inputMode="numeric"
        autoComplete="off"
      />
      {open && filtered.length > 0 && (
        <div className="absolute z-50 top-full left-0 mt-1 w-64 max-h-48 overflow-auto rounded-md border border-border bg-popover shadow-md">
          {filtered.map((a, idx) => (
            <button
              key={a.id}
              type="button"
              className={`w-full text-left px-2 py-1.5 text-sm flex items-center gap-2 hover:bg-accent ${
                idx === highlightIndex ? 'bg-accent' : ''
              }`}
              onMouseDown={(e) => {
                e.preventDefault();
                selectAccount(a);
              }}
              onMouseEnter={() => setHighlightIndex(idx)}
            >
              <span className="font-mono font-medium w-12">{a.number}</span>
              <span className="truncate text-muted-foreground">{a.name}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

// --- Amount Input (Swedish comma format) ---

function AmountInput({
  value,
  onChange,
  placeholder,
  className,
}: {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
}) {
  const [displayValue, setDisplayValue] = useState(() =>
    value ? value.replace('.', ',') : '',
  );
  const prevValueRef = useRef(value);
  const isFocusedRef = useRef(false);

  // Sync display when value changes externally (e.g. cleared by debit/credit toggle)
  if (prevValueRef.current !== value) {
    prevValueRef.current = value;
    if (!isFocusedRef.current) {
      const expected = value ? value.replace('.', ',') : '';
      if (displayValue !== expected) {
        setDisplayValue(expected);
      }
    }
  }

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const raw = e.target.value;
    const normalized = normalizeAmountInput(raw);
    setDisplayValue(raw.replace('.', ',').replace(/[^\d,]/g, ''));
    onChange(normalized);
  };

  const handleBlur = () => {
    isFocusedRef.current = false;
    if (!value) {
      setDisplayValue('');
      return;
    }
    const num = parseSEK(value);
    if (num > 0) {
      setDisplayValue(formatSEK(num));
    } else {
      setDisplayValue('');
      onChange('');
    }
  };

  const handleFocus = () => {
    isFocusedRef.current = true;
    if (value) {
      const num = parseSEK(value);
      if (num > 0) {
        setDisplayValue(num.toString().replace('.', ','));
      }
    }
  };

  return (
    <Input
      value={displayValue}
      onChange={handleChange}
      onBlur={handleBlur}
      onFocus={handleFocus}
      placeholder={placeholder}
      className={className}
      inputMode="decimal"
      autoComplete="off"
    />
  );
}

// --- File Upload (Underlag) ---

interface UnderlagFile {
  file: File;
  preview: string | null;
}

function UnderlagUpload({
  files,
  onChange,
}: {
  files: UnderlagFile[];
  onChange: (files: UnderlagFile[]) => void;
}) {
  const inputRef = useRef<HTMLInputElement>(null);

  const addFiles = async (fileList: FileList) => {
    const newFiles: UnderlagFile[] = [];
    for (const file of Array.from(fileList)) {
      let preview: string | null = null;
      if (file.type.startsWith('image/') || file.type === 'application/pdf') {
        preview = URL.createObjectURL(file);
      }
      newFiles.push({ file, preview });
    }
    onChange([...files, ...newFiles]);
  };

  const removeFile = (index: number) => {
    const removed = files[index];
    if (removed.preview) URL.revokeObjectURL(removed.preview);
    onChange(files.filter((_, i) => i !== index));
  };

  return (
    <div className="space-y-2">
      <Label>Underlag</Label>
      <div className="flex flex-wrap gap-3">
        {files.map((f, i) => (
          <div key={i} className="relative group">
            <div className="w-20 h-20 rounded-md border border-border overflow-hidden bg-muted flex items-center justify-center">
              {f.preview && f.file.type.startsWith('image/') ? (
                <img src={f.preview} alt={f.file.name} className="w-full h-full object-cover" />
              ) : f.preview && f.file.type === 'application/pdf' ? (
                <div className="text-center p-1">
                  <svg className="w-6 h-6 mx-auto text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
                  </svg>
                  <span className="text-[10px] text-muted-foreground">PDF</span>
                </div>
              ) : (
                <div className="text-center p-1">
                  <svg className="w-6 h-6 mx-auto text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
                  </svg>
                  <span className="text-[10px] text-muted-foreground truncate block">{f.file.name.split('.').pop()?.toUpperCase()}</span>
                </div>
              )}
            </div>
            <button
              type="button"
              onClick={() => removeFile(i)}
              className="absolute -top-1.5 -right-1.5 w-5 h-5 rounded-full bg-destructive text-destructive-foreground text-xs flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity"
            >
              x
            </button>
            <p className="text-[10px] text-muted-foreground truncate w-20 mt-0.5">{f.file.name}</p>
          </div>
        ))}
        <button
          type="button"
          onClick={() => inputRef.current?.click()}
          className="w-20 h-20 rounded-md border-2 border-dashed border-border hover:border-ring flex flex-col items-center justify-center text-muted-foreground hover:text-foreground transition-colors"
        >
          <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 4v16m8-8H4" />
          </svg>
          <span className="text-[10px]">Lägg till</span>
        </button>
      </div>
      <input
        ref={inputRef}
        type="file"
        multiple
        accept="image/*,application/pdf,.doc,.docx,.xls,.xlsx"
        className="hidden"
        onChange={(e) => {
          if (e.target.files) addFiles(e.target.files);
          e.target.value = '';
        }}
      />
    </div>
  );
}

// --- Voucher Form ---

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
  const [underlag, setUnderlag] = useState<UnderlagFile[]>([]);

  const { data: accounts } = useQuery({
    queryKey: ['accounts', companyId],
    queryFn: () => accountsApi.list(companyId),
  });

  const totalDebit = lines.reduce((sum, l) => sum + (parseSEK(l.debit)), 0);
  const totalCredit = lines.reduce((sum, l) => sum + (parseSEK(l.credit)), 0);
  const isBalanced = Math.abs(totalDebit - totalCredit) < 0.005 && totalDebit > 0;

  const mutation = useMutation({
    mutationFn: () => {
      const voucherLines: CreateVoucherLine[] = lines
        .filter((l) => l.account_number && (l.debit || l.credit))
        .map((l) => ({
          account_number: parseInt(l.account_number, 10),
          debit: (parseSEK(l.debit)).toFixed(2),
          credit: (parseSEK(l.credit)).toFixed(2),
        }));
      return vouchersApi.create(fyId, { date, description, lines: voucherLines });
    },
    onSuccess: async (result) => {
      // Upload underlag files if any
      if (underlag.length > 0) {
        for (const u of underlag) {
          const formData = new FormData();
          formData.append('file', u.file);
          try {
            const token = localStorage.getItem('balans_token');
            await fetch(`/api/vouchers/${result.id}/attachments`, {
              method: 'POST',
              headers: token ? { Authorization: `Bearer ${token}` } : {},
              body: formData,
            });
          } catch {
            // Attachment upload failed silently — voucher was still created
          }
        }
      }
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
          <div className="grid gap-4 sm:grid-cols-[180px_1fr]">
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
            <div className="hidden sm:grid grid-cols-[120px_1fr_120px_120px_40px] gap-2 text-sm font-medium text-muted-foreground">
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
                  <div className="hidden sm:grid grid-cols-[120px_1fr_120px_120px_40px] gap-2">
                    <AccountAutocomplete
                      value={line.account_number}
                      onChange={(v) => updateLine(i, 'account_number', v)}
                      accounts={accounts}
                      placeholder="1910"
                      className="font-mono"
                    />
                    <span className="flex items-center text-sm text-muted-foreground truncate">
                      {matchedAccount?.name || ''}
                    </span>
                    <AmountInput
                      value={line.debit}
                      onChange={(v) => {
                        updateLine(i, 'debit', v);
                        if (v) updateLine(i, 'credit', '');
                      }}
                      placeholder="0,00"
                      className="font-mono text-right"
                    />
                    <AmountInput
                      value={line.credit}
                      onChange={(v) => {
                        updateLine(i, 'credit', v);
                        if (v) updateLine(i, 'debit', '');
                      }}
                      placeholder="0,00"
                      className="font-mono text-right"
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
                        <AccountAutocomplete
                          value={line.account_number}
                          onChange={(v) => updateLine(i, 'account_number', v)}
                          accounts={accounts}
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
                        <AmountInput
                          value={line.debit}
                          onChange={(v) => {
                            updateLine(i, 'debit', v);
                            if (v) updateLine(i, 'credit', '');
                          }}
                          placeholder="0,00"
                          className="font-mono text-right"
                        />
                      </div>
                      <div className="space-y-1">
                        <span className="text-xs text-muted-foreground">Kredit</span>
                        <AmountInput
                          value={line.credit}
                          onChange={(v) => {
                            updateLine(i, 'credit', v);
                            if (v) updateLine(i, 'debit', '');
                          }}
                          placeholder="0,00"
                          className="font-mono text-right"
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

          <UnderlagUpload files={underlag} onChange={setUnderlag} />

          <Separator />

          <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex flex-wrap gap-x-4 gap-y-1 text-sm">
              <span>
                Debet: <span className="font-mono font-medium">{formatSEK(totalDebit)}</span>
              </span>
              <span>
                Kredit: <span className="font-mono font-medium">{formatSEK(totalCredit)}</span>
              </span>
              <span className={isBalanced ? 'text-green-600' : 'text-destructive'}>
                Diff: {formatSEK(totalDebit - totalCredit)}
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
                <TableCell className="text-right font-mono">{formatSEK(r.debit_total)}</TableCell>
                <TableCell className="text-right font-mono">{formatSEK(r.credit_total)}</TableCell>
                <TableCell className="text-right font-mono">{formatSEK(r.balance)}</TableCell>
              </TableRow>
            ))}
            <TableRow className="font-semibold">
              <TableCell></TableCell>
              <TableCell>Summa</TableCell>
              <TableCell className="text-right font-mono">{formatSEK(totalDebit)}</TableCell>
              <TableCell className="text-right font-mono">{formatSEK(totalCredit)}</TableCell>
              <TableCell className="text-right font-mono">
                {formatSEK(totalDebit - totalCredit)}
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}
