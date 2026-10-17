---
{
  "id": "website-server-windows",
  "locale": "en",
  "title": "Run a server on Windows",
  "summary": "The existing step-by-step guide for Windows servers.",
  "view": "server-windows",
  "navigation": {
    "path": [
      "QUICKSTART"
    ],
    "label": "Windows Dedicated Server"
  }
}
---


    <div class="page-intro"><div class="eyebrow">QUICKSTART · MULTIPLAYER · WINDOWS</div><h1>Windows Dedicated<br>Server.</h1><p class="lead">First complete <a href="#server-files" data-view="server-files">Install ShroudForge server files</a>. Then use this page to select the server folder and enable the mod.</p></div>
    <section class="server-docker-guide"><article><h3>Open the server's mod settings</h3><p>On your Windows PC, open PowerShell in the folder containing the ShroudForge release files. Run this command and replace the example path with the folder containing the server's <code>enshrouded_server.exe</code>:</p><code class="command">.\shroudforge\shroudforge.exe --root "D:\Games\EnshroudedServer\server" --desktop --target server</code><p>Enable the mod in the ShroudForge window. The command opens server settings for the selected folder.</p></article><article><h3>Start the server</h3><p>Start the Dedicated Server using its usual Steam or hosting panel method. The server loads the files from its own installation folder.</p></article><article><h3>Check the server log</h3><p>Server messages are written to <code>shroudforge/logs/shroudforge-server.log</code> in the server folder.</p></article></section>
    <div class="callout info"><b>No Proton setting needed</b><span><code>WINEDLLOVERRIDES</code> is only for the Proton Docker image. You do not need it for the Windows game or the Windows Dedicated Server installed through Steam.</span></div>