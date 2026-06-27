<script>
	import { app, closeEditor, saveEditor } from '$lib/state.svelte.js';

	let title = $state('');
	let artist = $state('');
	let album = $state('');
	let year = $state('');

	$effect(() => {
		if (app.editorOpen && app.editorTarget) {
			title = app.editorTarget.title || '';
			artist = app.editorTarget.artist || '';
			album = app.editorTarget.album || '';
			year = '';
		}
	});

	function save() {
		saveEditor({ title, artist, album, date: year });
	}
</script>

{#if app.editorOpen}
	<div class="overlay" role="presentation" onclick={closeEditor}>
		<div class="panel" role="dialog" tabindex="-1" onclick={(e) => e.stopPropagation()}>
			<div class="heading">Edit metadata</div>

			<label class="field">
				<span>Title</span>
				<input bind:value={title} />
			</label>
			<label class="field">
				<span>Artist</span>
				<input bind:value={artist} />
			</label>
			<label class="field">
				<span>Album</span>
				<input bind:value={album} />
			</label>
			<label class="field">
				<span>Year</span>
				<input bind:value={year} />
			</label>

			<div class="buttons">
				<button class="cancel" onclick={closeEditor}>Cancel</button>
				<button class="save" onclick={save}>Save</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		font-family: 'Undefined', monospace;
	}

	.panel {
		width: 420px;
		max-width: calc(100vw - 40px);
		background: var(--sidebar-bg);
		border: 1px solid var(--line);
		padding: 20px;
		display: flex;
		flex-direction: column;
		gap: 12px;
		border-radius: 0;
	}

	.heading {
		font-size: 18px;
		font-weight: bold;
		color: var(--text);
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.field span {
		font-size: 12px;
		color: var(--text-dim);
	}

	input {
		background: var(--input-bg);
		border: 1px solid var(--line);
		padding: 8px;
		color: var(--text);
		width: 100%;
		font-family: inherit;
		font-size: 14px;
		border-radius: 0;
		outline: none;
	}

	input:focus {
		border-color: var(--accent);
	}

	.buttons {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 4px;
	}

	.cancel {
		color: var(--text-dim);
		background: transparent;
		padding: 8px 16px;
		font-size: 14px;
		border-radius: 0;
	}

	.cancel:hover {
		color: var(--text);
	}

	.save {
		background: var(--accent);
		color: #fff;
		padding: 8px 16px;
		font-size: 14px;
		border-radius: 0;
	}
</style>
