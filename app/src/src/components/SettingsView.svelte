<script lang="ts">
  import { agents } from "../lib/agents";
  import { agentAiChoice } from "../lib/store";
  import { lang, t, type Lang } from "../lib/i18n";

  const langs: { id: Lang; label: string }[] = [
    { id: "id", label: "Indonesia" },
    { id: "en", label: "English" },
    { id: "zh", label: "中文" },
  ];

  const aiOptions = ["ai.gemini", "ai.Muse", "ai.local"];

  function setAi(agentId: string, key: string) {
    agentAiChoice.update((m) => ({ ...m, [agentId]: key }));
  }
</script>

<section class="view">
  <h2 class="title">{$t("settings.title")}</h2>

  <div class="block">
    <div class="field">
      <span class="group-label" id="lang-label">{$t("settings.language")}</span>
      <div class="lang-row" role="group" aria-labelledby="lang-label">
        {#each langs as l}
          <button
            class="btn-ghost lang-btn"
            class:active={$lang === l.id}
            onclick={() => lang.set(l.id)}
          >{l.label}</button>
        {/each}
      </div>
    </div>
  </div>

  <div class="block">
    <h3>{$t("settings.aiPerAgent")}</h3>
    {#each agents as agent}
      {@const choice = $agentAiChoice[agent.id] ?? (agent.id === "mimi" ? "ai.Muse" : "ai.gemini")}
      <div class="field agent-row">
        <label for={`ai-${agent.id}`}>
          <span class="badge" style:background={agent.color}>{agent.badge}</span>
          {agent.name}
        </label>
        <select id={`ai-${agent.id}`} value={choice} onchange={(e) => setAi(agent.id, e.currentTarget.value)}>
          {#each aiOptions as opt}
            <option value={opt} selected={choice === opt}>{$t(opt)}</option>
          {/each}
        </select>
      </div>
    {/each}
  </div>

  <!-- Advanced options stay hidden behind a smart default: simple for beginners -->
  <details class="block advanced">
    <summary>{$t("settings.advanced")}</summary>
    <p class="note">{$t("settings.advancedNote")}</p>
  </details>

  <p class="note">{$t("settings.note")}</p>
</section>

<style>
  .view { padding: 24px; max-width: 640px; overflow-y: auto; }
  .title { font-size: 22px; margin-bottom: 20px; }
  .block {
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 18px;
    margin-bottom: 16px;
  }
  .block h3 { font-size: 16px; margin-bottom: 14px; }
  .field { margin-bottom: 14px; }
  .field:last-child { margin-bottom: 0; }
  .field label { display: flex; align-items: center; gap: 8px; }
  .group-label {
    display: block;
    font-size: 13px;
    font-weight: 600;
    color: var(--dim);
    margin-bottom: 6px;
  }
  .badge {
    width: 28px; height: 28px; border-radius: 50%;
    display: inline-flex; align-items: center; justify-content: center;
    color: #fff; font-size: 14px; font-weight: 800;
  }
  .lang-row { display: flex; gap: 8px; flex-wrap: wrap; }
  .lang-btn.active { border-color: var(--accent); color: var(--accent); }
  .advanced summary {
    cursor: pointer; font-weight: 700; font-size: 15px;
    min-height: 44px; display: flex; align-items: center;
  }
  .note { font-size: 14px; color: var(--dim); margin-top: 8px; }
</style>
