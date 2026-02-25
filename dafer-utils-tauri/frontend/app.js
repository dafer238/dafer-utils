// ─── Tauri IPC ─────────────────────────────────────────────────────────────
function invoke(cmd, args) {
  return window.__TAURI__.core.invoke(cmd, args);
}

// ─── State ─────────────────────────────────────────────────────────────────
let columnNames = [];
let columnDtypes = [];
let previewRows = [];
let previewHeaders = [];
let sortColumn = null;
let sortDescending = false;
let selectedCell = null;
let selectedRow = null;
let selectedCol = null;
let plotYColumns = [];
let dataLoaded = false;

// ─── Tab Switching ──────────────────────────────────────────────────────────
document.querySelectorAll('.tab-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
    document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
    btn.classList.add('active');
    document.getElementById('tab-' + btn.dataset.tab).classList.add('active');
    // Resize plot when switching to visualize
    if (btn.dataset.tab === 'visualize') {
      setTimeout(() => { Plotly.Plots.resize('plot-container'); }, 50);
    }
  });
});

// ─── Status Bar ─────────────────────────────────────────────────────────────
function setStatus(msg) {
  document.getElementById('status-text').textContent = msg;
}

function updatePipelineStatus() {
  invoke('get_operations').then(ops => {
    const el = document.getElementById('pipeline-status');
    el.textContent = ops.length > 0 ? `Pipeline: ${ops.length} ops` : '';
    document.getElementById('pipeline-count').textContent =
      `${ops.length} operations in pipeline`;
  });
}

// ─── File Operations ────────────────────────────────────────────────────────
async function browseFile() {
  try {
    const result = await invoke('pick_data_file');
    if (result) {
      await openFilePath(result);
    }
  } catch (e) {
    setStatus('Error: ' + e);
  }
}

async function openFilePath(path) {
  try {
    setStatus('Loading...');
    const msg = await invoke('open_file', { path });
    setStatus(msg);
    plotYColumns = [];
    sortColumn = null;
    sortDescending = false;
    selectedCell = null;
    selectedRow = null;
    selectedCol = null;
    await loadPreview();
  } catch (e) {
    setStatus('Error: ' + e);
  }
}

async function menuOpen() {
  await browseFile();
}

async function menuSaveState() {
  try {
    const path = await invoke('pick_save_path', { ext: 'dfr' });
    if (path) {
      const msg = await invoke('save_state', { path });
      setStatus(msg);
    }
  } catch (e) {
    setStatus('Error: ' + e);
  }
}

async function menuLoadState() {
  try {
    const path = await invoke('pick_data_file');
    if (path) {
      const msg = await invoke('load_state', { path });
      setStatus(msg);
      plotYColumns = [];
      sortColumn = null;
      sortDescending = false;
      await loadPreview();
    }
  } catch (e) {
    setStatus('Error: ' + e);
  }
}

// ─── Preview Loading ────────────────────────────────────────────────────────
async function loadPreview() {
  try {
    const data = await invoke('get_preview');
    previewHeaders = data.headers;
    previewRows = data.rows;
    columnNames = data.headers;
    columnDtypes = data.dtypes;
    dataLoaded = true;

    // Update file path display
    try {
      const meta = await invoke('get_file_metadata');
      document.getElementById('file-path-display').textContent =
        `${meta.path} (${meta.source_type})`;
      document.getElementById('file-metadata').innerHTML =
        `File: ${meta.path} | Size: ${meta.size} | ${meta.source_type} | Pipeline: ${(await invoke('get_operations')).length} ops`;
      document.getElementById('file-metadata').classList.remove('hidden');
    } catch(_) {}

    // Show preview info
    document.getElementById('preview-info').textContent =
      `Showing ${data.preview_rows} of ${data.total_rows} rows × ${data.headers.length} columns`;
    document.getElementById('preview-info').classList.remove('hidden');

    // Hide welcome
    document.getElementById('welcome-msg').style.display = 'none';

    // Build preview table
    renderPreviewTable(data.headers, data.rows);
    document.getElementById('preview-table-container').classList.remove('hidden');

    // Build stats table
    renderStatsTable(data.stats);
    document.getElementById('stats-container').classList.remove('hidden');

    // Update modify tab
    updateModifyTab();

    // Update visualize tab
    updateVisualizeControls();

    // Status
    setStatus(`Showing ${data.preview_rows} of ${data.total_rows} rows × ${data.headers.length} columns`);
    updatePipelineStatus();
  } catch (e) {
    setStatus('Preview error: ' + e);
  }
}

