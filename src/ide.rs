use std::{
    env, fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use anadil::{check_source, diagnostics::Diagnostic, run_source_diagnostic};

const IDE_HOST: &str = "127.0.0.1";
const IDE_PORT_START: u16 = 5817;
const IDE_PORT_END: u16 = 5830;

pub fn run() -> Result<(), String> {
    let (listener, port) = bind_listener()?;
    println!("Anadil IDE hazir: http://{IDE_HOST}:{port}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle_connection(stream) {
                    eprintln!("IDE istegi islenemedi: {error}");
                }
            }
            Err(error) => eprintln!("IDE baglantisi alinamadi: {error}"),
        }
    }

    Ok(())
}

fn bind_listener() -> Result<(TcpListener, u16), String> {
    for port in IDE_PORT_START..=IDE_PORT_END {
        if let Ok(listener) = TcpListener::bind((IDE_HOST, port)) {
            return Ok((listener, port));
        }
    }

    Err(format!(
        "IDE icin bos port bulunamadi: {IDE_HOST}:{IDE_PORT_START}-{IDE_PORT_END}"
    ))
}

fn handle_connection(stream: TcpStream) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|error| format!("read timeout ayarlanamadi: {error}"))?;

    let mut reader = BufReader::new(stream);
    let request_line = read_request_line(&mut reader)?;
    let headers = read_headers(&mut reader)?;
    let content_length = header_value(&headers, "content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);

    let mut body = vec![0; content_length];
    if content_length > 0 {
        reader
            .read_exact(&mut body)
            .map_err(|error| format!("istek govdesi okunamadi: {error}"))?;
    }

    let mut stream = reader.into_inner();
    let response = route_request(&request_line, &body);
    write_response(&mut stream, response)
}

fn read_request_line(reader: &mut BufReader<TcpStream>) -> Result<String, String> {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|error| format!("istek satiri okunamadi: {error}"))?;
    Ok(line.trim_end_matches(['\r', '\n']).to_string())
}

fn read_headers(reader: &mut BufReader<TcpStream>) -> Result<Vec<(String, String)>, String> {
    let mut headers = Vec::new();

    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|error| format!("header okunamadi: {error}"))?;
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }

        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_ascii_lowercase(), value.trim().to_string()));
        }
    }

    Ok(headers)
}

fn header_value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find_map(|(header_name, value)| (header_name == name).then_some(value.as_str()))
}

fn route_request(request_line: &str, body: &[u8]) -> HttpResponse {
    let mut parts = request_line.split_whitespace();
    let Some(method) = parts.next() else {
        return text_response(Status::BadRequest, "Gecersiz istek");
    };
    let Some(target) = parts.next() else {
        return text_response(Status::BadRequest, "Gecersiz istek");
    };

    match (method, target) {
        ("GET", "/") => html_response(IDE_HTML),
        ("GET", "/api/examples") => json_response(Status::Ok, &examples_json()),
        ("POST", "/api/check") => json_response(Status::Ok, &check_json(source_from_body(body))),
        ("POST", "/api/run") => json_response(Status::Ok, &run_json(source_from_body(body))),
        ("POST", "/api/build") => build_response(source_from_body(body)),
        _ if method == "GET" && target.starts_with("/api/example?") => {
            match read_example_from_target(target) {
                Ok(source) => text_response(Status::Ok, &source),
                Err(message) => json_response(
                    Status::NotFound,
                    &json_result(false, &[Diagnostic::io(message)]),
                ),
            }
        }
        _ => text_response(Status::NotFound, "Bulunamadi"),
    }
}

fn source_from_body(body: &[u8]) -> &str {
    std::str::from_utf8(body).unwrap_or("")
}

fn build_response(source: &str) -> HttpResponse {
    if let Err(diagnostic) = check_source(source) {
        return json_response(Status::Ok, &json_build_result(false, None, &[diagnostic]));
    }

    match write_ide_source(source).and_then(|path| run_native_build_json(&path)) {
        Ok(json) => json_response(Status::Ok, &json),
        Err(message) => json_response(
            Status::Ok,
            &json_build_result(false, None, &[Diagnostic::native(message)]),
        ),
    }
}

