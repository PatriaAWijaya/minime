<script lang="ts">
  import { agents, getAgent } from "../lib/agents";
  import {
    activeAgentId,
    agentAiChoice,
    agentFacts,
    agentInstructions,
    agentNames,
    agentNotify,
    agentReasoning,
    agentStatus,
  } from "../lib/store";
  import { t } from "../lib/i18n";

  const aiOptions = ["ai.gemini", "ai.Muse", "ai.local"];
  const reasoningOptions = ["low", "mid", "high"] as const;

  const agent = $derived(getAgent($activeAgentId));
  const facts = $derived(agentFacts[agent.id] ?? agentFacts.mimi);
  const aiChoice = $derived(
    $agentAiChoice[agent.id] ?? (agent.id === "mimi" ? "ai.Muse" : "ai.gemini"),
  );
  const reasoning = $derived($agentReasoning[agent.id] ?? "mid");
  const notify = $derived($agentNotify[agent.id] ?? true);
  const displayName = $derived($agentNames[agent.id] ?? agent.name);

  function setAi(value: string) {
    agentAiChoice.update((m) => ({ ...m, [agent.id]: value }));
  }
  function setReasoning(value: "low" | "mid" | "high") {
    agentReasoning.update((m) => ({ ...m, [agent.id]: value }));
  }
  function setNotify(value: boolean) {
    agentNotify.update((m) => ({ ...m, [agent.id]: value }));
  }
</script>

<div class="split">
  <div class="list-col">
    <h2>{$t("agents.title")}</h2>
    {#each agents as a}
      {@const status = $agentStatus[a.id] ?? "idle"}
      <button
        class="agent-row"
        class:active={$activeAgentId === a.id}
        onclick={() => activeAgentId.set(a.id)}
      >
        <span class="badge" style:background={a.color}>{a.badge}</span>
        <span class="meta">
          <span class="name">{a.name}</span>
          <span class="role">{$t(a.roleKey)}</span>
        </span>
        <span class="dot {status}" aria-hidden="true"></span>
      </button>
    {/each}
  </div>

  <div class="detail">
    <div class="detail-head">
      <span class="badge big" style:background={agent.color}>{agent.badge}</span>
      <div class="who">
        <input
          class="name-input"
          value={displayName}
          oninput={(e) => agentNames.update((m) => ({ ...m, [agent.id]: e.currentTarget.value }))}
          aria-label={$t("agents.name")}
        />
        <div class="role">{$t(agent.roleKey)}</div>
      </div>
    </div>

    <div class="field">
      <label for="instructions">{$t("agents.instructions")}</label>
      <textarea
        id="instructions"
        rows="2"
        placeholder={$t("agents.instructions.text")}
        value={$agentInstructions[agent.id] ?? ""}
        oninput={(e) =>
          agentInstructions.update((m) => ({ ...m, [agent.id]: e.currentTarget.value }))}
      ></textarea>
    </div>

    <div class="section">
      <h3>{$t("agents.brain")}</h3>
      <div class="field">
        <label for="model">{$t("agents.model")}</label>
        <select id="model" value={aiChoice} onchange={(e) => setAi(e.currentTarget.value)}>
          {#each aiOptions as opt}
            <option value={opt} selected={aiChoice === opt}>{$t(opt)}</option>
          {/each}
        </select>
      </div>
      <div class="row">
        <span class="label">{$t("agents.reasoning")}</span>
        <span class="seg" role="group" aria-label={$t("agents.reasoning")}>
          {#each reasoningOptions as r}
            <button
              class:active={reasoning === r}
              onclick={() => setReasoning(r)}
            >{$t(`agents.reasoning.${r}`)}</button>
          {/each}
        </span>
      </div>
      <div class="row">
        <span class="label">{$t("agents.backup")}</span>
        <span class="value">{$t("agents.backup.ask")}</span>
      </div>
    </div>

    <div class="section">
      <h3>{$t("agents.knows")}</h3>
      <div class="row">
        <span class="label">{$t("agents.memories")}</span>
        <span class="value">{$t("agents.memories.n", { n: String(facts.memories) })}</span>
      </div>
      <div class="row">
        <span class="label">{$t("agents.skills")}</span>
        <span class="value">{$t("agents.skills.n", { n: String(facts.skills) })}</span>
      </div>
      <div class="row">
        <span class="label">{$t("agents.files")}</span>
        <span class="value">{facts.files}</span>
      </div>
      <div class="row">
        <span class="label">{$t("agents.tables")}</span>
        <span class="value">{$t("agents.tables.n", { n: String(facts.tables) })}</span>
      </div>
    </div>

    <div class="section">
      <h3>{$t("agents.does")}</h3>
      <div class="row">
        <span class="label">{$t("agents.routines")}</span>
        <span class="value">{$t("agents.routines.n", { n: String(facts.routines) })}</span>
      </div>
      <div class="row">
        <span class="label">{$t("agents.usage")}</span>
      </div>
      <div class="row">
        <span class="label">{$t("agents.publish")}</span>
      </div>
    </div>

    <div class="section">
      <h3>{$t("agents.rules")}</h3>
      <div class="row">
        <span class="label">{$t("agents.permissions")}</span>
        <span class="value">{$t("agents.permissions.full")}</span>
      </div>
      <div class="row">
        <span class="label">{$t("agents.notifications")}</span>
        <label class="switch">
          <input
            type="checkbox"
            checked={notify}
            onchange={(e) => setNotify(e.currentTarget.checked)}
            aria-label={$t("agents.notifications")}
          />
          <span class="track" aria-hidden="true"></span>
        </label>
      </div>
      <div class="row">
        <span class="label">{$t("agents.advanced")}</span>
      </div>
    </div>
  </div>
</div>