// ─── Preview Table Rendering ────────────────────────────────────────────────
function renderPreviewTable(headers, rows) {
  const wrapper = document.getElementById('preview-table-wrapper');
  // Apply client-side sort
  let displayRows = rows.slice();
  if (sortColumn !== null) {
    const colIdx = headers.indexOf(sortColumn);
    if (colIdx >= 0) {
      displayRows.sort((a, b) => {
        const av = a[colIdx], bv = b[colIdx];
        const an = parseFloat(av), bn = parseFloat(bv);
        let cmp;
        if (!isNaN(an) && !isNaN(bn)) {
          cmp = an - bn;
        } else {
          cmp = av.localeCompare(bv);
        }
        return sortDescending ? -cmp : cmp;
      });
    }
  }

  let html = '<table><thead><tr>';
  headers.forEach((h, i) => {
    let cls = '';
    if (h === sortColumn) {
      cls = sortDescending ? 'sorted-desc' : 'sorted-asc';
    }
    html += `<th class="${cls}" onclick="onHeaderClick(event, ${i}, '${escHtml(h)}')" oncontextmenu="onHeaderRightClick(event, ${i})">${escHtml(h)}</th>`;
  });
  html += '</tr></thead><tbody>';

  displayRows.forEach((row, ri) => {
    const selClass = selectedRow === ri ? ' class="selected"' : '';
    html += `<tr${selClass}>`;
    row.forEach((cell, ci) => {
      const isSel = (selectedCell && selectedCell[0] === ri && selectedCell[1] === ci)
                 || selectedRow === ri || selectedCol === ci;
      const style = isSel ? ' style="background:rgba(131,155,255,0.2)"' : '';
      html += `<td${style} onclick="onCellClick(${ri},${ci})">${escHtml(cell)}</td>`;
    });
    html += '</tr>';
  });
  html += '</tbody></table>';
  wrapper.innerHTML = html;
}

function onHeaderClick(event, colIdx, colName) {
  if (sortColumn === colName) {
    sortDescending = !sortDescending;
  } else {
    sortColumn = colName;
    sortDescending = false;
  }
  renderPreviewTable(previewHeaders, previewRows);
}

function onHeaderRightClick(event, colIdx) {
  event.preventDefault();
  selectedCol = colIdx;
  selectedCell = null;
  selectedRow = null;
  renderPreviewTable(previewHeaders, previewRows);
}

function onCellClick(rowIdx, colIdx) {
  selectedCell = [rowIdx, colIdx];
  selectedRow = null;
  selectedCol = null;
  renderPreviewTable(previewHeaders, previewRows);
}

// Ctrl+C copy
document.addEventListener('keydown', (e) => {
  if (e.ctrlKey && e.key === 'c') {
    let text = null;
    if (selectedCell) {
      const [r, c] = selectedCell;
      if (previewRows[r] && previewRows[r][c] !== undefined) {
        text = previewRows[r][c];
      }
    } else if (selectedRow !== null) {
      if (previewRows[selectedRow]) {
        text = previewRows[selectedRow].join('\t');
      }
    } else if (selectedCol !== null) {
      text = previewRows.map(r => r[selectedCol] || '').join('\n');
    }
    if (text !== null) {
      navigator.clipboard.writeText(text);
    }
  }
});

// ─── Stats Table ────────────────────────────────────────────────────────────
function renderStatsTable(stats) {
  const wrapper = document.getElementById('stats-table-wrapper');
  let html = '<table><thead><tr>';
  ['Name', 'Type', 'Min', 'Max', 'Nulls', 'Errors'].forEach(h => {
    html += `<th>${h}</th>`;
  });
  html += '</tr></thead><tbody>';
  stats.forEach(s => {
    html += `<tr>
      <td>${escHtml(s.name)}</td>
      <td>${escHtml(s.dtype)}</td>
      <td>${s.min !== null ? escHtml(s.min) : '-'}</td>
      <td>${s.max !== null ? escHtml(s.max) : '-'}</td>
      <td>${s.null_count}</td>
      <td>${s.error_count}</td>
    </tr>`;
  });
  html += '</tbody></table>';
  wrapper.innerHTML = html;
}

