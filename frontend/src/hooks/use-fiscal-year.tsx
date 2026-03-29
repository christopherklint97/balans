import { createContext, useContext, useState, useEffect, useCallback, type ReactNode } from 'react';
import { useQuery } from '@tanstack/react-query';
import { companiesApi, fiscalYearsApi } from '@/api/queries';
import { useAuth } from '@/auth/context';
import type { Company, FiscalYear } from '@/api/types';

const COMPANY_KEY = 'balans_company_id';
const FY_KEY = 'balans_fy_id';

interface FiscalYearContextValue {
  companies: Company[];
  fiscalYears: FiscalYear[];
  activeCompanyId: string | undefined;
  activeFyId: string | undefined;
  activeFy: FiscalYear | undefined;
  setCompanyId: (id: string) => void;
  setFyId: (id: string) => void;
}

const FiscalYearContext = createContext<FiscalYearContextValue | undefined>(undefined);

export function FiscalYearProvider({ children }: { children: ReactNode }) {
  const { user } = useAuth();

  const [storedCompanyId, setStoredCompanyId] = useState<string | null>(
    () => localStorage.getItem(COMPANY_KEY),
  );
  const [storedFyId, setStoredFyId] = useState<string | null>(
    () => localStorage.getItem(FY_KEY),
  );

  const { data: companies } = useQuery({
    queryKey: ['companies'],
    queryFn: companiesApi.list,
    enabled: !!user,
  });

  // Resolve active company: stored value if still valid, otherwise first
  const activeCompanyId =
    (storedCompanyId && companies?.some((c) => c.id === storedCompanyId)
      ? storedCompanyId
      : companies?.[0]?.id) ?? undefined;

  const { data: fiscalYears } = useQuery({
    queryKey: ['fiscal-years', activeCompanyId],
    queryFn: () => fiscalYearsApi.list(activeCompanyId!),
    enabled: !!activeCompanyId,
  });

  // Resolve active fiscal year: stored value if still valid, otherwise first
  const activeFyId =
    (storedFyId && fiscalYears?.some((fy) => fy.id === storedFyId)
      ? storedFyId
      : fiscalYears?.[0]?.id) ?? undefined;

  const activeFy = fiscalYears?.find((fy) => fy.id === activeFyId);

  const setCompanyId = useCallback((id: string) => {
    localStorage.setItem(COMPANY_KEY, id);
    setStoredCompanyId(id);
    // Clear FY when switching company
    localStorage.removeItem(FY_KEY);
    setStoredFyId(null);
  }, []);

  const setFyId = useCallback((id: string) => {
    localStorage.setItem(FY_KEY, id);
    setStoredFyId(id);
  }, []);

  // Sync localStorage when resolved values change (e.g. first load defaults)
  useEffect(() => {
    if (activeCompanyId && activeCompanyId !== storedCompanyId) {
      localStorage.setItem(COMPANY_KEY, activeCompanyId);
    }
  }, [activeCompanyId, storedCompanyId]);

  useEffect(() => {
    if (activeFyId && activeFyId !== storedFyId) {
      localStorage.setItem(FY_KEY, activeFyId);
    }
  }, [activeFyId, storedFyId]);

  return (
    <FiscalYearContext.Provider
      value={{
        companies: companies ?? [],
        fiscalYears: fiscalYears ?? [],
        activeCompanyId,
        activeFyId,
        activeFy,
        setCompanyId,
        setFyId,
      }}
    >
      {children}
    </FiscalYearContext.Provider>
  );
}

export function useFiscalYear() {
  const ctx = useContext(FiscalYearContext);
  if (!ctx) throw new Error('useFiscalYear must be used within FiscalYearProvider');
  return ctx;
}