fn write_ide_source(source: &str) -> Result<PathBuf, String> {
    let dir = PathBuf::from("target").join("ide");
    fs::create_dir_all(&dir).map_err(|error| {
        format!(
            "IDE build klasoru olusturulamadi `{}`: {error}",
            dir.display()
        )
    })?;

    let path = dir.join("ide_current.ana");
    fs::write(&path, source).map_err(|error| {
        format!(
            "IDE kaynak dosyasi yazilamadi `{}`: {error}",
            path.display()
        )
    })?;
    Ok(path)
}

fn run_native_build_json(path: &Path) -> Result<String, String> {
    let exe = env::current_exe()
        .map_err(|error| format!("Calisan Anadil executable yolu bulunamadi: {error}"))?;
    let output = Command::new(exe)
        .arg("derle")
        .arg("--json")
        .arg(path)
        .output()
        .map_err(|error| format!("Native build komutu calistirilamadi: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return Ok(stdout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(stderr.trim().to_string())
}

fn examples_json() -> String {
    let examples = list_examples()
        .into_iter()
        .map(|path| {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            format!("{{\"name\":\"{}\"}}", json_escape(name))
        })
        .collect::<Vec<_>>()
        .join(",");

    format!("{{\"examples\":[{examples}]}}")
}

fn list_examples() -> Vec<PathBuf> {
    let mut examples = fs::read_dir("examples")
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "ana"))
        .collect::<Vec<_>>();

    examples.sort_by(|left, right| left.file_name().cmp(&right.file_name()));
    examples
}

fn read_example_from_target(target: &str) -> Result<String, String> {
    let query = target
        .split_once('?')
        .map(|(_, query)| query)
        .ok_or("Ornek adi eksik")?;
    let name = query
        .split('&')
        .find_map(|part| part.strip_prefix("name="))
        .map(percent_decode)
        .ok_or("Ornek adi eksik")?;

    if name.contains(['/', '\\']) || !name.ends_with(".ana") {
        return Err("Gecersiz ornek adi".to_string());
    }

    let path = PathBuf::from("examples").join(name);
    fs::read_to_string(&path)
        .map_err(|error| format!("Ornek okunamadi `{}`: {error}", path.display()))
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = String::new();
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let hex = &value[index + 1..index + 3];
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte as char);
                    index += 3;
                } else {
                    out.push('%');
                    index += 1;
                }
            }
            b'+' => {
                out.push(' ');
                index += 1;
            }
            byte => {
                out.push(byte as char);
                index += 1;
            }
        }
    }

    out
}

fn check_json(source: &str) -> String {
    match check_source(source) {
        Ok(()) => json_result(true, &[]),
        Err(diagnostic) => json_result(false, &[diagnostic]),
    }
}

fn run_json(source: &str) -> String {
    match run_source_diagnostic(source) {
        Ok(output) => json_run_result(true, &output, &[]),
        Err(diagnostic) => json_run_result(false, "", &[diagnostic]),
    }
}

fn json_result(ok: bool, diagnostics: &[Diagnostic]) -> String {
    let diagnostics = diagnostics
        .iter()
        .map(json_diagnostic)
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"ok\":{ok},\"diagnostics\":[{diagnostics}]}}")
}

fn json_run_result(ok: bool, output: &str, diagnostics: &[Diagnostic]) -> String {
    let diagnostics = diagnostics
        .iter()
        .map(json_diagnostic)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"ok\":{ok},\"output\":\"{}\",\"diagnostics\":[{diagnostics}]}}",
        json_escape(output)
    )
}

fn json_build_result(ok: bool, exe: Option<&Path>, diagnostics: &[Diagnostic]) -> String {
    let diagnostics = diagnostics
        .iter()
        .map(json_diagnostic)
        .collect::<Vec<_>>()
        .join(",");
    let exe = exe
        .map(|path| format!("\"{}\"", json_escape(&path.display().to_string())))
        .unwrap_or_else(|| "null".to_string());
    format!("{{\"ok\":{ok},\"exe\":{exe},\"diagnostics\":[{diagnostics}]}}")
}

