// Auth
export interface AuthResponse {
  token: string | null;
  user: UserInfo | null;
  status: string;
  message: string | null;
}

export interface UserInfo {
  id: string;
  email: string;
  name: string;
  role: string;
}

// Admin
export interface AdminUser {
  id: string;
  email: string;
  name: string;
  role: string;
  status: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CompanyUser {
  user_id: string;
  email: string;
  name: string;
  company_role: string;
  system_role: string;
}

export interface AppConfig {
  mode: string;
  fixed_company_id: string | null;
}

export interface Company {
  id: string;
  name: string;
  org_number: string;
  company_form: string;
  fiscal_year_start_month: number;
  address: string | null;
  postal_code: string | null;
  city: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateCompany {
  name: string;
  org_number: string;
  company_form: string;
  fiscal_year_start_month?: number;
  address?: string;
  postal_code?: string;
  city?: string;
}

export interface FiscalYear {
  id: string;
  company_id: string;
  start_date: string;
  end_date: string;
  is_closed: boolean;
  closed_at: string | null;
  created_at: string;
}

export interface CreateFiscalYear {
  start_date: string;
  end_date: string;
}

export interface Account {
  id: string;
  company_id: string;
  number: number;
  name: string;
  account_type: string;
  is_active: boolean;
  created_at: string;
}

export interface Voucher {
  id: string;
  company_id: string;
  fiscal_year_id: string;
  voucher_number: number;
  date: string;
  description: string;
  is_closing_entry: boolean;
  created_at: string;
}

export interface VoucherLine {
  id: string;
  voucher_id: string;
  account_number: number;
  debit: string;
  credit: string;
  description: string | null;
}

export interface VoucherWithLines extends Voucher {
  lines: VoucherLine[];
}

export interface CreateVoucher {
  date: string;
  description: string;
  lines: CreateVoucherLine[];
}

export interface CreateVoucherLine {
  account_number: number;
  debit: string;
  credit: string;
  description?: string;
}

export interface AttachmentMeta {
  id: string;
  voucher_id: string;
  filename: string;
  content_type: string;
  size_bytes: number;
  created_at: string;
}

export interface TrialBalanceRow {
  account_number: number;
  account_name: string;
  debit_total: string;
  credit_total: string;
  balance: string;
}

export interface SieImportPreview {
  sie_type: string;
  company_name: string | null;
  org_number: string | null;
  fiscal_years: SieFiscalYearPreview[];
  account_count: number;
  voucher_count: number;
  transaction_count: number;
  opening_balances: number;
  closing_balances: number;
}

export interface SieFiscalYearPreview {
  index: number;
  start_date: string;
  end_date: string;
}

export interface SieImportResult {
  accounts_imported: number;
  vouchers_imported: number;
  fiscal_year_id: string | null;
}

// Closing (Årsbokslut)

export interface ValidationResult {
  passed: boolean;
  errors: ValidationIssue[];
  warnings: ValidationIssue[];
  summary: ClosingSummary;
}

export interface ValidationIssue {
  code: string;
  message: string;
  severity: 'error' | 'warning';
}

export interface ClosingSummary {
  total_revenue: string;
  total_expenses: string;
  operating_result: string;
  financial_income: string;
  financial_expenses: string;
  result_before_tax: string;
  estimated_tax: string;
  net_result: string;
  total_assets: string;
  total_equity_and_liabilities: string;
  balance_difference: string;
}

export interface ClosingParams {
  tax_amount?: string;
  carry_forward?: boolean;
}

export interface ClosingResult {
  closing_vouchers: ClosingVoucherInfo[];
  fiscal_year_closed: boolean;
  next_fiscal_year_id: string | null;
}

export interface ClosingVoucherInfo {
  voucher_id: string;
  voucher_number: number;
  description: string;
  total_amount: string;
}

export interface ClosingStatus {
  is_closed: boolean;
  closed_at: string | null;
  closing_voucher_count: number;
  total_voucher_count: number;
}

// Annual Report (Årsredovisning)

export interface IncomeStatement {
  current: IncomeStatementData;
  previous: IncomeStatementData | null;
}

export interface IncomeStatementData {
  fiscal_year: string;
  net_revenue: string;
  inventory_change: string;
  capitalized_work: string;
  other_operating_income: string;
  raw_materials: string;
  goods_for_resale: string;
  other_external_costs: string;
  personnel_costs: string;
  depreciation: string;
  other_operating_costs: string;
  operating_result: string;
  financial_income: string;
  financial_costs: string;
  result_after_financial: string;
  appropriations: string;
  result_before_tax: string;
  tax: string;
  net_result: string;
}

export interface BalanceSheet {
  current: BalanceSheetData;
  previous: BalanceSheetData | null;
}

export interface BalanceSheetData {
  fiscal_year: string;
  assets: {
    intangible_assets: string;
    tangible_assets: string;
    financial_fixed_assets: string;
    total_fixed_assets: string;
    inventory: string;
    current_receivables: string;
    short_term_investments: string;
    cash_and_bank: string;
    total_current_assets: string;
  };
  equity_and_liabilities: {
    restricted_equity: string;
    unrestricted_equity: string;
    total_equity: string;
    untaxed_reserves: string;
    provisions: string;
    long_term_liabilities: string;
    current_liabilities: string;
    total_liabilities: string;
  };
  total_assets: string;
  total_equity_and_liabilities: string;
}

// K2 Eligibility
export interface EligibilityResult {
  is_eligible: boolean;
  reason: string | null;
  checks: {
    average_employees: number;
    balance_sheet_total: string;
    net_revenue: string;
    employees_exceeded: boolean;
    balance_exceeded: boolean;
    revenue_exceeded: boolean;
    thresholds_exceeded: number;
    company_form_allowed: boolean;
  };
  thresholds: {
    max_employees: number;
    max_balance_sheet: string;
    max_net_revenue: string;
  };
}

export interface MultiYearOverview {
  years: {
    fiscal_year: string;
    net_revenue: string;
    operating_result: string;
    result_after_financial: string;
    total_assets: string;
    equity_ratio: string;
  }[];
}

// Bolagsverket Filing
export interface IxbrlPreview {
  company_name: string;
  org_number: string;
  fiscal_year_start: string;
  fiscal_year_end: string;
  is_closed: boolean;
  document_size_bytes: number;
  checksum_sha256: string;
}

export interface FilingResult {
  step: string;
  success: boolean;
  message: string;
  verification_errors: { kod: string | null; meddelande: string }[];
  verification_warnings: { kod: string | null; meddelande: string }[];
  submission_reference: string | null;
}

// INK2 / Tax
export interface Ink2Data {
  company_name: string;
  org_number: string;
  fiscal_year_start: string;
  fiscal_year_end: string;
  fields: Ink2Field[];
  sections: Ink2Section[];
}

export interface Ink2Field {
  sru_code: string;
  label: string;
  amount: string;
  accounts: number[];
}

// Fixed Assets
export interface FixedAsset {
  id: string;
  company_id: string;
  name: string;
  description: string | null;
  asset_type: string;
  acquisition_date: string;
  acquisition_cost: string;
  useful_life_months: number;
  residual_value: string;
  depreciation_start_date: string;
  asset_account: number;
  depreciation_account: number;
  expense_account: number;
  is_disposed: boolean;
  disposal_date: string | null;
  disposal_amount: string;
  created_at: string;
  updated_at: string;
}

export interface CreateFixedAsset {
  name: string;
  description?: string;
  asset_type: string;
  acquisition_date: string;
  acquisition_cost: string;
  useful_life_months: number;
  residual_value?: string;
  depreciation_start_date?: string;
}

export interface DepreciationSummary {
  fiscal_year_id: string;
  assets: AssetDepreciation[];
  total_depreciation: string;
}

export interface AssetDepreciation {
  asset_id: string;
  asset_name: string;
  asset_type: string;
  acquisition_cost: string;
  depreciation_this_year: string;
  accumulated_depreciation: string;
  book_value: string;
  expense_account: number;
  depreciation_account: number;
}

export interface Ink2Section {
  title: string;
  fields: Ink2Field[];
}

export interface AuditEntry {
  id: string;
  entity_type: string;
  entity_id: string;
  action: string;
  details: string | null;
  created_at: string;
}

export interface AnnualReport {
  company: {
    name: string;
    org_number: string;
    company_form: string;
    address: string | null;
    postal_code: string | null;
    city: string | null;
  };
  fiscal_year: {
    start_date: string;
    end_date: string;
    is_closed: boolean;
  };
  directors_report: {
    business_description: string;
    important_events: string;
    future_outlook: string;
    profit_allocation: {
      result_for_year: string;
      retained_earnings: string;
      total_available: string;
      carry_forward: string;
      dividend: string;
    } | null;
  };
  income_statement: IncomeStatement;
  balance_sheet: BalanceSheet;
  notes: {
    items: { number: number; title: string; content: string }[];
  };
}
