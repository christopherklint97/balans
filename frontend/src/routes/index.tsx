import { createFileRoute, Link } from '@tanstack/react-router';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { companiesApi, fiscalYearsApi } from '@/api/queries';
import { useFiscalYear } from '@/hooks/use-fiscal-year';
import type { Company } from '@/api/types';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';

export const Route = createFileRoute('/')({
  component: Dashboard,
});

function Dashboard() {
  const queryClient = useQueryClient();
  const [showForm, setShowForm] = useState(false);

  const { data: companies, isLoading } = useQuery({
    queryKey: ['companies'],
    queryFn: companiesApi.list,
  });

  if (isLoading) {
    return <p className="text-muted-foreground">Laddar...</p>;
  }

  if (!companies?.length) {
    return (
      <div className="space-y-6">
        <h1 className="text-2xl font-semibold">Välkommen till Balans</h1>
        <p className="text-muted-foreground">
          Skapa ett företag för att komma igång.
        </p>
        <CreateCompanyForm onSuccess={() => queryClient.invalidateQueries({ queryKey: ['companies'] })} />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Kontrollpanel</h1>
        <Button variant="outline" size="sm" onClick={() => setShowForm(!showForm)}>
          {showForm ? 'Avbryt' : 'Nytt företag'}
        </Button>
      </div>

      {showForm && (
        <CreateCompanyForm
          onSuccess={() => {
            queryClient.invalidateQueries({ queryKey: ['companies'] });
            setShowForm(false);
          }}
        />
      )}

      <div className="grid gap-4 md:grid-cols-2">
        {companies.map((company) => (
          <CompanyCard key={company.id} company={company} />
        ))}
      </div>
    </div>
  );
}

function CompanyCard({ company }: { company: Company }) {
  const queryClient = useQueryClient();
  const { setCompanyId, setFyId } = useFiscalYear();
  const [showFyForm, setShowFyForm] = useState(false);
  const [editing, setEditing] = useState(false);
  const [editName, setEditName] = useState(company.name);
  const [editOrgNumber, setEditOrgNumber] = useState(company.org_number);
  const [editCompanyForm, setEditCompanyForm] = useState(company.company_form);
  const [editError, setEditError] = useState('');

  const updateMutation = useMutation({
    mutationFn: () =>
      companiesApi.update(company.id, { name: editName, org_number: editOrgNumber, company_form: editCompanyForm }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['companies'] });
      setEditing(false);
      setEditError('');
    },
    onError: (err: Error) => setEditError(err.message),
  });

  const { data: fiscalYears } = useQuery({
    queryKey: ['fiscal-years', company.id],
    queryFn: () => fiscalYearsApi.list(company.id),
  });

  const activeFy = fiscalYears?.find((fy) => !fy.is_closed);

  return (
    <Card>
      <CardHeader className="pb-2">
        {editing ? (
          <div className="space-y-2">
            <div className="space-y-1">
              <Label htmlFor={`edit-name-${company.id}`}>Företagsnamn</Label>
              <Input
                id={`edit-name-${company.id}`}
                value={editName}
                onChange={(e) => setEditName(e.target.value)}
              />
            </div>
            <div className="space-y-1">
              <Label htmlFor={`edit-org-${company.id}`}>Organisationsnummer</Label>
              <Input
                id={`edit-org-${company.id}`}
                value={editOrgNumber}
                onChange={(e) => setEditOrgNumber(e.target.value)}
              />
            </div>
            <div className="space-y-1">
              <Label htmlFor={`edit-form-${company.id}`}>Bolagsform</Label>
              <select
                id={`edit-form-${company.id}`}
                className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                value={editCompanyForm}
                onChange={(e) => setEditCompanyForm(e.target.value)}
              >
                <option value="AB">Aktiebolag (AB)</option>
                <option value="EF">Enskild firma (EF)</option>
                <option value="HB">Handelsbolag (HB)</option>
                <option value="KB">Kommanditbolag (KB)</option>
                <option value="EK">Ekonomisk förening (EK)</option>
              </select>
            </div>
            {editError && <p className="text-sm text-destructive">{editError}</p>}
            <div className="flex gap-2">
              <Button size="sm" onClick={() => updateMutation.mutate()} disabled={updateMutation.isPending}>
                {updateMutation.isPending ? 'Sparar...' : 'Spara'}
              </Button>
              <Button size="sm" variant="ghost" onClick={() => { setEditing(false); setEditError(''); setEditName(company.name); setEditOrgNumber(company.org_number); setEditCompanyForm(company.company_form); }}>
                Avbryt
              </Button>
            </div>
          </div>
        ) : (
          <>
            <div className="flex items-center justify-between">
              <CardTitle className="text-base">{company.name}</CardTitle>
              <div className="flex items-center gap-2">
                <Badge variant="secondary">{company.company_form}</Badge>
                <Button variant="ghost" size="sm" className="h-6 px-2 text-xs" onClick={() => setEditing(true)}>
                  Redigera
                </Button>
              </div>
            </div>
            <p className="text-sm text-muted-foreground">{company.org_number}</p>
          </>
        )}
      </CardHeader>
      <CardContent className="space-y-2">
        {activeFy ? (
          <div className="text-sm">
            <span className="text-muted-foreground">Räkenskapsår: </span>
            {activeFy.start_date} — {activeFy.end_date}
          </div>
        ) : (
          <div className="space-y-2">
            <p className="text-sm text-muted-foreground">Inget räkenskapsår skapat</p>
            {!showFyForm && (
              <Button variant="outline" size="sm" onClick={() => setShowFyForm(true)}>
                Skapa räkenskapsår
              </Button>
            )}
          </div>
        )}

        {showFyForm && !activeFy && (
          <CreateFiscalYearForm
            companyId={company.id}
            onSuccess={() => {
              queryClient.invalidateQueries({ queryKey: ['fiscal-years', company.id] });
              setShowFyForm(false);
            }}
            onCancel={() => setShowFyForm(false)}
          />
        )}

        <div className="flex flex-wrap gap-2 pt-2">
          <Link to="/accounts" onClick={() => setCompanyId(company.id)}>
            <Button variant="outline" size="sm">Kontoplan</Button>
          </Link>
          {activeFy && (
            <Link to="/vouchers" onClick={() => { setCompanyId(company.id); setFyId(activeFy.id); }}>
              <Button variant="outline" size="sm">Verifikationer</Button>
            </Link>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

function CreateFiscalYearForm({
  companyId,
  onSuccess,
  onCancel,
}: {
  companyId: string;
  onSuccess: () => void;
  onCancel: () => void;
}) {
  const year = new Date().getFullYear();
  const [startDate, setStartDate] = useState(`${year}-01-01`);
  const [endDate, setEndDate] = useState(`${year}-12-31`);
  const [error, setError] = useState('');

  const mutation = useMutation({
    mutationFn: () =>
      fiscalYearsApi.create(companyId, { start_date: startDate, end_date: endDate }),
    onSuccess,
    onError: (err: Error) => setError(err.message),
  });

  return (
    <div className="space-y-3 rounded-md border border-border p-3">
      <p className="text-sm font-medium">Nytt räkenskapsår</p>
      <div className="grid gap-3 sm:grid-cols-2">
        <div className="space-y-1">
          <Label htmlFor="fy-start">Startdatum</Label>
          <Input
            id="fy-start"
            type="date"
            value={startDate}
            onChange={(e) => setStartDate(e.target.value)}
            required
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor="fy-end">Slutdatum</Label>
          <Input
            id="fy-end"
            type="date"
            value={endDate}
            onChange={(e) => setEndDate(e.target.value)}
            required
          />
        </div>
      </div>
      {error && <p className="text-sm text-destructive">{error}</p>}
      <div className="flex gap-2">
        <Button size="sm" onClick={() => mutation.mutate()} disabled={mutation.isPending}>
          {mutation.isPending ? 'Skapar...' : 'Skapa'}
        </Button>
        <Button size="sm" variant="ghost" onClick={onCancel}>
          Avbryt
        </Button>
      </div>
    </div>
  );
}

function CreateCompanyForm({ onSuccess }: { onSuccess: () => void }) {
  const [name, setName] = useState('');
  const [orgNumber, setOrgNumber] = useState('');
  const [companyForm, setCompanyForm] = useState('AB');
  const [error, setError] = useState('');

  const mutation = useMutation({
    mutationFn: () =>
      companiesApi.create({
        name,
        org_number: orgNumber,
        company_form: companyForm,
      }),
    onSuccess: async (company) => {
      // Auto-create a fiscal year for the current calendar year
      const year = new Date().getFullYear();
      await fiscalYearsApi.create(company.id, {
        start_date: `${year}-01-01`,
        end_date: `${year}-12-31`,
      });
      onSuccess();
    },
    onError: (err: Error) => setError(err.message),
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Skapa företag</CardTitle>
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
              <Label htmlFor="name">Företagsnamn</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Mitt Företag AB"
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="org">Organisationsnummer</Label>
              <Input
                id="org"
                value={orgNumber}
                onChange={(e) => setOrgNumber(e.target.value)}
                placeholder="5561234567"
                required
              />
            </div>
          </div>
          <div className="space-y-2">
            <Label htmlFor="form">Företagsform</Label>
            <select
              id="form"
              value={companyForm}
              onChange={(e) => setCompanyForm(e.target.value)}
              className="flex h-9 w-full sm:max-w-[200px] rounded-md border border-input bg-transparent px-3 py-1 text-sm"
            >
              <option value="AB">Aktiebolag (AB)</option>
              <option value="EF">Enskild firma (EF)</option>
              <option value="HB">Handelsbolag (HB)</option>
              <option value="KB">Kommanditbolag (KB)</option>
              <option value="EK">Ekonomisk förening (EK)</option>
            </select>
          </div>
          {error && <p className="text-sm text-destructive">{error}</p>}
          <Button type="submit" disabled={mutation.isPending}>
            {mutation.isPending ? 'Skapar...' : 'Skapa företag'}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