// ─── Modify Tab ─────────────────────────────────────────────────────────────
function updateModifyTab() {
  if (!dataLoaded) return;
  document.getElementById('modify-no-data').classList.add('hidden');
  document.getElementById('modify-content').classList.remove('hidden');
  refreshPipelineList();
  onOpTypeChange();
  renderModifyTable();
}

async function refreshPipelineList() {
  const ops = await invoke('get_operations');
  const el = document.getElementById('pipeline-list');
  if (ops.length === 0) {
    el.innerHTML = '<em>No operations yet.</em>';
  } else {
    el.innerHTML = ops.map((op, i) =>
      `<div class="pipeline-item">
        <span class="op-num">${i + 1}.</span>
        <span class="op-desc">${escHtml(op)}</span>
        <button onclick="removeOp(${i})">✕</button>
      </div>`
    ).join('');
  }
  updatePipelineStatus();
}

async function removeOp(index) {
  try {
    await invoke('remove_operation', { index });
    setStatus('Operation removed');
    await loadPreview();
  } catch (e) {
    setStatus('Error: ' + e);
  }
}

async function undoOp() {
  try {
    const result = await invoke('undo_operation');
    setStatus(result ? 'Undo' : 'Nothing to undo');
    await loadPreview();
  } catch (e) {
    setStatus('Error: ' + e);
  }
}

async function redoOp() {
  try {
    const result = await invoke('redo_operation');
    setStatus(result ? 'Redo' : 'Nothing to redo');
    await loadPreview();
  } catch (e) {
    setStatus('Error: ' + e);
  }
}

async function clearPipeline() {
  try {
    await invoke('clear_pipeline');
    setStatus('Pipeline cleared');
    await loadPreview();
  } catch (e) {
    setStatus('Error: ' + e);
  }
}

function renderModifyTable() {
  const container = document.getElementById('modify-table-container');
  if (previewRows.length === 0) {
    container.innerHTML = '';
    return;
  }
  let html = `<div style="font-size:12px;color:var(--fg2);margin-bottom:4px">Preview (${previewRows.length} rows × ${previewHeaders.length} cols)</div>`;
  html += '<div style="overflow:auto;max-height:calc(100vh - 380px);border:1px solid var(--bg3);border-radius:4px"><table><thead><tr>';
  previewHeaders.forEach(h => {
    html += `<th>${escHtml(h)}</th>`;
  });
  html += '</tr></thead><tbody>';
  previewRows.forEach(row => {
    html += '<tr>';
    row.forEach(cell => {
      html += `<td>${escHtml(cell)}</td>`;
    });
    html += '</tr>';
  });
  html += '</tbody></table></div>';
  container.innerHTML = html;
}

