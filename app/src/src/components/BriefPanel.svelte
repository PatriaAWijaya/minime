<script lang="ts">
  import { agents } from "../lib/agents";
  import {
    activeFlowProjectId,
    getProjectBrief,
    initiateProject,
    projectBriefs,
    projectTodos,
    projects,
    setProjectBrief,
    addProjectTodo,
    removeProjectTodo,
    toggleProjectTodo,
  } from "../lib/store";
  import { t } from "../lib/i18n";

  const pid = $derived($activeFlowProjectId);
  const briefRec = $derived($projectBriefs[pid] ?? getProjectBrief(pid));
  const todos = $derived($projectTodos.filter((x) => x.projectId === pid));

  let briefText = $state("");
  let goalsText = $state("");
  let lastPid = $state("");
  let newTodo = $state("");
  let newTodoAgent = $state("mimi");
  let showInitPopup = $state(false);

  $effect(() => {
    if (pid !== lastPid) {
      lastPid = pid;
      const rec = getProjectBrief(pid);
      briefText = rec.brief;
      goalsText = rec.goals;
    }
  });

  function saveBrief() {
    setProjectBrief(pid, briefText, goalsText);
  }

  function submitTodo() {
    addProjectTodo(pid, newTodo, newTodoAgent);
    newTodo = "";
  }

  function doInitiate() {
    const assignments = todos.map((x) => ({ agentId: x.agentId, task: x.text }));
    initiateProject(pid, assignments);
    showInitPopup = false;
  }

  function agentName(id: string): string {
    return agents.find((a) => a.id === id)?.name ?? id;
  }
</script>

<div class="brief-panel">
  <div class="brief-head">
    <label>
      <span>{$t("tasks.brief.project")}</span>
      <select bind:value={$activeFlowProjectId}>
        {#each $projects as p (p.id)}
          <option value={p.id}>{p.name}</option>
        {/each}
      </select>
    </label>
    {#if briefRec.initiated}
      <span class="init-badge">{$t("tasks.brief.initiated")}</span>
    {/if}
  </div>

  <div class="field">
    <label for="brief-text">{$t("tasks.brief.brief")}</label>
    <textarea
      id="brief-text"
      rows="4"
      bind:value={briefText}
      placeholder={$t("tasks.brief.briefPlaceholder")}
      onblur={saveBrief}
    ></textarea>
  </div>

  <div class="field">
    <label for="brief-goals">{$t("tasks.brief.goals")}</label>
    <textarea
      id="brief-goals"
      rows="2"
      bind:value={goalsText}
      placeholder={$t("tasks.brief.goalsPlaceholder")}
      onblur={saveBrief}
    ></textarea>
  </div>

  <div class="field">
    <span class="field-label">{$t("tasks.brief.todos")}</span>
    <div class="todo-add">
      <input
        type="text"
        bind:value={newTodo}
        placeholder={$t("tasks.brief.todoPlaceholder")}
        onkeydown={(e) => { if (e.key === "Enter") submitTodo(); }}
      />
      <select bind:value={newTodoAgent} aria-label={$t("tasks.brief.agent")}>
        {#each agents as a (a.id)}
          <option value={a.id}>{a.name}</option>
        {/each}
      </select>
      <button class="btn-ghost" onclick={submitTodo} disabled={!newTodo.trim()}>
        {$t("tasks.brief.addTodo")}
      </button>
    </div>
    {#if todos.length === 0}
      <p class="empty-note">{$t("tasks.brief.noTodos")}</p>
    {:else}
      <ul class="todo-list">
        {#each todos as td (td.id)}
          <li>
            <label class="todo-item">
              <input
                type="checkbox"
                checked={td.done}
                onchange={() => toggleProjectTodo(td.id)}
              />
              <span class:done={td.done}>{td.text}</span>
            </label>
            <span class="todo-agent">{agentName(td.agentId)}</span>
            <button
              class="mini-btn danger"
              onclick={() => removeProjectTodo(td.id)}
              aria-label={$t("tasks.brief.removeTodo")}
            >×</button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>

  <div class="brief-actions">
    <button
      class="btn-primary"
      onclick={() => (showInitPopup = true)}
      disabled={!briefText.trim() || todos.length === 0}
    >
      {$t("tasks.brief.initiate")}
    </button>
    <p class="hint">{$t("tasks.brief.initiateHint")}</p>
  </div>
</div>

{#if showInitPopup}
  <div class="overlay" role="presentation" onclick={(e) => { if (e.target === e.currentTarget) showInitPopup = false; }}>
    <div class="dialog" role="dialog" aria-modal="true" aria-label={$t("tasks.brief.initiate")}>
      <h3>{$t("tasks.brief.initTitle")}</h3>
      <p>{$t("tasks.brief.initBody", { n: String(todos.length) })}</p>
      <div class="actions">
        <button class="btn-ghost" onclick={() => (showInitPopup = false)}>
          {$t("tasks.brief.cancel")}
        </button>
        <button class="btn-primary" onclick={doInitiate}>
          {$t("tasks.brief.initConfirm")}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .brief-panel {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 18px;
    max-width: 760px;
  }
  .brief-head {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .brief-head label {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 14px;
  }
  .init-badge {
    font-size: 12px;
    font-weight: 600;
    color: var(--accent);
    border: 1px solid var(--accent);
    border-radius: 12px;
    padding: 3px 10px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .field label, .field-label { font-size: 14px; font-weight: 600; }
  .field textarea, .todo-add input {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--panel-2);
    color: var(--text);
    font-size: 14px;
  }
  .todo-add {
    display: flex;
    gap: 8px;
  }
  .todo-add input { flex: 1; }
  .todo-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .todo-list li {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }
  .todo-item {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    font-size: 14px;
  }
  .todo-item .done { text-decoration: line-through; color: var(--dim); }
  .todo-agent {
    font-size: 12px;
    color: var(--dim);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 2px 8px;
  }
  .empty-note, .hint { font-size: 13px; color: var(--dim); }
  .brief-actions {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 50;
  }
  .dialog {
    background: var(--panel-solid);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 24px;
    max-width: 420px;
    width: 90%;
  }
  .dialog h3 { margin: 0 0 10px; font-size: 17px; }
  .dialog p { font-size: 14px; color: var(--dim); margin: 0 0 18px; }
  .dialog .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }
</style>
