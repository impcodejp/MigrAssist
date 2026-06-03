export type Encoding = 'sjis' | 'utf-8' | 'utf-8-bom';

export interface ColumnMapping {
  mjsHeader: string;
  otherHeader: string;
  isCompare: boolean;
  tolerance: number;
}

export interface ComparisonConfig {
  mjsPath: string;
  mjsEncoding: Encoding;
  otherPath: string;
  otherEncoding: Encoding;
  outputDir: string;
  columns: ColumnMapping[];
  timestamp: string;
}

export interface ConfigData {
  columns: ColumnMapping[];
}

export interface ProgressPayload {
  message: string;
  step: number;
  total: number;
}