// ─── Operation Builder ──────────────────────────────────────────────────────
function onOpTypeChange() {
  const opType = document.getElementById('op-type-select').value;
  const fields = document.getElementById('op-builder-fields');
  let html = '';

  const colOptions = columnNames.map(n => `<option value="${escHtml(n)}">${escHtml(n)}</option>`).join('');

  switch (opType) {
    case 'filter':
      html = `
        <label>Column<select id="op-col">${colOptions}</select></label>
        <label>Operator
          <select id="op-filter-op">
            <option value="eq">=</option>
            <option value="neq">≠</option>
            <option value="gt">&gt;</option>
            <option value="gte">≥</option>
            <option value="lt">&lt;</option>
            <option value="lte">≤</option>
            <option value="contains">contains</option>
            <option value="is_null">is null</option>
            <option value="is_not_null">is not null</option>
          </select>
        </label>
        <label>Value<input type="text" id="op-value" /></label>
        <button onclick="applyOperation()">Apply Filter</button>`;
      break;
    case 'sort':
      html = `
        <label>Column<select id="op-col">${colOptions}</select></label>
        <label><input type="checkbox" id="op-descending" /> Descending</label>
        <button onclick="applyOperation()">Apply Sort</button>`;
      break;
    case 'drop_column':
      html = `
        <label>Column to drop<select id="op-col">${colOptions}</select></label>
        <button onclick="applyOperation()">Drop Column</button>`;
      break;
    case 'rename_column':
      html = `
        <label>From<select id="op-rename-from">${colOptions}</select></label>
        <label>To<input type="text" id="op-rename-to" /></label>
        <button onclick="applyOperation()">Rename Column</button>`;
      break;
    case 'select_columns':
      html = `<div>Select columns to keep:</div><div class="checkbox-grid">`;
      columnNames.forEach((n, i) => {
        html += `<label><input type="checkbox" class="sel-col-check" value="${escHtml(n)}" checked /> ${escHtml(n)}</label>`;
      });
      html += `</div>
        <div style="display:flex;gap:4px;margin:4px 0">
          <button onclick="selectAllCols(true)">Select All</button>
          <button onclick="selectAllCols(false)">Deselect All</button>
        </div>
        <button onclick="applyOperation()">Apply Selection</button>`;
      break;
    case 'limit':
      html = `
        <label>Max rows<input type="number" id="op-limit" value="1000" min="1" /></label>
        <button onclick="applyOperation()">Apply Limit</button>`;
      break;
    case 'fill_null':
      html = `
        <label>Column<select id="op-col">${colOptions}</select></label>
        <label>Strategy
          <select id="op-fill-strategy" onchange="onFillStratChange()">
            <option value="forward">Forward Fill</option>
            <option value="backward">Backward Fill</option>
            <option value="with_value">With Value</option>
            <option value="mean">Mean</option>
            <option value="min">Min</option>
            <option value="max">Max</option>
          </select>
        </label>
        <div id="fill-value-row" class="hidden">
          <label>Fill value<input type="text" id="op-fill-value" /></label>
        </div>
        <button onclick="applyOperation()">Apply Fill Null</button>`;
      break;
    case 'cast_column':
      html = `
        <label>Column<select id="op-col">${colOptions}</select></label>
        <label>Target type
          <select id="op-cast-dtype">
            <option value="Int32">Int32</option>
            <option value="Int64">Int64</option>
            <option value="Float32">Float32</option>
            <option value="Float64" selected>Float64</option>
            <option value="String">String</option>
            <option value="Boolean">Boolean</option>
            <option value="Date">Date</option>
          </select>
        </label>
        <button onclick="applyOperation()">Apply Cast</button>`;
      break;
    case 'parse_datetime':
      html = `
        <label>Column<select id="op-col">${colOptions}</select></label>
        <label>Format<input type="text" id="op-dt-format" value="%Y-%m-%d %H:%M:%S" /></label>
        <span style="font-size:11px;color:var(--fg3)">e.g. %Y-%m-%d %H:%M:%S</span>
        <button onclick="applyOperation()">Parse Datetime</button>`;
      break;
  }
  fields.innerHTML = html;
}

function onFillStratChange() {
  const strat = document.getElementById('op-fill-strategy').value;
  const row = document.getElementById('fill-value-row');
  if (strat === 'with_value') {
    row.classList.remove('hidden');
  } else {
    row.classList.add('hidden');
  }
}

function selectAllCols(checked) {
  document.querySelectorAll('.sel-col-check').forEach(cb => cb.checked = checked);
}

async function applyOperation() {
  const opType = document.getElementById('op-type-select').value;
  const input = { op_type: opType };

  try {
    switch (opType) {
      case 'filter':
        input.column = document.getElementById('op-col').value;
        input.filter_op = document.getElementById('op-filter-op').value;
        input.value = document.getElementById('op-value').value;
        break;
      case 'sort':
        input.column = document.getElementById('op-col').value;
        input.descending = document.getElementById('op-descending').checked;
        break;
      case 'drop_column':
        input.column = document.getElementById('op-col').value;
        break;
      case 'rename_column':
        input.rename_from = document.getElementById('op-rename-from').value;
        input.rename_to = document.getElementById('op-rename-to').value;
        break;
      case 'select_columns': {
        const checks = document.querySelectorAll('.sel-col-check:checked');
        input.columns = Array.from(checks).map(cb => cb.value);
        if (input.columns.length === 0) { setStatus('Select at least one column'); return; }
        break;
      }
      case 'limit':
        input.limit = parseInt(document.getElementById('op-limit').value);
        break;
      case 'fill_null':
        input.column = document.getElementById('op-col').value;
        input.fill_strategy = document.getElementById('op-fill-strategy').value;
        if (input.fill_strategy === 'with_value') {
          input.fill_value = document.getElementById('op-fill-value').value;
        }
        break;
      case 'cast_column':
        input.column = document.getElementById('op-col').value;
        input.cast_dtype = document.getElementById('op-cast-dtype').value;
        break;
      case 'parse_datetime':
        input.column = document.getElementById('op-col').value;
        input.datetime_format = document.getElementById('op-dt-format').value;
        break;
    }

    const desc = await invoke('add_operation', { input });
    setStatus('Applied: ' + desc);
    await loadPreview();
  } catch (e) {
    setStatus('Error: ' + e);
  }
}

