<script lang="ts">
  import { onMount } from "svelte";
  import {
    customAgents,
    toggleAppInstalled,
    refreshTools,
    toolInfos,
    connectRuangapi,
    ruangapiConnected,
    ruangapiKey,
    ruangapiMcpUrl,
    agentNames,
    skillCatalog,
    refreshSkillCatalog,
    saveAgentSkills,
    createSkillFile,
    addAgentFromTemplate,
    type SkillCatalogEntry,
  } from "../lib/store";
  import { agents as seedList } from "../lib/agents";
  import { AGENT_TEMPLATES, SOON_APPS, APP_TILES, APP_CAT_KEYS, appDescKey, type AgentTemplateDef } from "../lib/marketplace";
  import { t } from "../lib/i18n";

  type Tab = "agents" | "apps" | "skills";
  let tab = $state<Tab>("agents");
  let query = $state("");
  let filter = $state("all");

  // Backend registries: tool connection status + real skill catalog.
  onMount(() => {
    void refreshTools();
    void refreshSkillCatalog();
  });
  $effect(() => {
    // Load the skill catalog the first time the Skills tab opens.
    if (tab === "skills" && !$skillCatalog) void refreshSkillCatalog();
  });

  const allAgents = $derived([...seedList, ...$customAgents]);
  function agentLabel(id: string): string {
    const a = allAgents.find((x) => x.id === id);
    if (!a) return id;
    return $agentNames[a.id] ?? (a.nameKey ? $t(a.nameKey) : a.name);
  }

  /* ---------------- Agents tab ---------------- */
  const addedTplIds = $derived(new Set($customAgents.map((a) => a.id)));
  let adding = $state<string | null>(null);
  async function onAddTemplate(tpl: AgentTemplateDef) {
    if (adding) return;
    adding = tpl.slug;
    try {
      await addAgentFromTemplate(tpl);
    } finally {
      adding = null;
    }
  }
  const filteredAgents = $derived(
    AGENT_TEMPLATES.filter(
      (tpl) =>
        !query ||
        tpl.name.toLowerCase().includes(query.toLowerCase()) ||
        tpl.desc.toLowerCase().includes(query.toLowerCase()),
    ).sort((a, b) =>
      filter === "za" ? b.name.localeCompare(a.name) : a.name.localeCompare(b.name),
    ),
  );

  /* ---------------- Apps tab ---------------- */
  const apps = $derived(
    ($toolInfos ?? [])
      .map((ti) => ({
        ...ti,
        tile: APP_TILES[ti.id] ?? { color: "#6b7280", glyph: ti.name.slice(0, 1).toUpperCase() },
      }))
      .filter(
        (a) =>
          !query ||
          a.name.toLowerCase().includes(query.toLowerCase()) ||
          (a.category ?? "").toLowerCase().includes(query.toLowerCase()),
      )
      .filter((a) =>
        filter === "connected" ? a.connected :
        filter === "notConnected" ? !a.connected : true,
      ),
  );
  const soonApps = $derived(
    SOON_APPS.filter(
      (a) =>
        !query ||
        a.name.toLowerCase().includes(query.toLowerCase()) ||
        a.desc.toLowerCase().includes(query.toLowerCase()),
    ),
  );

  // RuangAPI credentials form (kept inside its own card, Patria 19 Okt).
  let raKey = $state("");
  let raMcpUrl = $state("");
  let raInit = $state(false);
  $effect(() => {
    if (!raInit) {
      raKey = $ruangapiKey;
      raMcpUrl = $ruangapiMcpUrl;
      raInit = true;
    }
  });
  function connectRuangapiForm() {
    connectRuangapi(raKey);
  }

  /* ---------------- Skills tab ---------------- */
  const skills = $derived(
    ($skillCatalog ?? [])
      .filter(
        (s) =>
          !query ||
          s.name.toLowerCase().includes(query.toLowerCase()) ||
          s.description.toLowerCase().includes(query.toLowerCase()),
      )
      .filter((s) =>
        filter === "bundled" ? s.source === "bundled" :
        filter === "user" ? s.source === "user" : true,
      )
      .sort((a, b) => a.name.localeCompare(b.name)),
  );

  // Per-card agent picker (assign skills to the chosen agents).
  let pickerSkill = $state<string | null>(null);
  let pickerSelection = $state<string[]>([]);
  let saving = $state(false);
  function openPicker(s: SkillCatalogEntry) {
    pickerSkill = s.id;
    pickerSelection = [...s.agents];
  }
  function closePicker() {
    pickerSkill = null;
    pickerSelection = [];
  }
  function togglePickerAgent(id: string) {
    pickerSelection = pickerSelection.includes(id)
      ? pickerSelection.filter((x) => x !== id)
      : [...pickerSelection, id];
  }
  async function savePicker() {
    const s = ($skillCatalog ?? []).find((x) => x.id === pickerSkill);
    if (!s || saving) return;
    saving = true;
    try {
      const before = new Set(s.agents);
      const after = new Set(pickerSelection);
      const changed = allAgents
        .map((a) => a.id)
        .filter((id) => before.has(id) !== after.has(id));
      for (const id of changed) {
        const current = ($skillCatalog ?? [])
          .filter((x) => x.agents.includes(id))
          .map((x) => x.id)
          .filter((x) => x !== s.id);
        const next = after.has(id) ? [...current, s.id] : current;
        await saveAgentSkills(id, next);
      }
      await refreshSkillCatalog();
      closePicker();
    } finally {
      saving = false;
    }
  }

  // Create-skill dialog (writes a real SKILL.md through the backend).
  let showCreate = $state(false);
  let newName = $state("");
  let newDesc = $state("");
  let createError = $state("");
  async function createSkill() {
    createError = "";
    const entry = await createSkillFile(newName.trim(), newDesc.trim());
    if (entry) {
      showCreate = false;
      newName = "";
      newDesc = "";
      await refreshSkillCatalog();
    } else {
      createError = $t("mkt.createFail");
    }
  }
