import type { ColumnMapping } from '../types';

interface Props {
  columns: ColumnMapping[];
  otherHeaders: string[];
  onUpdate: (idx: number, patch: Partial<ColumnMapping>) => void;
}

export function MappingTable({ columns, otherHeaders, onUpdate }: Props) {
  if (columns.length === 0) {
    return (
      <div className="mapping-table-empty">
        CSVファイルを取り込むと突合設定テーブルが表示されます
      </div>
    );
  }

  return (
    <div className="mapping-table-scroll">
      <table className="mapping-table">
        <tbody>
          {/* Row 1: 比較対象 */}
          <tr>
            <th className="mapping-row-header">比較対象</th>
            {columns.map((col, idx) => (
              <td key={idx} className="mapping-cell">
                {idx === 0 ? (
                  <span className="cell-fixed">—</span>
                ) : (
                  <input
                    type="checkbox"
                    checked={col.isCompare}
                    onChange={(e) =>
                      onUpdate(idx, { isCompare: e.target.checked })
                    }
                  />
                )}
              </td>
            ))}
          </tr>

          {/* Row 2: MJSシステム */}
          <tr>
            <th className="mapping-row-header">MJSシステム</th>
            {columns.map((col, idx) => (
              <td key={idx} className="mapping-cell cell-mjs">
                <span title={col.mjsHeader}>{col.mjsHeader}</span>
              </td>
            ))}
          </tr>

          {/* Row 3: 他社システム */}
          <tr>
            <th className="mapping-row-header">他社システム</th>
            {columns.map((col, idx) => (
              <td key={idx} className="mapping-cell">
                <select
                  value={col.otherHeader}
                  onChange={(e) =>
                    onUpdate(idx, { otherHeader: e.target.value })
                  }
                  className={
                    idx === 0 && !col.otherHeader ? 'select-required' : ''
                  }
                >
                  <option value="">（未選択）</option>
                  {otherHeaders.map((h) => (
                    <option key={h} value={h}>
                      {h}
                    </option>
                  ))}
                </select>
              </td>
            ))}
          </tr>

          {/* Row 4: 許容誤差 */}
          <tr>
            <th className="mapping-row-header">許容誤差±</th>
            {columns.map((col, idx) => (
              <td key={idx} className="mapping-cell">
                {idx === 0 ? (
                  <span className="cell-fixed" />
                ) : (
                  <input
                    type="number"
                    min={0}
                    value={col.tolerance}
                    onChange={(e) =>
                      onUpdate(idx, {
                        tolerance: Math.max(0, parseInt(e.target.value) || 0),
                      })
                    }
                    className="tolerance-input"
                  />
                )}
              </td>
            ))}
          </tr>
        </tbody>
      </table>
    </div>
  );
}
