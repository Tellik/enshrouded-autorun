---
{
  "id": "website-server-files",
  "locale": "en",
  "title": "Install the server files",
  "summary": "The existing guide to downloading and copying the server files.",
  "view": "server-files",
  "navigation": {
    "path": [
      "QUICKSTART"
    ],
    "label": "Server files"
  }
}
---


    <div class="page-intro"><div class="eyebrow">QUICKSTART · MULTIPLAYER</div><h1>Install ShroudForge<br>server files.</h1><p class="lead">These shared steps apply to Windows and Proton Docker servers. After copying the files, choose the matching server guide from the Quickstart menu.</p></div>
    <section class="server-docker-guide"><article><h3>Download and extract the release</h3><p>Stop the server and back up its world. Download the <a href="https://github.com/bonsaibauer/shroudforge/releases/latest">latest ShroudForge release</a>. Right-click the ZIP and choose <b>Extract All</b>. Keep the extracted files together.</p></article><article><h3>Open the server installation folder</h3><p>Find the folder containing <code>enshrouded_server.exe</code>. ShroudForge goes directly in this folder, beside the server executable.</p><p>For Proton Docker, the host folder is mounted into the container. See <a href="#server-docker" data-view="server-docker">Proton Docker</a> to locate it. In the <code>mornedhels/enshrouded-server</code> image, the executable is usually in the mounted folder's <code>server</code> subfolder. Confirm the path before copying files.</p></article><article><h3>Copy all ShroudForge files</h3><p>Copy <b>all files and folders</b> from the extracted release ZIP into the folder containing <code>enshrouded_server.exe</code>. The release includes DLL files and the <code>shroudforge</code> folder. Allow files to be copied into the destination if asked.</p></article><article><h3>Copy the mod to the server</h3><p>Unpack the mod download and copy the mod's folder into <code>mods</code> in the same server folder. Follow the mod creator's instructions for any additional files or client installation.</p></article></section>
    <div class="path-cards"><button data-view="server-windows"><b>WINDOWS</b><strong>Windows Dedicated Server</strong><small>Open server mod settings and start the server.</small><span>Open guide →</span></button><button data-view="server-docker"><b>PROTON · DOCKER</b><strong>Recreate the container</strong><small>Keep its current settings and add the ShroudForge environment value.</small><span>Open guide →</span></button></div>