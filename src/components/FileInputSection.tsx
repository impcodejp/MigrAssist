import type { Encoding } from '../types';

interface Props {
  label: string;
  path: string;
  encoding: Encoding;
  onPathChange: (path: string) => void;
  onEncodingChange: (encoding: Encoding) => void;
  onBrowse: () => void;
}

const ENCODINGS: { value: Encoding; label: string }[] = [
  { value: 'sjis', label: 'SJIS' },
  { value: 'utf-8', label: 'UTF-8' },
  { value: 'utf-8-bom', label: 'UTF-8 BOM' },
];

export function FileInputSection({
  label,
  path,
  encoding,
  onPathChange,
  onEncodingChange,
  onBrowse,
}: Props) {
  return (
    <div className="file-input-row">
      <span className="file-input-label">{label}</span>
      <div className="file-input-controls">
        <input
          type="text"
          className="file-path-input"
          value={path}
          onChange={(e) => onPathChange(e.target.value)}
          placeholder="ファイルパスを入力または参照ボタンで選択"
        />
        <select
          className="encoding-select"
          value={encoding}
          onChange={(e) => onEncodingChange(e.target.value as Encoding)}
        >
          {ENCODINGS.map((enc) => (
            <option key={enc.value} value={enc.value}>
              {enc.label}
            </option>
          ))}
        </select>
        <button className="btn btn-secondary" onClick={onBrowse}>
          参照
        </button>
      </div>
    </div>
  );
}
