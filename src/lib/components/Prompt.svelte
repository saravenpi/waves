<script>
	import { app, submitPrompt, cancelPrompt } from '$lib/state.svelte.js';

	let value = $state('');
	let inputEl = $state(null);

	$effect(() => {
		if (app.prompt) {
			value = app.prompt.value || '';
			if (inputEl) inputEl.focus();
		}
	});

	function onKey(e) {
		if (e.key === 'Enter') {
			e.preventDefault();
			submitPrompt(value);
		} else if (e.key === 'Escape') {
			e.preventDefault();
			cancelPrompt();
		}
		e.stopPropagation();
	}
</script>

{#if app.prompt}
	<div class="backdrop" onclick={cancelPrompt} role="presentation">
		<div class="panel" onclick={(e) => e.stopPropagation()} role="presentation">
			<div class="title">{app.prompt.kind === 'rename' ? 'Rename' : 'New folder'}</div>
			<input bind:this={inputEl} bind:value onkeydown={onKey} spellcheck="false" />
			<div class="actions">
				<button class="cancel" onclick={cancelPrompt}>Cancel</button>
				<button class="ok" onclick={() => submitPrompt(value)}>OK</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
	}
	.panel {
		background: var(--sidebar-bg);
		border: 1px solid var(--line);
		padding: 20px;
		width: 380px;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.title {
		font-size: 18px;
		font-weight: bold;
	}
	input {
		background: var(--input-bg);
		border: 1px solid var(--line);
		padding: 9px;
		color: var(--text);
		font-size: 14px;
		outline: none;
	}
	input:focus {
		border-color: var(--accent);
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 10px;
	}
	.actions button {
		padding: 8px 16px;
		font-size: 14px;
	}
	.cancel {
		color: var(--text-dim);
		border: 1px solid var(--line);
	}
	.ok {
		background: var(--accent);
		color: #fff;
	}
</style>
