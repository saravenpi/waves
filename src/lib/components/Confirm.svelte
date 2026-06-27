<script>
	import { app, confirmDelete, cancelConfirm } from '$lib/state.svelte.js';
</script>

{#if app.confirm}
	<div class="backdrop" onclick={cancelConfirm} role="presentation">
		<div class="panel" onclick={(e) => e.stopPropagation()} role="presentation">
			<div class="msg">{app.confirm.message}</div>
			<div class="hint">Enter to confirm · Esc to cancel</div>
			<div class="actions">
				<button class="cancel" onclick={cancelConfirm}>Cancel</button>
				<button class="ok" onclick={confirmDelete}>Move to Trash</button>
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
		gap: 12px;
	}
	.msg {
		font-size: 15px;
		color: #c8c8c8;
	}
	.hint {
		font-size: 12px;
		color: var(--text-muted);
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
