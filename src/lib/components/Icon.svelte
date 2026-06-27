<script>
	let { name, size = 16 } = $props();

	const ICONS = {
		folder: '<path d="M4 4h6v2H4zm0 14h16v2H4zM20 8h2v10h-2zM2 6h2v12H2zm8 0h10v2H10z"/>',
		music:
			'<path d="M4 12h4v2H4zm-2 2h2v4H2zm2 4h4v2H4zM8 6h2v12H8zm10 0h2v12h-2zm-6 8h2v4h-2zm2-2h4v2h-4zm0 6h4v2h-4zM10 4h8v2h-8z"/>',
		heart:
			'<path d="M13 22h-2v-2h2zm-2-2H9v-2h2zm4 0h-2v-2h2zm-6-2H7v-2h2zm8 0h-2v-2h2zM7 16H5v-2h2zm12 0h-2v-2h2zM5 14H3v-2h2zm16 0h-2v-2h2zM3 12H1V6h2zm20 0h-2V6h2zM13 8h-2V6h2zM5 6H3V4h2zm6 0H9V4h2zm4 0h-2V4h2zm6 0h-2V4h2zM9 4H5V2h4zm10 0h-4V2h4z"/>',
		sliders:
			'<path d="M4 4h6v3h12v2H10v3H4V9H2V7h2zm2 2v4h2V6zm8 6h6v3h2v2h-2v3h-6v-3H2v-2h12zm2 2v4h2v-4z"/>',
		users:
			'<path d="M5 2h6v2H5zm10 0h4v2h-4zM5 10h6v2H5zm10 0h4v2h-4zm4-6h2v6h-2zm-8 0h2v6h-2zM3 4h2v6H3zM0 18h2v4H0zm14 0h2v4h-2zm8 0h2v4h-2zM4 14h8v2H4zm12 0h4v2h-4zM2 16h2v2H2zm10 0h2v2h-2zm8 0h2v2h-2z"/>',
		album:
			'<path d="M4 2h16v2H4zm2 5h2v2H6zm4 0h8v2h-8zm-4 4h2v2H6zm4 0h8v2h-8zm-4 4h2v2H6zm4 0h8v2h-8zm-6 5h16v2H4zM2 4h2v16H2zm18 0h2v16h-2z"/>',
		list: '<path d="M6 6H4v2h2zm14 0H8v2h12zM4 11h2v2H4zm16 0H8v2h12zM4 16h2v2H4zm16 0H8v2h12z"/>',
		play: '<path d="M15 11h-2V9h2zm0 4h-2v-2h2zm-2 2h-2v-2h2zm0-8h-2V7h2zm-2-2H9V5h2zM9 21H7V3h2zm6-8h2v-2h-2zm-6 4h2v2H9z"/>',
		pause: '<path d="M10 4H5v16h5zm9 0h-5v16h5z"/>',
		next: '<path d="M6 4h2v2h2v2h2v2h2v4h-2v2h-2v2H8v2H6zm12 0h-2v16h2z"/>',
		prev: '<path d="M6 4h2v16H6zm12 0h-2v2h-2v3h-2v2h-2v2h2v3h2v2h2v2h2z"/>',
		repeat:
			'<path d="M17 5h2v2h-2zM5 17h2v2H5zm6-14h2v6h-2zM9 1h2v8H9zm0 8h2v2H9zm10 8H9v2h10zM5 7H3v10h2z"/><path d="M13 15h-2v6h2zm2-2h-2v8h2zm0 8h-2v2h2zM5 5h10v2H5zm14 12h2V7h-2z"/>',
		search:
			'<path d="M22 22h-2v-2h2zm-2-2h-2v-2h2zm-6-2H6v-2h8zm4 0h-2v-2h2zM6 16H4v-2h2zm10 0h-2v-2h2zM4 14H2V6h2zm14 0h-2V6h2zM6 6H4V4h2zm10 0h-2V4h2zm-2-2H6V2h8z"/>',
		'chevron-left':
			'<path d="M8 13v-2h2v2zm2-2V9h2v2zm0 4v-2h2v2zm2-6V7h2v2zm0 8v-2h2v2zm2-10V5h2v2zm0 12v-2h2v2z"/>',
		'chevron-right':
			'<path d="M16 13v-2h-2v2zm-2-2V9h-2v2zm0 4v-2h-2v2zm-2-6V7h-2v2zm0 8v-2h-2v2zM10 7V5H8v2zm0 12v-2H8v2z"/>'
	};
</script>

<svg
	class="icon"
	width={size}
	height={size}
	viewBox="0 0 24 24"
	fill="currentColor"
	aria-hidden="true">{@html ICONS[name] || ''}</svg>

<style>
	.icon {
		display: inline-block;
		flex-shrink: 0;
		vertical-align: -0.15em;
		shape-rendering: crispEdges;
	}
</style>
