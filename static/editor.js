window.initCodeMirror = function (textareaId, mode) {
  try {
    if (!window.CodeMirror) return;
    const ta = document.getElementById(textareaId);
    if (!ta) return;
    if (ta._cm) {
      try {
        ta._cm.setOption('mode', mode || 'text/plain');
        return;
      } catch (_) {}
    }
    const cm = window.CodeMirror.fromTextArea(ta, {
      mode: mode || 'text/plain',
      lineNumbers: true,
      matchBrackets: true,
      autoCloseBrackets: true,
      foldGutter: true,
      gutters: ["CodeMirror-linenumbers", "CodeMirror-foldgutter"]
    });
    ta._cm = cm;
  } catch (e) {
    console.error('initCodeMirror error', e);
  }
};

window.destroyCodeMirror = function (textareaId) {
  try {
    const ta = document.getElementById(textareaId);
    if (ta && ta._cm) {
      ta._cm.toTextArea();
      ta._cm = null;
    }
  } catch (e) {
    console.error('destroyCodeMirror error', e);
  }
};

window.highlightCode = function (codeId) {
  try {
    if (!window.hljs) return;
    const el = document.getElementById(codeId);
    if (el) window.hljs.highlightElement(el);
  } catch (e) {
    console.error('highlightCode error', e);
  }
};

window.createDiffHtml = function(oldStr, newStr) {
  try {
    if (!window.Diff) return `<pre>${newStr}</pre>`;
    const parts = window.Diff.diffLines(oldStr || '', newStr || '');
    let html = '<pre style="white-space:pre-wrap">';
    parts.forEach(part => {
      const color = part.added ? '#e6ffed' : part.removed ? '#ffeef0' : 'transparent';
      const sign = part.added ? '+ ' : part.removed ? '- ' : '  ';
      const safe = (part.value || '').replace(/[&<>]/g, s => ({'&':'&amp;','<':'&lt;','>':'&gt;'}[s]));
      html += `<div style="background:${color}">${sign}${safe}</div>`;
    });
    html += '</pre>';
    return html;
  } catch (e) {
    console.error('createDiffHtml error', e);
    return `<pre>${newStr}</pre>`;
  }
};

window.createSideBySideDiffHtml = function(leftStr, rightStr) {
  try {
    if (!window.Diff) {
      return `<div style="display:flex;gap:12px">
        <pre style="flex:1;white-space:pre-wrap">${leftStr || ''}</pre>
        <pre style="flex:1;white-space:pre-wrap">${rightStr || ''}</pre>
      </div>`;
    }
    const diffs = window.Diff.diffLines(leftStr || '', rightStr || '');
    const leftLines = [];
    const rightLines = [];
    diffs.forEach(part => {
      const lines = (part.value || '').split('\n');
      // Remove trailing last empty from split if exists
      if (lines.length && lines[lines.length - 1] === '') lines.pop();
      if (part.added) {
        // only in right
        lines.forEach(l => {
          leftLines.push({ text: '', cls: '' });
          rightLines.push({ text: '+ ' + l, cls: 'bg-add' });
        });
      } else if (part.removed) {
        // only in left
        lines.forEach(l => {
          leftLines.push({ text: '- ' + l, cls: 'bg-del' });
          rightLines.push({ text: '', cls: '' });
        });
      } else {
        // unchanged
        lines.forEach(l => {
          leftLines.push({ text: '  ' + l, cls: '' });
          rightLines.push({ text: '  ' + l, cls: '' });
        });
      }
    });
    const safe = s => (s || '').replace(/[&<>]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;'}[c]));
    let html = `
      <style>
        .diff-grid{display:grid;grid-template-columns:1fr 1fr;gap:12px}
        .diff-col{border:1px solid #e5e5e5;border-radius:4px;padding:8px;max-height:60vh;overflow:auto}
        .bg-add{background:#e6ffed}
        .bg-del{background:#ffeef0}
        .diff-line{white-space:pre-wrap}
      </style>
      <div class="diff-grid">
        <div class="diff-col" id="diff-left">
          ${leftLines.map(l => `<div class="diff-line ${l.cls}">${safe(l.text)}</div>`).join('')}
        </div>
        <div class="diff-col" id="diff-right">
          ${rightLines.map(l => `<div class="diff-line ${l.cls}">${safe(l.text)}</div>`).join('')}
        </div>
      </div>
    `;
    return html;
  } catch (e) {
    console.error('createSideBySideDiffHtml error', e);
    return `<div style="display:flex;gap:12px">
      <pre style="flex:1;white-space:pre-wrap">${leftStr || ''}</pre>
      <pre style="flex:1;white-space:pre-wrap">${rightStr || ''}</pre>
    </div>`;
  }
};

window.showToast = function(message, type) {
  try {
    const containerId = 'toast-container';
    let container = document.getElementById(containerId);
    if (!container) {
      container = document.createElement('div');
      container.id = containerId;
      container.style.position = 'fixed';
      container.style.top = '20px';
      container.style.right = '20px';
      container.style.zIndex = '9999';
      document.body.appendChild(container);
    }
    const bg = type === 'error' ? '#dc3545' : type === 'success' ? '#198754' : '#0d6efd';
    const toast = document.createElement('div');
    toast.style.background = bg;
    toast.style.color = '#fff';
    toast.style.padding = '10px 14px';
    toast.style.marginTop = '10px';
    toast.style.borderRadius = '4px';
    toast.style.boxShadow = '0 2px 8px rgba(0,0,0,.2)';
    toast.textContent = message;
    container.appendChild(toast);
    setTimeout(() => {
      if (container.contains(toast)) container.removeChild(toast);
    }, 3000);
  } catch (e) {
    console.error('showToast error', e);
  }
};