fn json_diagnostic(diagnostic: &Diagnostic) -> String {
    let (line, column) = match diagnostic.span {
        Some(span) => (span.line.to_string(), span.column.to_string()),
        None => ("null".to_string(), "null".to_string()),
    };

    format!(
        "{{\"severity\":\"{}\",\"stage\":\"{}\",\"message\":\"{}\",\"line\":{},\"column\":{}}}",
        diagnostic.severity.as_str(),
        diagnostic.stage.as_str(),
        json_escape(&diagnostic.message),
        line,
        column
    )
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            ch if ch <= '\u{1f}' => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
}

#[derive(Clone, Copy)]
enum Status {
    Ok,
    BadRequest,
    NotFound,
}

impl Status {
    fn code(self) -> &'static str {
        match self {
            Self::Ok => "200 OK",
            Self::BadRequest => "400 Bad Request",
            Self::NotFound => "404 Not Found",
        }
    }
}

struct HttpResponse {
    status: Status,
    content_type: &'static str,
    body: Vec<u8>,
}

fn html_response(body: &str) -> HttpResponse {
    HttpResponse {
        status: Status::Ok,
        content_type: "text/html; charset=utf-8",
        body: body.as_bytes().to_vec(),
    }
}

fn json_response(status: Status, body: &str) -> HttpResponse {
    HttpResponse {
        status,
        content_type: "application/json; charset=utf-8",
        body: body.as_bytes().to_vec(),
    }
}

fn text_response(status: Status, body: &str) -> HttpResponse {
    HttpResponse {
        status,
        content_type: "text/plain; charset=utf-8",
        body: body.as_bytes().to_vec(),
    }
}

fn write_response(stream: &mut TcpStream, response: HttpResponse) -> Result<(), String> {
    write!(
        stream,
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response.status.code(),
        response.content_type,
        response.body.len()
    )
    .map_err(|error| format!("yanit header yazilamadi: {error}"))?;
    stream
        .write_all(&response.body)
        .map_err(|error| format!("yanit govdesi yazilamadi: {error}"))
}