</script>

<div class="mkt">
  <div class="mkt-head">
    <div class="mkt-titles">
      <h2>{$t("mkt.title")}</h2>
      <p>{$t("mkt.sub")}</p>
    </div>
    <div class="mkt-controls">
      <div class="seg" role="tablist" aria-label={$t("mkt.title")}>
        <button role="tab" class:active={tab === "agents"} onclick={() => { tab = "agents"; filter = "all"; query = ""; }}>
          {$t("mkt.tab.agents")}
        </button>
        <button role="tab" class:active={tab === "apps"} onclick={() => { tab = "apps"; filter = "all"; query = ""; }}>
          {$t("mkt.tab.apps")}
        </button>
        <button role="tab" class:active={tab === "skills"} onclick={() => { tab = "skills"; filter = "all"; query = ""; }}>
          {$t("mkt.tab.skills")}
        </button>
      </div>
      <div class="mkt-tools">
        {#if tab === "skills"}
          <button class="btn-primary" onclick={() => { showCreate = true; createError = ""; }}>
            {$t("mkt.createSkill")}
          </button>
        {/if}
        <input
          class="mkt-search"
          type="search"
          placeholder={$t("mkt.search")}
          bind:value={query}
          aria-label={$t("mkt.search")}
        />
        <label class="mkt-filter">
          <span>{$t("mkt.filter")}</span>
          <select bind:value={filter} aria-label={$t("mkt.filter")}>
            <option value="all">{$t("mkt.filter.all")}</option>
            {#if tab === "agents"}
              <option value="az">{$t("mkt.sort.az")}</option>
              <option value="za">{$t("mkt.sort.za")}</option>
            {:else if tab === "apps"}
              <option value="connected">{$t("mkt.filter.connected")}</option>
              <option value="notConnected">{$t("mkt.filter.notConnected")}</option>
            {:else}
              <option value="bundled">{$t("mkt.filter.bundled")}</option>
              <option value="user">{$t("mkt.filter.user")}</option>
            {/if}
          </select>
        </label>
      </div>
    </div>
  </div>

  {#if tab === "agents"}
    {#if filteredAgents.length === 0}
      <p class="mkt-empty">{$t("mkt.noResults")}</p>
    {:else}
      <div class="mkt-grid">
        {#each filteredAgents as tpl (tpl.slug)}
          {@const added = addedTplIds.has(`tpl-${tpl.slug}`)}
          <article class="mkt-card">
            <div class="mkt-top">
              <span class="mkt-tile sh-{tpl.shape}" style:background={tpl.color} aria-hidden="true">
                <span class="eye"></span><span class="eye"></span>
              </span>
              <h3>{tpl.name}</h3>
              <button
                class="mkt-add"
                class:added
                onclick={() => onAddTemplate(tpl)}
                disabled={added || adding === tpl.slug}
              >
                {#if added}
                  {$t("mkt.added")}
                {:else if adding === tpl.slug}
                  {$t("mkt.adding")}
                {:else}
                  {$t("mkt.add")}
                {/if}
              </button>
            </div>
            <p class="mkt-desc">{tpl.desc}</p>
            <footer class="mkt-foot">
              <span>Minime</span>
              <span class="dot">·</span>
              <span>{$t("mkt.footer.skillsN", { n: tpl.skills.length })}</span>
            </footer>
          </article>
        {/each}
      </div>
    {/if}
  {:else if tab === "apps"}
    {#if apps.length === 0 && soonApps.length === 0}
      <p class="mkt-empty">{$t("mkt.noResults")}</p>
    {:else}
      <div class="mkt-grid">
        {#each apps as app (app.id)}
          <article class="mkt-card">
            <div class="mkt-top">
              <span class="mkt-tile sh-squircle" style:background={app.tile.color} aria-hidden="true">
                <span class="glyph">{app.tile.glyph}</span>
              </span>
              <h3>{app.name}</h3>
              <button
                class="mkt-add"
                class:added={app.connected}
                onclick={() => toggleAppInstalled(app.id)}
              >
                {#if app.connected}
                  {$t("mkt.connected")}
                {:else}
                  {$t("mkt.connect")}
                {/if}
              </button>
            </div>
            <p class="mkt-desc">{$t(appDescKey(app.id))}</p>
            {#if app.id === "ruangapi"}
              <div class="ra-form">
                <div class="field">
                  <label for="ra-key">{$t("tools.ruangapi.key")}</label>
                  <input
                    id="ra-key"
                    type="password"
                    bind:value={raKey}
                    placeholder={$t("tools.ruangapi.keyPlaceholder")}
                    autocomplete="off"
                  />
                </div>
                <div class="field">
                  <label for="ra-mcp">{$t("tools.ruangapi.mcpUrl")}</label>
                  <input id="ra-mcp" type="text" bind:value={raMcpUrl} inputmode="url" />
                </div>
                <div class="ra-actions">
                  <button class="btn-ghost" onclick={connectRuangapiForm} disabled={!raKey.trim()}>
                    {$t("tools.connect")}
                  </button>
                  <a class="link" href="https://ruangapi.id/mcp" target="_blank" rel="noopener">
                    {$t("tools.ruangapi.docs")}
                  </a>
                </div>
                {#if $ruangapiConnected}
                  <span class="ok-note" role="status">{$t("tools.ruangapi.connected")}</span>
                {/if}
              </div>
            {/if}
            <footer class="mkt-foot">
              <span>Minime</span>
              <span class="dot">·</span>
              <span>{$t(APP_CAT_KEYS[app.category] ?? "mkt.cat.other")}</span>
            </footer>
          </article>
        {/each}
      </div>
      {#if soonApps.length > 0}
        <h3 class="mkt-soon-title">{$t("mkt.soon")}</h3>
        <div class="mkt-grid">
          {#each soonApps as app (app.name)}
            <article class="mkt-card soon" aria-disabled="true">
              <div class="mkt-top">
                <span class="mkt-tile sh-squircle" style:background={app.color} aria-hidden="true">
                  <span class="glyph">{app.glyph}</span>
                </span>
                <h3>{app.name}</h3>
                <span class="soon-badge">{$t("mkt.soon")}</span>
              </div>
              <p class="mkt-desc">{app.desc}</p>
              <footer class="mkt-foot">
                <span>Minime</span>
                <span class="dot">·</span>
                <span>{$t(app.catKey)}</span>
              </footer>
            </article>
          {/each}
        </div>
      {/if}
    {/if}
  {:else}
    {#if skills.length === 0}
      <p class="mkt-empty">{$t("mkt.noResults")}</p>
    {:else}
      <div class="mkt-grid">
        {#each skills as s (s.id)}
          <article class="mkt-card">
            <div class="mkt-top">
              <span class="mkt-tile sh-squircle" style:background={s.source === "user" ? "#4FA6F0" : "#8b5cf6"} aria-hidden="true">
                <span class="glyph">{"</>"}</span>
              </span>
              <h3>{s.name}</h3>
              <button
                class="mkt-add"
                class:added={s.agents.length > 0}
                onclick={() => openPicker(s)}
              >
                {#if s.agents.length === 0}
                  {$t("mkt.install")}
                {:else if s.agents.length === 1}
                  ✓ {agentLabel(s.agents[0])} ∨
                {:else}
                  {$t("mkt.installedN", { n: s.agents.length })}
                {/if}
              </button>
            </div>
            <p class="mkt-desc">{s.description}</p>
            <footer class="mkt-foot">
              <span>Minime</span>
              <span class="dot">·</span>
              <span>{$t("mkt.footer.skillsN", { n: s.agents.length })}</span>
              <span class="dot">·</span>
              <span class="src">{$t(s.source === "user" ? "mkt.source.user" : "mkt.source.bundled")}</span>
            </footer>
          </article>
        {/each}
      </div>
    {/if}
  {/if}
</div>

{#if pickerSkill}
  {@const s = ($skillCatalog ?? []).find((x) => x.id === pickerSkill)}
  {#if s}
    <div class="mkt-backdrop" role="presentation" onclick={closePicker}></div>
    <div class="mkt-dialog" role="dialog" aria-label={s.name}>
      <h3>{$t("mkt.chooseAgents")}</h3>
      <p class="mkt-dlg-sub">{s.name}</p>
      <!-- Patria 20 Okt: hanya 6 agen utama Minime yang tampil di sini. -->
      <div class="mkt-agent-list">
        {#each seedList as a (a.id)}
          <label class="mkt-agent-row">
            <input
              type="checkbox"
              checked={pickerSelection.includes(a.id)}
              onchange={() => togglePickerAgent(a.id)}
            />
            <span>{$agentNames[a.id] ?? (a.nameKey ? $t(a.nameKey) : a.name)}</span>
          </label>
        {/each}
      </div>
      <div class="mkt-dlg-actions">
        <button class="btn-ghost" onclick={closePicker} disabled={saving}>{$t("mkt.cancel")}</button>
        <button class="btn-primary" onclick={savePicker} disabled={saving}>
          {saving ? $t("mkt.saving") : $t("mkt.save")}
        </button>
      </div>
    </div>
  {/if}
{/if}

{#if showCreate}
  <div class="mkt-backdrop" role="presentation" onclick={() => (showCreate = false)}></div>
  <div class="mkt-dialog" role="dialog" aria-label={$t("mkt.createTitle")}>
    <h3>{$t("mkt.createTitle")}</h3>
    <div class="field">
      <label for="mkt-new-name">{$t("mkt.createName")}</label>
      <input
        id="mkt-new-name"
        type="text"
        bind:value={newName}
        placeholder={$t("mkt.createNamePh")}
      />
    </div>
    <div class="field">
      <label for="mkt-new-desc">{$t("mkt.createDesc")}</label>
      <textarea
        id="mkt-new-desc"
        rows="3"
        bind:value={newDesc}
        placeholder={$t("mkt.createDescPh")}
      ></textarea>
    </div>
    {#if createError}
      <p class="mkt-error" role="alert">{createError}</p>
    {/if}
    <div class="mkt-dlg-actions">
      <button class="btn-ghost" onclick={() => (showCreate = false)}>{$t("mkt.cancel")}</button>
      <button class="btn-primary" onclick={createSkill} disabled={!newName.trim() || !newDesc.trim()}>
        {$t("mkt.create")}
      </button>
    </div>
  </div>
{/if}

<style>
  .mkt {
    padding: 22px 24px 32px;
    overflow-y: auto;
    height: 100%;
  }
  .mkt-head {
    display: flex;
    flex-wrap: wrap;
    gap: 14px;
    align-items: flex-end;
    justify-content: space-between;
    margin-bottom: 18px;
  }
  .mkt-titles h2 { margin: 0 0 4px; font-size: 20px; }
  .mkt-titles p { margin: 0; color: var(--dim); font-size: 13px; }
  .mkt-controls {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    align-items: center;
  }
  .seg {
    display: inline-flex;
    background: var(--panel-2);
    border: 1px solid var(--border);
    border-radius: 999px;
    padding: 3px;
  }
  .seg button {
    border: 0;
    background: transparent;
    color: var(--dim);
    padding: 7px 16px;
    border-radius: 999px;
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .seg button.active {
    background: var(--text);
    color: var(--bg);
  }
  .mkt-tools {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .mkt-search {
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 8px 12px;
    color: var(--text);
    font: inherit;
    font-size: 13px;
    min-width: 180px;
  }
  .mkt-filter {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 6px 10px;
    font-size: 13px;
    color: var(--dim);
  }
  .mkt-filter select {
    border: 0;
    background: transparent;
    color: var(--text);
    font: inherit;
    cursor: pointer;
  }
  .mkt-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 14px;
  }
  .mkt-card {
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 14px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
  }
  .mkt-card.soon { opacity: 0.55; }
  .mkt-top {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .mkt-top h3 {
    margin: 0;
    font-size: 15px;
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .mkt-tile {
    width: 40px;
    height: 40px;
    flex: none;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: #fff;
    position: relative;
  }
  .mkt-tile .glyph { font-weight: 800; font-size: 14px; }
  .mkt-tile .eye {
    width: 7px;
    height: 9px;
    background: #fff;
    border-radius: 50%;
    margin: 0 2.5px;
    position: relative;
  }
  .mkt-tile .eye::after {
    content: "";
    position: absolute;
    left: 50%;
    top: 4px;
    width: 3.5px;
    height: 5px;
    transform: translateX(-50%);
    background: #111;
    border-radius: 50%;
  }
  .sh-circle { border-radius: 50%; }
  .sh-squircle { border-radius: 12px; }
  .sh-blob { border-radius: 58% 42% 55% 45% / 48% 52% 48% 52%; }
  .sh-hex { clip-path: polygon(25% 5%, 75% 5%, 100% 50%, 75% 95%, 25% 95%, 0% 50%); }
  .mkt-add {
    flex: none;
    border: 1px solid var(--border);
    background: var(--panel-2);
    color: var(--text);
    border-radius: 10px;
    padding: 7px 14px;
    font: inherit;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }
  .mkt-add.added {
    color: var(--success);
    border-color: color-mix(in srgb, var(--success) 45%, transparent);
  }
  .mkt-add:disabled { cursor: default; opacity: 0.9; }
  .soon-badge {
    flex: none;
    font-size: 12px;
    color: var(--dim);
    border: 1px dashed var(--border);
    border-radius: 8px;
    padding: 5px 10px;
  }
  .mkt-desc {
    margin: 0;
    font-size: 13px;
    color: var(--dim);
    line-height: 1.5;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    min-height: 58px;
  }
  .mkt-foot {
    margin-top: auto;
    padding-top: 10px;
    border-top: 1px solid var(--border);
    display: flex;
    gap: 6px;
    align-items: center;
    font-size: 12px;
    color: var(--dim);
  }
  .mkt-foot .dot { opacity: 0.6; }
  .mkt-foot .src { margin-left: auto; }
  .mkt-empty { color: var(--dim); padding: 32px 0; text-align: center; }
  .mkt-soon-title { margin: 26px 0 14px; font-size: 15px; }
  .ra-form {
    display: flex;
    flex-direction: column;
    gap: 10px;
    border-top: 1px dashed var(--border);
    padding-top: 12px;
  }
  .field { display: flex; flex-direction: column; gap: 5px; }
  .field label { font-size: 12px; color: var(--dim); }
  .field input,
  .field textarea {
    background: var(--panel-2);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 9px 12px;
    color: var(--text);
    font: inherit;
    font-size: 13px;
  }
  .field textarea { resize: vertical; }
  .ra-actions { display: flex; gap: 10px; align-items: center; }
  .link { color: var(--accent); font-size: 13px; }
  .ok-note { color: var(--success); font-size: 12px; }
  .mkt-backdrop {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 0.55);
    z-index: 100;
  }
  .mkt-dialog {
    position: fixed;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    z-index: 101;
    width: min(460px, calc(100vw - 48px));
    max-height: min(560px, calc(100vh - 96px));
    overflow-y: auto;
    background: var(--panel-solid);
    border: 1px solid var(--border);
    border-radius: 16px;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .mkt-dialog h3 { margin: 0; font-size: 16px; }
  .mkt-dlg-sub { margin: 0; color: var(--dim); font-size: 13px; }
  .mkt-agent-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 240px;
    overflow-y: auto;
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 6px;
  }
  .mkt-agent-row {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 8px 10px;
    border-radius: 8px;
    font-size: 13px;
    cursor: pointer;
  }
  .mkt-agent-row:hover { background: var(--panel-2); }
  .mkt-dlg-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 4px;
  }
  .mkt-error { color: var(--danger); font-size: 13px; margin: 0; }
</style>