// ─── Export ─────────────────────────────────────────────────────────────────
async function exportData() {
  try {
    const format = document.getElementById('export-format').value;
    const ext = format === 'csv' ? 'csv' : 'parquet';
    const path = await invoke('pick_save_path', { ext });
    if (path) {
      const msg = await invoke('export_data', { path, format });
      setStatus(msg);
    }
  } catch (e) {
    setStatus('Error: ' + e);
  }
}

// ─── Visualize Tab ──────────────────────────────────────────────────────────
function updateVisualizeControls() {
  if (!dataLoaded) return;
  document.getElementById('viz-no-data').classList.add('hidden');
  document.getElementById('viz-content').classList.remove('hidden');

  // Populate X column select
  const xSel = document.getElementById('plot-x-select');
  const currentX = xSel.value;
  xSel.innerHTML = '<option value="">(select)</option>' +
    columnNames.map(n => `<option value="${escHtml(n)}"${n === currentX ? ' selected' : ''}>${escHtml(n)}</option>`).join('');

  updateYSeriesList();
}

function updateYSeriesList() {
  const container = document.getElementById('y-series-list');
  container.innerHTML = plotYColumns.map((col, i) =>
    `<span class="y-series-tag">${escHtml(col)} <button onclick="removeYCol(${i})">✕</button></span>`
  ).join('');
}

function removeYCol(index) {
  plotYColumns.splice(index, 1);
  updateYSeriesList();
  onPlotConfigChange();
}

function toggleYDropdown() {
  const dd = document.getElementById('y-dropdown');
  if (dd.classList.contains('hidden')) {
    const list = document.getElementById('y-dropdown-list');
    list.innerHTML = columnNames
      .filter(n => !plotYColumns.includes(n))
      .map(n => `<button onclick="addYCol('${escAttr(n)}')">${escHtml(n)}</button>`)
      .join('');
    dd.classList.remove('hidden');
  } else {
    dd.classList.add('hidden');
  }
}

function addYCol(name) {
  plotYColumns.push(name);
  document.getElementById('y-dropdown').classList.add('hidden');
  updateYSeriesList();
  onPlotConfigChange();
}

// Close Y dropdown when clicking outside
document.addEventListener('click', (e) => {
  if (!e.target.closest('.y-add-dropdown')) {
    document.getElementById('y-dropdown')?.classList.add('hidden');
  }
});

async function onPlotConfigChange() {
  const plotType = document.getElementById('plot-type-select').value;
  const xCol = document.getElementById('plot-x-select').value;
  const binsLabel = document.getElementById('hist-bins-label');

  if (plotType === 'histogram') {
    binsLabel.classList.remove('hidden');
  } else {
    binsLabel.classList.add('hidden');
  }

  if (plotType === 'histogram') {
    await renderHistogram();
  } else {
    await renderPlot();
  }
}

