import { createFileRoute } from '@tanstack/react-router';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState, useRef, useEffect, useCallback } from 'react';
import { toast } from 'sonner';
import { vouchersApi, accountsApi, reportsApi, attachmentsApi } from '@/api/queries';
import { useFiscalYear } from '@/hooks/use-fiscal-year';
import type { Account, AttachmentMeta, CreateVoucherLine, VoucherWithLines } from '@/api/types';
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
  view?: 'list' | 'new' | 'balance' | 'detail';
  voucherId?: string;
}

export const Route = createFileRoute('/vouchers')({
  component: VouchersPage,
  validateSearch: (search: Record<string, unknown>): VouchersSearch => ({
    view: (search.view as VouchersSearch['view']) || 'list',
    voucherId: search.voucherId as string | undefined,
  }),
});

function VouchersPage() {
  const { view, voucherId } = Route.useSearch();
  const navigate = Route.useNavigate();
  const { activeCompanyId, activeFyId, activeFy } = useFiscalYear();

  if (!activeCompanyId || !activeFyId) {
    return <p className="text-muted-foreground">Skapa ett företag och räkenskapsår först.</p>;
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <h1 className="text-2xl font-semibold">Verifikationer</h1>
        <div className="flex flex-wrap gap-2">
          <Button
            variant={view === 'list' || view === 'detail' ? 'default' : 'outline'}
            size="sm"
            onClick={() => navigate({ search: { view: 'list' } })}
          >
            Lista
          </Button>
          <Button
            variant={view === 'new' ? 'default' : 'outline'}
            size="sm"
            onClick={() => navigate({ search: { view: 'new' } })}
          >
            Ny verifikation
          </Button>
          <Button
            variant={view === 'balance' ? 'default' : 'outline'}
            size="sm"
            onClick={() => navigate({ search: { view: 'balance' } })}
          >
            Saldobalans
          </Button>
        </div>
      </div>

      {view === 'new' ? (
        <VoucherForm
          companyId={activeCompanyId}
          fyId={activeFyId}
          onSuccess={() => navigate({ search: { view: 'list' } })}
        />
      ) : view === 'balance' ? (
        <TrialBalance fyId={activeFyId} />
      ) : view === 'detail' && voucherId ? (
        <VoucherDetail
          voucherId={voucherId}
          companyId={activeCompanyId}
          fyId={activeFyId}
          isFyClosed={activeFy?.is_closed ?? false}
          onBack={() => navigate({ search: { view: 'list' } })}
        />
      ) : (
        <VoucherList
          fyId={activeFyId}
          onSelect={(id) => navigate({ search: { view: 'detail', voucherId: id } })}
        />
      )}
    </div>
  );
}

function VoucherList({ fyId, onSelect }: { fyId: string; onSelect: (id: string) => void }) {
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
              <TableRow
                key={v.id}
                className="cursor-pointer"
                onClick={() => onSelect(v.id)}
              >
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

// --- Attachment Preview (fetches via Bearer auth) ---

function AttachmentPreview({ voucherId, attachment }: { voucherId: string; attachment: AttachmentMeta }) {
  const [blobUrl, setBlobUrl] = useState<string | null>(null);
  const blobUrlRef = useRef<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const url = attachmentsApi.downloadUrl(voucherId, attachment.id);
    const token = localStorage.getItem('balans_token');
    fetch(url, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    })
      .then((res) =>
        res.ok ? res.blob() : Promise.reject(new Error('Kunde inte ladda bilaga')),
      )
      .then((blob) => {
        if (!cancelled) {
          const objectUrl = URL.createObjectURL(blob);
          blobUrlRef.current = objectUrl;
          setBlobUrl(objectUrl);
        }
      })
      .catch((err: Error) => {
        if (!cancelled) toast.error(err.message);
      });
    return () => {
      cancelled = true;
      if (blobUrlRef.current) URL.revokeObjectURL(blobUrlRef.current);
    };
  }, [voucherId, attachment.id]);

  return (
    <div className="rounded-md border border-border overflow-hidden">
      {blobUrl && attachment.content_type.startsWith('image/') ? (
        <img src={blobUrl} alt={attachment.filename} className="w-full object-contain max-h-[500px]" />
      ) : blobUrl && attachment.content_type === 'application/pdf' ? (
        <iframe src={blobUrl} title={attachment.filename} className="w-full h-[500px] border-0" />
      ) : (
        <div className="text-center p-6">
          <p className="text-sm text-muted-foreground">{attachment.filename}</p>
        </div>
      )}
      <div className="px-3 py-1.5 bg-muted/50">
        <p className="text-xs text-muted-foreground">{attachment.filename}</p>
      </div>
    </div>
  );
}

