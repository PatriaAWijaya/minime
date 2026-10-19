<script lang="ts">
  import { addSkill, skills, templates } from "../lib/store";
  import {
    connectRuangapi,
    ruangapiConnected,
    ruangapiKey,
    ruangapiMcpUrl,
  } from "../lib/store";
  import { t } from "../lib/i18n";

  // Apps stay disabled until Phase 3; the note under each card says so.
  // Exception: RuangAPI has a mock key/MCP-URL form (Patria 19 Okt).
  const apps: { nameKey: string; descKey: string; badge: string; color: string; custom?: string }[] = [
    { nameKey: "tools.meta", descKey: "tools.metaDesc", badge: "M", color: "#2563eb" },
    { nameKey: "tools.postiz", descKey: "tools.postizDesc", badge: "P", color: "#7c3aed" },
    { nameKey: "tools.pireel", descKey: "tools.pireelDesc", badge: "P", color: "#e11d48" },
    { nameKey: "tools.drive", descKey: "tools.driveDesc", badge: "D", color: "#059669" },
    { nameKey: "tools.slack", descKey: "tools.slackDesc", badge: "S", color: "#d97706" },
    { nameKey: "tools.discord", descKey: "tools.discordDesc", badge: "D", color: "#5865F2" },
    { nameKey: "tools.telegram", descKey: "tools.telegramDesc", badge: "T", color: "#0891b2" },
    { nameKey: "tools.github", descKey: "tools.githubDesc", badge: "G", color: "#6b7280" },
    { nameKey: "tools.notion", descKey: "tools.notionDesc", badge: "N", color: "#a3a3a3" },
    { nameKey: "tools.canva", descKey: "tools.canvaDesc", badge: "C", color: "#0ea5e9" },
    { nameKey: "tools.ruangapi", descKey: "tools.ruangapiDesc", badge: "R", color: "#0ea5e9", custom: "ruangapi" },
  ];

  // RuangAPI credentials (mock, local only — real connection in Phase 3)
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

  let showDialog = $state(false);
  let skillName = $state("");

  function toggleTemplate(id: number) {
    templates.update((list) =>
      list.map((tpl) => (tpl.id === id ? { ...tpl, enabled: !tpl.enabled } : tpl)),
    );
  }

  function toggleSkill(id: number) {
    skills.update((list) =>
      list.map((s) => (s.id === id ? { ...s, installed: !s.installed } : s)),
    );
  }

  function createSkill() {
    addSkill(skillName);
    skillName = "";
    showDialog = false;
  }

  function skillNameOf(s: { nameKey?: string; customName?: string }): string {
    return s.customName ?? $t(s.nameKey ?? "");
  }
</script>

<div class="page">
  <div class="page-head">
    <h2>{$t("tools.title")}</h2>
  </div>

  <div class="section">
    <h3>{$t("tools.agents")}</h3>
    <p class="lede">{$t("tools.agents.desc")}</p>
    <div class="grid">
      {#each $templates as tpl (tpl.id)}
        <div class="card">
          <h4>{$t(tpl.nameKey)}</h4>
          <p>{$t(tpl.descKey)}</p>
          <div class="actions">
            <button class="btn-ghost" onclick={() => toggleTemplate(tpl.id)}>
              {tpl.enabled ? $t("tools.deactivate") : $t("tools.activate")}
            </button>
          </div>
        </div>
      {/each}
    </div>
  </div>

  <div class="section">
    <h3>{$t("tools.apps")}</h3>
    <p class="lede">{$t("tools.apps.desc")}</p>
    <div class="grid">
      {#each apps as app}
        <div class="card">
          <div class="top">
            <span class="mini-badge" style:background={app.color}>{app.badge}</span>
            <h4>{$t(app.nameKey)}</h4>
          </div>
          <p>{$t(app.descKey)}</p>
          {#if app.custom === "ruangapi"}
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
                <input
                  id="ra-mcp"
                  type="text"
                  bind:value={raMcpUrl}
                  inputmode="url"
                />
              </div>
              <div class="actions ra-actions">
                <button
                  class="btn-ghost"
                  onclick={connectRuangapiForm}
                  disabled={!raKey.trim()}
                >
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
          {:else}
            <div class="actions">
              <button class="btn-ghost" disabled title={$t("tools.soon")}>
                {$t("tools.connect")}
              </button>
            </div>
            <span class="soon-note">{$t("tools.soon")}</span>
          {/if}
        </div>
      {/each}
    </div>
  </div>

  <div class="section">
    <div class="page-head">
      <div>
        <h3>{$t("tools.skills")}</h3>
        <p class="lede">{$t("tools.skills.desc")}</p>
      </div>
      <button class="btn-primary" onclick={() => (showDialog = true)}>
        {$t("tools.newSkill")}
      </button>
    </div>
    <div class="grid">
      {#each $skills as skill (skill.id)}
        <div class="card">
          <h4>{skillNameOf(skill)}</h4>
          {#if skill.descKey}<p>{$t(skill.descKey)}</p>{/if}
          <div class="actions">
            <button class="btn-ghost" onclick={() => toggleSkill(skill.id)}>
              {skill.installed ? $t("tools.installed") : $t("tools.install")}
            </button>
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>

{#if showDialog}
  <div class="overlay" role="presentation" onclick={(e) => { if (e.target === e.currentTarget) showDialog = false; }}>
    <div class="dialog" role="dialog" aria-modal="true" aria-label={$t("tools.newSkill.title")}>
      <h3>{$t("tools.newSkill.title")}</h3>
      <div class="field">
        <label for="skill-name">{$t("tools.newSkill.prompt")}</label>
        <input
          id="skill-name"
          bind:value={skillName}
          placeholder={$t("tools.newSkill.placeholder")}
        />
      </div>
      <div class="actions">
        <button class="btn-ghost" onclick={() => (showDialog = false)}>
          {$t("tools.newSkill.cancel")}
        </button>
        <button class="btn-primary" onclick={createSkill} disabled={!skillName.trim()}>
          {$t("tools.newSkill.create")}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .page-head .btn-primary { flex: none; }
  .ra-form {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 4px;
  }
  .ra-form .field { margin: 0; }
  .ra-actions { align-items: center; }
  .ra-actions .link {
    font-size: 13px;
    color: var(--accent);
    text-decoration: none;
  }
  .ra-actions .link:hover { text-decoration: underline; }
  .ok-note {
    font-size: 13px;
    color: var(--accent);
  }
</style>