const IDE_HTML: &str = r#"<!doctype html>
<html lang="tr">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Anadil IDE</title>
  <style>
    :root {
      color-scheme: dark;
      --bg: #111312;
      --panel: #191d1b;
      --panel-2: #202622;
      --border: #334139;
      --text: #eef5ef;
      --muted: #aab8ad;
      --soft: #d7e6d9;
      --green: #7bd88f;
      --cyan: #65c7d0;
      --yellow: #e8c15a;
      --red: #ef767a;
      --shadow: rgba(0, 0, 0, 0.3);
      font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    }

    * { box-sizing: border-box; }
    body {
      margin: 0;
      min-height: 100vh;
      background: var(--bg);
      color: var(--text);
      overflow: hidden;
    }

    .shell {
      min-height: 100vh;
      display: grid;
      grid-template-rows: 56px 1fr 220px;
    }

    .topbar {
      display: flex;
      align-items: center;
      gap: 14px;
      padding: 0 18px;
      border-bottom: 1px solid var(--border);
      background: #151816;
      box-shadow: 0 10px 30px var(--shadow);
    }

    .brand {
      display: flex;
      align-items: center;
      gap: 10px;
      min-width: 176px;
      font-weight: 700;
      letter-spacing: 0;
    }

    .mark {
      width: 28px;
      height: 28px;
      display: grid;
      place-items: center;
      border: 1px solid #6fae78;
      color: var(--green);
      font-weight: 800;
      background: #1d2a20;
    }

    .actions {
      display: flex;
      align-items: center;
      gap: 8px;
      flex-wrap: wrap;
    }

    button, .file-button {
      height: 34px;
      border: 1px solid var(--border);
      color: var(--text);
      background: var(--panel-2);
      padding: 0 12px;
      font: inherit;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      gap: 8px;
      min-width: 34px;
    }

    button:hover, .file-button:hover { border-color: #6a8472; background: #263029; }
    button.primary { border-color: #639a6d; background: #23442b; }
    button.warn { border-color: #8b7442; background: #4b3f23; }
    button.danger { border-color: #97565a; background: #4f2528; }
    input[type="file"] { display: none; }

    .status {
      margin-left: auto;
      color: var(--muted);
      font-size: 13px;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    .workspace {
      display: grid;
      grid-template-columns: 260px minmax(0, 1fr);
      min-height: 0;
    }

    .sidebar {
      border-right: 1px solid var(--border);
      background: #141715;
      padding: 14px;
      overflow: auto;
    }

    .sidebar h2, .panel-title {
      margin: 0 0 10px;
      color: var(--soft);
      font-size: 13px;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0;
    }

    .example-list {
      display: grid;
      gap: 6px;
    }

    .example {
      width: 100%;
      justify-content: space-between;
      color: var(--muted);
      background: transparent;
    }

    .example.active {
      color: var(--text);
      border-color: #618b68;
      background: #203326;
    }

    .main {
      display: grid;
      grid-template-rows: 36px minmax(0, 1fr);
      min-width: 0;
      background: var(--panel);
    }

    .editor-head {
      display: flex;
      align-items: center;
      gap: 10px;
      padding: 0 14px;
      border-bottom: 1px solid var(--border);
      color: var(--muted);
      font-size: 13px;
    }

    .filename {
      color: var(--text);
      font-weight: 650;
    }

    .editor-wrap {
      display: grid;
      grid-template-columns: 58px minmax(0, 1fr);
      min-height: 0;
      background: #111513;
      font-family: "Cascadia Code", "Fira Code", Consolas, monospace;
      font-size: 15px;
      line-height: 1.55;
    }

    .gutter {
      color: #6f7f73;
      text-align: right;
      padding: 16px 12px 16px 0;
      border-right: 1px solid #243028;
      user-select: none;
      overflow: hidden;
      white-space: pre;
    }

    textarea {
      width: 100%;
      height: 100%;
      resize: none;
      border: 0;
      outline: none;
      padding: 16px;
      background: transparent;
      color: var(--text);
      caret-color: var(--green);
      font: inherit;
      line-height: inherit;
      tab-size: 4;
      white-space: pre;
      overflow: auto;
    }

    .bottom {
      display: grid;
      grid-template-columns: minmax(0, 1fr) minmax(320px, 42%);
      border-top: 1px solid var(--border);
      background: #151816;
      min-height: 0;
    }

    .panel {
      min-width: 0;
      min-height: 0;
      padding: 12px 14px;
      overflow: auto;
    }

    .panel + .panel { border-left: 1px solid var(--border); }

    pre {
      margin: 0;
      color: #dce9de;
      white-space: pre-wrap;
      font-family: "Cascadia Code", Consolas, monospace;
      font-size: 13px;
      line-height: 1.45;
    }

    .diagnostics {
      display: grid;
      gap: 8px;
    }

    .diagnostic {
      border: 1px solid #653a3d;
      background: #2d1d1f;
      padding: 9px 10px;
    }

    .diagnostic strong {
      color: #ffb0b3;
      font-size: 13px;
    }

    .diagnostic span {
      display: block;
      margin-top: 4px;
      color: var(--muted);
      font-size: 12px;
    }

    .empty {
      color: var(--muted);
      font-size: 13px;
    }

    @media (max-width: 860px) {
      body { overflow: auto; }
      .shell { min-height: 100vh; grid-template-rows: auto auto auto; }
      .topbar { align-items: flex-start; padding: 12px; flex-direction: column; }
      .status { margin-left: 0; }
      .workspace { grid-template-columns: 1fr; }
      .sidebar { border-right: 0; border-bottom: 1px solid var(--border); }
      .main { min-height: 62vh; }
      .bottom { grid-template-columns: 1fr; }
      .panel + .panel { border-left: 0; border-top: 1px solid var(--border); }
    }
  </style>
</head>
<body>
  <div class="shell">
    <header class="topbar">
      <div class="brand"><div class="mark">A</div><span>Anadil IDE</span></div>
      <div class="actions">
        <label class="file-button" title="Dosya ac">Ac<input id="openFile" type="file" accept=".ana,.txt" /></label>
        <button id="saveBtn" title="Dosyayi kaydet">Kaydet</button>
        <button id="checkBtn" class="primary" title="Programi kontrol et">Kontrol</button>
        <button id="runBtn" class="warn" title="Interpreter ile calistir">Calistir</button>
        <button id="buildBtn" class="danger" title="Native executable uret">EXE Derle</button>
      </div>
      <div id="status" class="status">Hazir</div>
    </header>

    <section class="workspace">
      <aside class="sidebar">
        <h2>Ornekler</h2>
        <div id="examples" class="example-list"></div>
      </aside>

      <main class="main">
        <div class="editor-head">
          <span class="filename" id="filename">adsiz.ana</span>
          <span id="cursor">1:1</span>
        </div>
        <div class="editor-wrap">
          <div id="gutter" class="gutter">1</div>
          <textarea id="editor" spellcheck="false"></textarea>
        </div>
      </main>
    </section>

    <section class="bottom">
      <div class="panel">
        <div class="panel-title">Cikti</div>
        <pre id="output">Henüz calistirma yok.</pre>
      </div>
      <div class="panel">
        <div class="panel-title">Diagnostics</div>
        <div id="diagnostics" class="diagnostics"><div class="empty">Hata yok.</div></div>
      </div>
    </section>
  </div>

  <script>
    const editor = document.getElementById('editor');
    const gutter = document.getElementById('gutter');
    const statusEl = document.getElementById('status');
    const filenameEl = document.getElementById('filename');
    const cursorEl = document.getElementById('cursor');
    const outputEl = document.getElementById('output');
    const diagnosticsEl = document.getElementById('diagnostics');
    const examplesEl = document.getElementById('examples');
    let currentName = 'adsiz.ana';
    let fileHandle = null;

    const starter = `Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}

Ana() {
    sonuc: sayı = Topla(10, 20);
    yazdir(sonuc);
}
`;

    editor.value = starter;
    refreshEditorChrome();
    loadExamples();

    editor.addEventListener('input', refreshEditorChrome);
    editor.addEventListener('scroll', () => gutter.scrollTop = editor.scrollTop);
    editor.addEventListener('keyup', updateCursor);
    editor.addEventListener('click', updateCursor);
    editor.addEventListener('keydown', event => {
      if (event.key === 'Tab') {
        event.preventDefault();
        const start = editor.selectionStart;
        const end = editor.selectionEnd;
        editor.value = editor.value.slice(0, start) + '    ' + editor.value.slice(end);
        editor.selectionStart = editor.selectionEnd = start + 4;
        refreshEditorChrome();
      }
    });

    document.getElementById('checkBtn').addEventListener('click', () => postSource('/api/check', 'Kontrol edildi'));
    document.getElementById('runBtn').addEventListener('click', () => postSource('/api/run', 'Calistirildi'));
    document.getElementById('buildBtn').addEventListener('click', () => postSource('/api/build', 'Build tamamlandi'));
    document.getElementById('saveBtn').addEventListener('click', saveFile);
    document.getElementById('openFile').addEventListener('change', openPickedFile);

    async function loadExamples() {
      const response = await fetch('/api/examples');
      const data = await response.json();
      examplesEl.innerHTML = '';
      data.examples.forEach(example => {
        const button = document.createElement('button');
        button.className = 'example';
        button.textContent = example.name;
        button.addEventListener('click', () => loadExample(example.name, button));
        examplesEl.appendChild(button);
      });
    }

    async function loadExample(name, button) {
      const response = await fetch(`/api/example?name=${encodeURIComponent(name)}`);
      if (!response.ok) {
        setStatus('Ornek okunamadi');
        return;
      }
      editor.value = await response.text();
      currentName = name;
      fileHandle = null;
      filenameEl.textContent = currentName;
      document.querySelectorAll('.example').forEach(item => item.classList.remove('active'));
      button.classList.add('active');
      refreshEditorChrome();
      outputEl.textContent = 'Ornek yuklendi.';
      renderDiagnostics([]);
      setStatus(`${name} acildi`);
    }

    async function postSource(endpoint, doneText) {
      setStatus('Calisiyor...');
      const response = await fetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'text/plain; charset=utf-8' },
        body: editor.value
      });
      const data = await response.json();
      if ('output' in data) {
        outputEl.textContent = data.output || (data.ok ? 'Program cikti uretmedi.' : '');
      } else if ('exe' in data) {
        outputEl.textContent = data.ok ? `EXE: ${data.exe}` : '';
      } else {
        outputEl.textContent = data.ok ? 'Program gecerli.' : '';
      }
      renderDiagnostics(data.diagnostics || []);
      setStatus(data.ok ? doneText : 'Hata bulundu');
    }

    function renderDiagnostics(items) {
      diagnosticsEl.innerHTML = '';
      if (!items.length) {
        diagnosticsEl.innerHTML = '<div class="empty">Hata yok.</div>';
        return;
      }
      items.forEach(item => {
        const row = document.createElement('div');
        row.className = 'diagnostic';
        const place = item.line ? `satir ${item.line}, sutun ${item.column}` : 'konum yok';
        row.innerHTML = `<strong>${escapeHtml(item.stage)} / ${escapeHtml(item.severity)}</strong><span>${escapeHtml(place)}</span><div>${escapeHtml(item.message)}</div>`;
        row.addEventListener('click', () => {
          if (item.line) moveTo(item.line, item.column || 1);
        });
        diagnosticsEl.appendChild(row);
      });
    }

    async function openPickedFile(event) {
      const file = event.target.files[0];
      if (!file) return;
      editor.value = await file.text();
      currentName = file.name || 'adsiz.ana';
      fileHandle = null;
      filenameEl.textContent = currentName;
      refreshEditorChrome();
      setStatus(`${currentName} acildi`);
    }

    async function saveFile() {
      const suggestedName = currentName || 'adsiz.ana';
      if ('showSaveFilePicker' in window) {
        if (!fileHandle) {
          fileHandle = await window.showSaveFilePicker({
            suggestedName,
            types: [{ description: 'Anadil kaynak', accept: { 'text/plain': ['.ana'] } }]
          });
        }
        const writable = await fileHandle.createWritable();
        await writable.write(editor.value);
        await writable.close();
        currentName = fileHandle.name || suggestedName;
        filenameEl.textContent = currentName;
        setStatus('Kaydedildi');
        return;
      }

      const blob = new Blob([editor.value], { type: 'text/plain;charset=utf-8' });
      const link = document.createElement('a');
      link.href = URL.createObjectURL(blob);
      link.download = suggestedName;
      link.click();
      URL.revokeObjectURL(link.href);
      setStatus('Dosya indirildi');
    }

    function refreshEditorChrome() {
      const count = Math.max(1, editor.value.split('\n').length);
      gutter.textContent = Array.from({ length: count }, (_, index) => index + 1).join('\n');
      filenameEl.textContent = currentName;
      updateCursor();
    }

    function updateCursor() {
      const before = editor.value.slice(0, editor.selectionStart);
      const lines = before.split('\n');
      cursorEl.textContent = `${lines.length}:${lines[lines.length - 1].length + 1}`;
    }

    function moveTo(line, column) {
      const lines = editor.value.split('\n');
      let pos = 0;
      for (let i = 0; i < Math.max(0, line - 1); i++) pos += lines[i].length + 1;
      pos += Math.max(0, column - 1);
      editor.focus();
      editor.selectionStart = editor.selectionEnd = Math.min(pos, editor.value.length);
      updateCursor();
    }

    function setStatus(message) {
      statusEl.textContent = message;
    }

    function escapeHtml(value) {
      return String(value).replace(/[&<>"']/g, ch => ({
        '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#039;'
      })[ch]);
    }
  </script>
</body>
</html>
"#;