// --- Voucher Detail (read-only) ---

function VoucherDetail({
  voucherId,
  companyId,
  fyId,
  isFyClosed,
  onBack,
}: {
  voucherId: string;
  companyId: string;
  fyId: string;
  isFyClosed: boolean;
  onBack: () => void;
}) {
  const queryClient = useQueryClient();
  const navigate = Route.useNavigate();
  const { data: voucher, isLoading } = useQuery({
    queryKey: ['voucher', voucherId],
    queryFn: () => vouchersApi.get(voucherId),
  });

  const { data: attachments } = useQuery({
    queryKey: ['attachments', voucherId],
    queryFn: () => attachmentsApi.list(voucherId),
  });

  const { data: accounts } = useQuery({
    queryKey: ['accounts', companyId],
    queryFn: () => accountsApi.list(companyId),
  });

  const [showConfirm, setShowConfirm] = useState(false);
  const [error, setError] = useState('');

  const strykaMutation = useMutation({
    mutationFn: (original: VoucherWithLines) => {
      const today = new Date().toISOString().slice(0, 10);
      const lines: CreateVoucherLine[] = original.lines.map((l) => ({
        account_number: l.account_number,
        debit: l.credit,
        credit: l.debit,
      }));
      return vouchersApi.create(fyId, {
        date: today,
        description: `Strykning av verifikation ${original.voucher_number}`,
        lines,
      });
    },
    onSuccess: (result) => {
      queryClient.invalidateQueries({ queryKey: ['vouchers', fyId] });
      queryClient.invalidateQueries({ queryKey: ['trial-balance', fyId] });
      navigate({
        search: {
          view: 'detail' as const,
          voucherId: result.id,
        },
      });
    },
    onError: (err: Error) => setError(err.message),
  });

  if (isLoading) return <p className="text-muted-foreground">Laddar...</p>;
  if (!voucher) return <p className="text-muted-foreground">Verifikationen hittades inte.</p>;

  const accountName = (num: number) => accounts?.find((a) => a.number === num)?.name ?? '';

  const totalDebit = voucher.lines.reduce((s, l) => s + parseFloat(l.debit), 0);
  const totalCredit = voucher.lines.reduce((s, l) => s + parseFloat(l.credit), 0);

  return (
    <div className="flex flex-col lg:flex-row gap-6 lg:h-[calc(100vh-10rem)]">
      {/* Left: Voucher details */}
      <Card className="flex-1 min-w-0 lg:w-1/2 lg:flex lg:flex-col lg:overflow-hidden">
        <CardHeader className="flex flex-row items-center justify-between space-y-0">
          <div className="flex items-center gap-3">
            <Button variant="ghost" size="sm" onClick={onBack}>
              &larr; Tillbaka
            </Button>
            <CardTitle className="text-base">
              Verifikation #{voucher.voucher_number}
            </CardTitle>
          </div>
        </CardHeader>
        <CardContent className="space-y-4 lg:overflow-y-auto lg:flex-1">
          <div className="grid gap-4 sm:grid-cols-[150px_1fr]">
            <div className="space-y-1">
              <Label className="text-muted-foreground text-xs">Datum</Label>
              <p className="text-sm">{voucher.date}</p>
            </div>
            <div className="space-y-1">
              <Label className="text-muted-foreground text-xs">Beskrivning</Label>
              <p className="text-sm">{voucher.description}</p>
            </div>
          </div>

          <Separator />

          {/* Voucher lines table */}
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-20">Konto</TableHead>
                <TableHead>Kontonamn</TableHead>
                <TableHead className="text-right w-28">Debet</TableHead>
                <TableHead className="text-right w-28">Kredit</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {voucher.lines.map((line) => (
                <TableRow key={line.id}>
                  <TableCell className="font-mono">{line.account_number}</TableCell>
                  <TableCell className="text-muted-foreground">{accountName(line.account_number)}</TableCell>
                  <TableCell className="text-right font-mono">
                    {parseFloat(line.debit) > 0 ? formatSEK(line.debit) : ''}
                  </TableCell>
                  <TableCell className="text-right font-mono">
                    {parseFloat(line.credit) > 0 ? formatSEK(line.credit) : ''}
                  </TableCell>
                </TableRow>
              ))}
              <TableRow className="font-semibold">
                <TableCell />
                <TableCell>Summa</TableCell>
                <TableCell className="text-right font-mono">{formatSEK(totalDebit)}</TableCell>
                <TableCell className="text-right font-mono">{formatSEK(totalCredit)}</TableCell>
              </TableRow>
            </TableBody>
          </Table>

          <Separator />

          {/* Stryka section */}
          {!isFyClosed && (
            <div className="space-y-2">
              {!showConfirm ? (
                <Button
                  variant="destructive"
                  size="sm"
                  onClick={() => setShowConfirm(true)}
                >
                  Stryka verifikation
                </Button>
              ) : (
                <div className="flex flex-col gap-2 rounded-md border border-destructive/50 p-3">
                  <p className="text-sm">
                    Detta skapar en ny verifikation som nollställer alla belopp i denna verifikation. Vill du fortsätta?
                  </p>
                  <div className="flex gap-2">
                    <Button
                      variant="destructive"
                      size="sm"
                      disabled={strykaMutation.isPending}
                      onClick={() => strykaMutation.mutate(voucher)}
                    >
                      {strykaMutation.isPending ? 'Skapar...' : 'Ja, stryka'}
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setShowConfirm(false)}
                    >
                      Avbryt
                    </Button>
                  </div>
                </div>
              )}
              {error && <p className="text-sm text-destructive">{error}</p>}
            </div>
          )}

          <p className="text-xs text-muted-foreground">
            Skapad: {voucher.created_at}
          </p>
        </CardContent>
      </Card>

      {/* Right: Attachments panel (desktop) */}
      {attachments && attachments.length > 0 && (
        <div className="lg:w-1/2 lg:min-w-0">
          <Card className="flex flex-col w-full overflow-hidden lg:h-full">
            <CardHeader className="pb-3">
              <CardTitle className="text-base">Underlag</CardTitle>
            </CardHeader>
            <CardContent className="flex-1 overflow-y-auto space-y-3">
              {attachments.map((att) => (
                <AttachmentPreview key={att.id} voucherId={voucherId} attachment={att} />
              ))}
            </CardContent>
          </Card>
        </div>
      )}
    </div>
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
          setHighlightIndex(0);
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
  const [isFocused, setIsFocused] = useState(false);
  const [localDisplay, setLocalDisplay] = useState('');

  // When not focused, derive display from value prop directly
  const displayValue = isFocused
    ? localDisplay
    : value ? value.replace('.', ',') : '';

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const raw = e.target.value;
    const normalized = normalizeAmountInput(raw);
    setLocalDisplay(raw.replace('.', ',').replace(/[^\d,]/g, ''));
    onChange(normalized);
  };

  const handleBlur = () => {
    setIsFocused(false);
    if (!value) return;
    const num = parseSEK(value);
    if (num <= 0) {
      onChange('');
    }
  };

  const handleFocus = () => {
    if (value) {
      const num = parseSEK(value);
      if (num > 0) {
        setLocalDisplay(num.toString().replace('.', ','));
      } else {
        setLocalDisplay('');
      }
    } else {
      setLocalDisplay('');
    }
    setIsFocused(true);
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
      <div className="flex gap-3 overflow-x-auto pb-2">
        {files.map((f, i) => (
          <div key={i} className="relative group shrink-0">
            <div className="w-36 h-48 rounded-md border border-border overflow-hidden bg-muted flex items-center justify-center">
              {f.preview && f.file.type.startsWith('image/') ? (
                <img src={f.preview} alt={f.file.name} className="w-full h-full object-cover" />
              ) : f.preview && f.file.type === 'application/pdf' ? (
                <iframe
                  src={`${f.preview}#toolbar=0&navpanes=0`}
                  title={f.file.name}
                  className="w-full h-full border-0"
                />
              ) : (
                <div className="text-center p-2">
                  <svg className="w-8 h-8 mx-auto text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
                  </svg>
                  <span className="text-xs text-muted-foreground mt-1 block">{f.file.name.split('.').pop()?.toUpperCase()}</span>
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
            <p className="text-[10px] text-muted-foreground truncate w-36 mt-0.5">{f.file.name}</p>
          </div>
        ))}
        <button
          type="button"
          onClick={() => inputRef.current?.click()}
          className="w-36 h-48 shrink-0 rounded-md border-2 border-dashed border-border hover:border-ring flex flex-col items-center justify-center text-muted-foreground hover:text-foreground transition-colors"
        >
          <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 4v16m8-8H4" />
          </svg>
          <span className="text-xs mt-1">Lägg till</span>
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
  const underlagInputRef = useRef<HTMLInputElement>(null);

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
            const res = await fetch(`/api/vouchers/${result.id}/attachments`, {
              method: 'POST',
              headers: token ? { Authorization: `Bearer ${token}` } : {},
              body: formData,
            });
            if (!res.ok) {
              const body = await res.json().catch(() => ({ error: res.statusText }));
              throw new Error(body.error || res.statusText);
            }
          } catch (err) {
            const msg = err instanceof Error ? err.message : 'Kunde inte ladda upp bilaga';
            toast.error(`${u.file.name}: ${msg}`);
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
    <form
      onSubmit={(e) => {
        e.preventDefault();
        setError('');
        mutation.mutate();
      }}
    >
      <div className="flex flex-col lg:flex-row gap-6 lg:h-[calc(100vh-10rem)]">
        {/* Left: Form */}
        <Card className="flex-1 min-w-0 lg:w-1/2 lg:flex lg:flex-col lg:overflow-hidden">
          <CardHeader>
            <CardTitle className="text-base">Ny verifikation</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4 lg:overflow-y-auto lg:flex-1">
            <div className="grid gap-4 sm:grid-cols-[150px_1fr]">
              <div className="space-y-2">
                <Label htmlFor="date">Datum</Label>
                <Input
                  id="date"
                  type="date"
                  value={date}
                  onChange={(e) => setDate(e.target.value)}
                  className="max-w-[150px]"
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

            {/* Mobile only: show underlag inline */}
            <div className="lg:hidden">
              <Separator />
              <UnderlagUpload files={underlag} onChange={setUnderlag} />
            </div>

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
          </CardContent>
        </Card>

        {/* Right: Underlag panel (desktop only) */}
        <div className="hidden lg:flex lg:w-1/2 lg:min-w-0">
          <Card className="flex flex-col w-full overflow-hidden">
            <CardHeader className="pb-3">
              <CardTitle className="text-base">Underlag</CardTitle>
            </CardHeader>
            <CardContent className="flex-1 overflow-y-auto space-y-3">
              {underlag.length === 0 ? (
                <button
                  type="button"
                  onClick={() => underlagInputRef.current?.click()}
                  className="w-full h-48 rounded-md border-2 border-dashed border-border hover:border-ring flex flex-col items-center justify-center text-muted-foreground hover:text-foreground transition-colors"
                >
                  <svg className="w-10 h-10" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 4v16m8-8H4" />
                  </svg>
                  <span className="text-sm mt-2">Lägg till underlag</span>
                </button>
              ) : (
                <>
                  {underlag.map((f, i) => (
                    <div key={i} className="relative group">
                      <div className="w-full rounded-md border border-border overflow-hidden bg-muted">
                        {f.preview && f.file.type.startsWith('image/') ? (
                          <img src={f.preview} alt={f.file.name} className="w-full object-contain max-h-[500px]" />
                        ) : f.preview && f.file.type === 'application/pdf' ? (
                          <iframe
                            src={`${f.preview}#toolbar=0&navpanes=0`}
                            title={f.file.name}
                            className="w-full h-[500px] border-0"
                          />
                        ) : (
                          <div className="text-center p-6">
                            <svg className="w-10 h-10 mx-auto text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
                            </svg>
                            <span className="text-sm text-muted-foreground mt-2 block">{f.file.name}</span>
                          </div>
                        )}
                      </div>
                      <div className="flex items-center justify-between mt-1">
                        <p className="text-xs text-muted-foreground truncate">{f.file.name}</p>
                        <button
                          type="button"
                          onClick={() => {
                            const removed = underlag[i];
                            if (removed.preview) URL.revokeObjectURL(removed.preview);
                            setUnderlag(underlag.filter((_, j) => j !== i));
                          }}
                          className="text-xs text-muted-foreground hover:text-destructive transition-colors shrink-0 ml-2"
                        >
                          Ta bort
                        </button>
                      </div>
                    </div>
                  ))}
                  <button
                    type="button"
                    onClick={() => underlagInputRef.current?.click()}
                    className="w-full h-16 rounded-md border-2 border-dashed border-border hover:border-ring flex items-center justify-center text-muted-foreground hover:text-foreground transition-colors"
                  >
                    <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 4v16m8-8H4" />
                    </svg>
                    <span className="text-sm">Lägg till fler</span>
                  </button>
                </>
              )}
              <input
                ref={underlagInputRef}
                type="file"
                multiple
                accept="image/*,application/pdf,.doc,.docx,.xls,.xlsx"
                className="hidden"
                onChange={(e) => {
                  if (e.target.files) {
                    const newFiles: UnderlagFile[] = [];
                    for (const file of Array.from(e.target.files)) {
                      let preview: string | null = null;
                      if (file.type.startsWith('image/') || file.type === 'application/pdf') {
                        preview = URL.createObjectURL(file);
                      }
                      newFiles.push({ file, preview });
                    }
                    setUnderlag([...underlag, ...newFiles]);
                  }
                  e.target.value = '';
                }}
              />
            </CardContent>
          </Card>
        </div>
      </div>
    </form>
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
