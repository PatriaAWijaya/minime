<script lang="ts">
  import { onMount } from "svelte";
  import {
    customAgents,
    deleteCustomAgent,
    toggleAppInstalled,
    refreshTools,
    toolInfos,
    connectRuangapi,
    ruangapiConnected,
    ruangapiKey,
    ruangapiMcpUrl,
    agentNames,
    agentTitles,
    skillCatalog,
    refreshSkillCatalog,
    saveAgentSkills,
    createSkillFile,
    activeView,
    pireelStatus,
    pireelOAuth,
    pireelOAuthError,
    pireelAuthUrl,
    pireelConnect,
    pireelPollOAuth,
    pireelRefreshStatus,
    pireelDisconnect,
    driveStatus,
    driveOAuth,
    driveOAuthError,
    driveAuthUrl,
    driveClientId,
    driveConnect,
    drivePollOAuth,
    driveRefreshStatus,
    driveDisconnect,
    type SkillCatalogEntry,
  } from "../lib/store";
  import { agents as seedList, agentDisplayName, agentDisplayRole } from "../lib/agents";
  import { SOON_APPS, APP_TILES, APP_CAT_KEYS, appDescKey } from "../lib/marketplace";
  import { t } from "../lib/i18n";

  type Tab = "agents" | "apps" | "skills";
  let tab = $state<Tab>("agents");
  let query = $state("");
  let filter = $state("all");

  // Backend registries: tool connection status + real skill catalog.
  onMount(() => {
    void refreshTools();
    void refreshSkillCatalog();
    void pireelRefreshStatus();
    void driveRefreshStatus();
    const timer = setInterval(() => {
      if ($pireelOAuth === "pending") void pireelPollOAuth();
      if ($driveOAuth === "pending") void drivePollOAuth();
    }, 2000);
    return () => clearInterval(timer);
  });
  $effect(() => {
    // Load the skill catalog the first time the Skills tab opens.
    if (tab === "skills" && !$skillCatalog) void refreshSkillCatalog();
  });

  const allAgents = $derived([...seedList, ...$customAgents]);
  function agentLabel(id: string): string {
    const a = allAgents.find((x) => x.id === id);
    if (!a) return id;
    return agentDisplayName(a, $agentNames, $t);
  }

  /* ---------------- Agents tab: the 6 built-in Minime agents --------------
     Patria 20 Okt: no fictitious templates — only the real agents with
     their real skills from the backend catalog. Names/roles sync both ways
     with the Agents view via the shared helpers in lib/agents. */
  const agentSkillCounts = $derived.by(() => {
    const counts: Record<string, number> = {};
    for (const s of $skillCatalog ?? []) {
      for (const id of s.agents) counts[id] = (counts[id] ?? 0) + 1;
    }
    return counts;
  });
  const agentName = (a: { id: string; name: string; nameKey?: string }): string =>
    agentDisplayName(a, $agentNames, $t);
  const agentRole = (a: { id: string; roleKey: string }): string =>
    agentDisplayRole(a, $agentTitles, $t);
  /* Patria 20 Okt: tab Agen + dialog skill menampilkan SEMUA agen nyata —
     6 bawaan + agen custom buatan user ("Agen baru"). Tanpa template fiktif. */
  const toolsAgents = $derived([...seedList, ...$customAgents]);
  const filteredAgents = $derived(
    toolsAgents
      .filter(
        (a) =>
          !query ||
          agentName(a).toLowerCase().includes(query.toLowerCase()) ||
          agentRole(a).toLowerCase().includes(query.toLowerCase()),
      )
      .sort((a, b) =>
        filter === "za"
          ? agentName(b).localeCompare(agentName(a))
          : agentName(a).localeCompare(agentName(b)),
      ),
  );
  function openAgent() {
    activeView.set("agents");
  }

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

  // Permanent delete of a custom agent (Patria 20 Okt): ONLY from Tools;
  // the 6 built-in Minime agents have no delete button (deactivate only).
  const customIds = $derived(new Set($customAgents.map((a) => a.id)));
  let delAgentId = $state<string | null>(null);
  let deleting = $state(false);
  async function confirmDeleteAgent() {
    if (!delAgentId || deleting) return;
    deleting = true;
    await deleteCustomAgent(delAgentId);
    delAgentId = null;
    deleting = false;
  }
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
        {#each filteredAgents as a (a.id)}
          <article class="mkt-card">
            <div class="mkt-top">
              <span class="mkt-tile sh-squircle" style:background={a.color} aria-hidden="true">
                <span class="glyph">{a.badge}</span>
              </span>
              <h3>{agentName(a)}</h3>
              {#if customIds.has(a.id)}
                <button
                  class="mkt-del"
                  onclick={() => (delAgentId = a.id)}
                  aria-label={$t("mkt.deleteAgent")}
                  title={$t("mkt.deleteAgent")}
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                    <path d="M3 6h18M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2m3 0-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
                  </svg>
                </button>
              {/if}
              <button class="mkt-add" onclick={openAgent}>
                {$t("mkt.open")}
              </button>
            </div>
            <p class="mkt-desc">{agentRole(a)}</p>
            <footer class="mkt-foot">
              <span>Minime</span>
              <span class="dot">·</span>
              <span>{$t("mkt.footer.skillsN", { n: agentSkillCounts[a.id] ?? 0 })}</span>
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
              {#if app.id === "pireel"}
                {#if $pireelStatus?.connected}
                  <span class="mkt-add added" role="status">✓ {$t("mkt.connected")}</span>
                {:else if $pireelOAuth === "pending"}
                  <button class="mkt-add" disabled>…</button>
                {:else}
                  <button class="mkt-add" onclick={pireelConnect}>
                    {$t("mkt.pireel.connect")}
                  </button>
                {/if}
              {:else if app.id === "drive"}
                {#if $driveStatus?.connected}
                  <span class="mkt-add added" role="status">✓ {$t("mkt.connected")}</span>
                {:else if $driveOAuth === "pending"}
                  <button class="mkt-add" disabled>…</button>
                {:else}
                  <button class="mkt-add" onclick={driveConnect}>
                    {$t("mkt.drive.connect")}
                  </button>
                {/if}
              {:else}
              <button
                class="mkt-add"
                class:added={app.connected}
                onclick={() => toggleAppInstalled(app.id)}
              >
                {#if app.connected}
                  ✓ {$t("mkt.connected")}
                {:else}
                  {$t("mkt.connect")}
                {/if}
              </button>
              {/if}
            </div>
            <p class="mkt-desc">{$t(appDescKey(app.id))}</p>
            {#if app.id === "pireel"}
              <div class="pireel-box">
                {#if $pireelStatus?.connected}
                  <span class="ok-note" role="status">
                    {$t(`mkt.pireel.status.${$pireelStatus.detail}`)}
                  </span>
                  <div class="ra-actions">
                    <button class="btn-ghost" onclick={pireelDisconnect}>
                      {$t("mkt.pireel.disconnect")}
                    </button>
                  </div>
                {:else if $pireelOAuth === "pending"}
                  <span class="warn-note" role="status">{$t("mkt.pireel.waiting")}</span>
                  <div class="ra-actions">
                    {#if $pireelAuthUrl}
                      <a class="link" href={$pireelAuthUrl} target="_blank" rel="noopener">
                        {$t("mkt.pireel.openAgain")}
                      </a>
                    {/if}
                    <button class="btn-ghost" onclick={() => pireelOAuth.set("idle")}>
                      {$t("mkt.pireel.cancel")}
                    </button>
                  </div>
                {:else if $pireelOAuthError}
                  <span class="err-note" role="alert">{$pireelOAuthError}</span>
                  <div class="ra-actions">
                    <button class="btn-ghost" onclick={pireelConnect}>
                      {$t("mkt.pireel.retry")}
                    </button>
                  </div>
                {/if}
                <p class="pireel-note">{$t("mkt.pireel.freeNote")}</p>
              </div>
            {/if}
            {#if app.id === "drive"}
              <div class="pireel-box">
                {#if $driveStatus?.connected}
                  <span class="ok-note" role="status">
                    {$t(`mkt.drive.status.${$driveStatus.detail}`)}{#if $driveStatus.email} — {$driveStatus.email}{/if}
                  </span>
                  <div class="ra-actions">
                    <button class="btn-ghost" onclick={driveDisconnect}>
                      {$t("mkt.drive.disconnect")}
                    </button>
                  </div>
                {:else if $driveOAuth === "pending"}
                  <span class="warn-note" role="status">{$t("mkt.drive.waiting")}</span>
                  <div class="ra-actions">
                    {#if $driveAuthUrl}
                      <a class="link" href={$driveAuthUrl} target="_blank" rel="noopener">
                        {$t("mkt.drive.openAgain")}
                      </a>
                    {/if}
                    <button class="btn-ghost" onclick={() => driveOAuth.set("idle")}>
                      {$t("mkt.drive.cancel")}
                    </button>
                  </div>
                {:else}
                  {#if $driveOAuthError}
                    <span class="err-note" role="alert">{$driveOAuthError}</span>
                  {/if}
                  <div class="field">
                    <label for="drive-client-id">{$t("mkt.drive.clientId")}</label>
                    <input
                      id="drive-client-id"
                      type="text"
                      bind:value={$driveClientId}
                      placeholder={$t("mkt.drive.clientIdPh")}
                      autocomplete="off"
                      spellcheck={false}
                    />
                  </div>
                  <a
                    class="link"
                    href="https://console.cloud.google.com/apis/credentials"
                    target="_blank"
                    rel="noopener"
                  >
                    {$t("mkt.drive.guide")}
                  </a>
                {/if}
                <p class="pireel-note">{$t("mkt.drive.note")}</p>
              </div>
            {/if}
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
      <!-- Patria 20 Okt: semua agen nyata (6 bawaan + custom user). -->
      <div class="mkt-agent-list">
        {#each toolsAgents as a (a.id)}
          <label class="mkt-agent-row">
            <input
              type="checkbox"
              checked={pickerSelection.includes(a.id)}
              onchange={() => togglePickerAgent(a.id)}
            />
            <span>{agentName(a)}</span>
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

{#if delAgentId}
  {@const da = toolsAgents.find((x) => x.id === delAgentId)}
  <div class="mkt-backdrop" role="presentation" onclick={() => (delAgentId = null)}></div>
  <div
    class="mkt-dialog"
    role="alertdialog"
    aria-modal="true"
    aria-labelledby="delagent-title"
    aria-describedby="delagent-desc"
  >
    <h3 id="delagent-title">{$t("mkt.delAgentTitle")}</h3>
    <p class="mkt-dlg-sub" id="delagent-desc">
      {$t("mkt.delAgentMsg", { name: da ? agentName(da) : delAgentId })}
    </p>
    <div class="mkt-dlg-actions">
      <button class="btn-ghost" onclick={() => (delAgentId = null)} disabled={deleting}>
        {$t("mkt.cancel")}
      </button>
      <button class="mkt-btn-del" onclick={confirmDeleteAgent} disabled={deleting}>
        {deleting ? $t("mkt.deleting") : $t("mkt.delete")}
      </button>
    </div>
  </div>
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
  .mkt-del {
    flex: none;
    width: 34px;
    height: 34px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--dim);
    border-radius: 10px;
    cursor: pointer;
  }
  .mkt-del:hover {
    color: #dc2626;
    border-color: #dc2626;
  }
  .mkt-del svg { width: 16px; height: 16px; }
  .mkt-btn-del {
    flex: 1;
    min-height: 44px;
    border: 0;
    border-radius: 10px;
    background: #dc2626;
    color: #fff;
    font-size: 14px;
    font-weight: 700;
    font-family: inherit;
    cursor: pointer;
  }
  .mkt-btn-del:hover { filter: brightness(1.1); }
  .mkt-btn-del:disabled { opacity: 0.6; cursor: default; }
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
  .warn-note { color: var(--warning, #f59e0b); font-size: 12px; }
  .err-note { color: var(--danger, #f87171); font-size: 12px; }
  .pireel-box {
    display: flex;
    flex-direction: column;
    gap: 10px;
    border-top: 1px dashed var(--border);
    padding-top: 12px;
  }
  .pireel-note { margin: 0; font-size: 12px; color: var(--dim); }
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
