---
{
  "id": "website-api",
  "locale": "en",
  "title": "Searchable API reference",
  "summary": "The existing searchable Lua, type, and resource reference.",
  "view": "api",
  "navigation": {
    "path": [
      "MODDING"
    ],
    "label": "Modding API"
  }
}
---


    <div class='page-intro'><div class='eyebrow'>API REFERENCE</div><h1>Find a function<br>and use it in your mod.</h1><p class='lead'>Search for a function name or describe what your mod should do. Results explain the purpose, inputs, and result. Practical tasks link to a step-by-step guide.</p></div>
    <div class='callout info'><b>How to use the search</b><span>Choose EML or ShroudForge to narrow the results. Click a row to open its explanation. You will find the signature, parameters, return value, and related guides. Copy a Lua example when one is available.</span></div>
    <div class='api-examples'><span>Common tasks</span><button type='button' data-api-query='shroudforge.notifications.publish'>Show a Modloader notice</button><button type='button' data-api-query='shroudforge.settings.get'>Read a mod setting</button><button type='button' data-api-query='shroudforge.ui.on_action'>Connect a Modloader button to Lua</button><button type='button' data-api-query='runtime.network.send_mod'>Send a message between mod processes</button><button type='button' data-api-query='shroudforge.log.info'>Write a mod log message</button></div>
    <div class='catalog-meta'><span class='status-dot'></span><span id='build-state'>Loading Lua API …</span><span class='catalog-version' id='catalog-version'></span></div>
    <div class='catalog-tabs'><button class='active' data-catalog='api'>Lua functions</button><button data-catalog='types'>Game details</button><button data-catalog='resources'>Game files</button></div>
    <section id='catalog-api' class='catalog-section'><div class='catalog-tools'><div class='catalog-switch'><button class='active' data-api-source='all'>All</button><button data-api-source='eml'>EML</button><button data-api-source='sf'>ShroudForge</button></div><input id='api-search' type='search' aria-label='Search the Lua API' placeholder='Search a task or function name…'></div><div id='api-result-count' class='api-result-count' aria-live='polite'></div><div id='api-results' class='catalog-results'></div></section>
    <section id='catalog-types' class='catalog-section hidden'><div class='catalog-tools'><p class='muted'>Find names and values from a specific Enshrouded version.</p><input id='type-search' type='search' placeholder='Search a name or field…'></div><div id='type-results' class='catalog-results'></div></section>
    <section id='catalog-resources' class='catalog-section hidden'><div class='catalog-tools'><p class='muted'>Find names and ID numbers for game files.</p><input id='resource-search' type='search' placeholder='Search a file type or ID…'></div><div id='resource-results' class='catalog-results'></div></section>
    <div class='callout warning'><b>Data for the selected game version</b><span>Game information belongs to one specific version and may change after an update.</span></div>
  