async function renderPlot() {
  const container = document.getElementById('plot-container');
  const plotType = document.getElementById('plot-type-select').value;
  const xCol = document.getElementById('plot-x-select').value;

  if (!xCol || plotYColumns.length === 0) {
    container.innerHTML = '<p style="color:var(--fg3);padding:20px">Select valid X and Y columns (must be numeric) to plot.</p>';
    return;
  }

  try {
    const data = await invoke('get_plot_data', { xCol, yCols: plotYColumns });

    if (data.series.length === 0) {
      container.innerHTML = '<p style="color:var(--fg3);padding:20px">No numeric data for selected columns.</p>';
      return;
    }

    const traces = data.series.map(s => {
      const xData = data.x_is_datetime
        ? s.x.map(v => new Date(v * 1000))
        : s.x;

      const trace = {
        x: xData,
        y: s.y,
        name: s.name,
      };

      switch (plotType) {
        case 'scatter':
          trace.type = 'scattergl';
          trace.mode = 'markers';
          trace.marker = { size: 4 };
          break;
        case 'line':
          trace.type = 'scattergl';
          trace.mode = 'lines';
          break;
        case 'bar':
          trace.type = 'bar';
          break;
      }
      return trace;
    });

    const layout = {
      paper_bgcolor: '#282828',
      plot_bgcolor: '#282828',
      font: { color: '#ebdbb2', size: 12 },
      xaxis: {
        title: xCol,
        gridcolor: '#3c3836',
        zerolinecolor: '#504945',
        type: data.x_is_datetime ? 'date' : undefined,
      },
      yaxis: {
        gridcolor: '#3c3836',
        zerolinecolor: '#504945',
      },
      legend: {
        bgcolor: 'rgba(40,40,40,0.8)',
        font: { color: '#ebdbb2' },
      },
      margin: { l: 60, r: 30, t: 30, b: 60 },
      showlegend: true,
    };

    const config = {
      responsive: true,
      displayModeBar: true,
      modeBarButtonsToRemove: ['lasso2d', 'select2d'],
      displaylogo: false,
    };

    Plotly.newPlot(container, traces, layout, config);
  } catch (e) {
    container.innerHTML = `<p style="color:var(--red);padding:20px">Plot error: ${escHtml(String(e))}</p>`;
  }
}

async function renderHistogram() {
  const container = document.getElementById('plot-container');
  const bins = parseInt(document.getElementById('hist-bins').value) || 30;

  // Use Y columns if set, else fall back to X
  let columns = plotYColumns.length > 0 ? plotYColumns : [];
  if (columns.length === 0) {
    const xCol = document.getElementById('plot-x-select').value;
    if (xCol) columns = [xCol];
  }
  if (columns.length === 0) {
    container.innerHTML = '<p style="color:var(--fg3);padding:20px">Select columns for the histogram (use Y series or X).</p>';
    return;
  }

  try {
    const data = await invoke('get_histogram_data', { columns });

    if (data.series.length === 0) {
      container.innerHTML = '<p style="color:var(--fg3);padding:20px">No numeric data in selected columns.</p>';
      return;
    }

    const traces = data.series.map(s => ({
      x: s.values,
      type: 'histogram',
      name: s.name,
      nbinsx: bins,
      opacity: data.series.length > 1 ? 0.7 : 1.0,
    }));

    const layout = {
      paper_bgcolor: '#282828',
      plot_bgcolor: '#282828',
      font: { color: '#ebdbb2', size: 12 },
      xaxis: {
        gridcolor: '#3c3836',
        zerolinecolor: '#504945',
      },
      yaxis: {
        title: 'Count',
        gridcolor: '#3c3836',
        zerolinecolor: '#504945',
      },
      legend: {
        bgcolor: 'rgba(40,40,40,0.8)',
        font: { color: '#ebdbb2' },
      },
      barmode: data.series.length > 1 ? 'overlay' : undefined,
      margin: { l: 60, r: 30, t: 30, b: 60 },
      showlegend: true,
    };

    const config = {
      responsive: true,
      displayModeBar: true,
      modeBarButtonsToRemove: ['lasso2d', 'select2d'],
      displaylogo: false,
    };

    Plotly.newPlot(container, traces, layout, config);
  } catch (e) {
    container.innerHTML = `<p style="color:var(--red);padding:20px">Histogram error: ${escHtml(String(e))}</p>`;
  }
}

function resetZoom() {
  const container = document.getElementById('plot-container');
  Plotly.relayout(container, {
    'xaxis.autorange': true,
    'yaxis.autorange': true,
  });
}

// ─── Utilities ──────────────────────────────────────────────────────────────
function escHtml(str) {
  if (str === null || str === undefined) return '';
  return String(str).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function escAttr(str) {
  return String(str).replace(/'/g, "\\'").replace(/"/g, '&quot;');
}
