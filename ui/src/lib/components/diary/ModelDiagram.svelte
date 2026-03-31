<script lang="ts">
	let { model }: { model: 'random-forest' | 'hybrid-cnn' | 'pure-cnn' } = $props();
</script>

{#if model === 'random-forest'}
<svg class="diagram" viewBox="0 0 860 320" xmlns="http://www.w3.org/2000/svg">
	<!-- Input: Audio -->
	<rect x="10" y="110" width="110" height="60" rx="4" fill="none" stroke="#33ddff" stroke-width="1.5" />
	<text x="65" y="137" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="12" font-weight="600">Audio Input</text>
	<text x="65" y="155" text-anchor="middle" fill="#555577" font-family="monospace" font-size="10">24,000 samples</text>

	<!-- Arrow 1 -->
	<line x1="120" y1="140" x2="150" y2="140" stroke="#555577" stroke-width="1" marker-end="url(#arrowRF)" />

	<!-- Mel-Spectrogram -->
	<rect x="150" y="95" width="120" height="90" rx="4" fill="#151428" stroke="#2a2848" stroke-width="1" />
	<text x="210" y="125" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="11" font-weight="600">Mel-Spectrogram</text>
	<!-- Mini grid to suggest 2D image -->
	<g opacity="0.4">
		{#each Array(6) as _, row}
			{#each Array(9) as _, col}
				<rect x={170 + col * 8} y={138 + row * 6} width="6" height="4" rx="0.5"
					fill={row < 2 ? '#33ddff' : row < 4 ? '#555577' : '#2a2848'} />
			{/each}
		{/each}
	</g>
	<text x="210" y="180" text-anchor="middle" fill="#555577" font-family="monospace" font-size="10">64 x 94</text>

	<!-- Arrow 2 -->
	<line x1="270" y1="140" x2="300" y2="140" stroke="#555577" stroke-width="1" marker-end="url(#arrowRF)" />

	<!-- Flatten -->
	<rect x="300" y="105" width="100" height="70" rx="4" fill="#151428" stroke="#2a2848" stroke-width="1" />
	<text x="350" y="130" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="11" font-weight="600">Flatten</text>
	<!-- Visual: 2D to 1D strip -->
	<rect x="315" y="140" width="70" height="6" rx="2" fill="#33ddff" opacity="0.3" />
	<text x="350" y="165" text-anchor="middle" fill="#555577" font-family="monospace" font-size="10">6,016 numbers</text>

	<!-- Arrow 3 -->
	<line x1="400" y1="140" x2="430" y2="140" stroke="#555577" stroke-width="1" marker-end="url(#arrowRF)" />

	<!-- Decision Trees fan -->
	<rect x="430" y="50" width="180" height="180" rx="6" fill="#151428" stroke="#ff3388" stroke-width="1.5" />
	<text x="520" y="75" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="11" font-weight="600">200 Decision Trees</text>
	<!-- Tree icons (simplified) -->
	{#each Array(5) as _, i}
		<g transform="translate({455 + i * 32}, 90)">
			<!-- Trunk -->
			<line x1="12" y1="40" x2="12" y2="55" stroke="#555577" stroke-width="1.5" />
			<!-- Branches -->
			<line x1="12" y1="40" x2="4" y2="28" stroke="#555577" stroke-width="1" />
			<line x1="12" y1="40" x2="20" y2="28" stroke="#555577" stroke-width="1" />
			<line x1="4" y1="28" x2="0" y2="18" stroke="#555577" stroke-width="0.8" />
			<line x1="4" y1="28" x2="8" y2="18" stroke="#555577" stroke-width="0.8" />
			<line x1="20" y1="28" x2="16" y2="18" stroke="#555577" stroke-width="0.8" />
			<line x1="20" y1="28" x2="24" y2="18" stroke="#555577" stroke-width="0.8" />
			<!-- Leaves -->
			<circle cx="0" cy="16" r="3" fill="#33ddff" opacity="0.3" />
			<circle cx="8" cy="16" r="3" fill="#ff3388" opacity="0.3" />
			<circle cx="16" cy="16" r="3" fill="#00ccaa" opacity="0.3" />
			<circle cx="24" cy="16" r="3" fill="#33ddff" opacity="0.3" />
		</g>
	{/each}
	<!-- "..." to suggest more trees -->
	<text x="520" y="170" text-anchor="middle" fill="#555577" font-family="monospace" font-size="14">... x 200</text>
	<!-- Converging lines to vote -->
	{#each Array(5) as _, i}
		<line x1={467 + i * 32} y1="185" x2="520" y2="210" stroke="#555577" stroke-width="0.6" opacity="0.5" />
	{/each}
	<text x="520" y="220" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="10" font-weight="600">Majority Vote</text>

	<!-- Arrow 4 -->
	<line x1="610" y1="140" x2="640" y2="140" stroke="#555577" stroke-width="1" marker-end="url(#arrowRF)" />

	<!-- Output -->
	<rect x="640" y="110" width="130" height="60" rx="4" fill="none" stroke="#00ccaa" stroke-width="1.5" />
	<text x="705" y="137" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="12" font-weight="600">Predicted Class</text>
	<text x="705" y="155" text-anchor="middle" fill="#555577" font-family="monospace" font-size="10">1 of 138</text>

	<!-- Arrow marker -->
	<defs>
		<marker id="arrowRF" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
			<path d="M0,0 L8,3 L0,6" fill="none" stroke="#555577" stroke-width="1" />
		</marker>
	</defs>
</svg>

{:else if model === 'hybrid-cnn'}
<svg class="diagram" viewBox="0 0 880 300" xmlns="http://www.w3.org/2000/svg">
	<defs>
		<marker id="arrowHC" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
			<path d="M0,0 L8,3 L0,6" fill="none" stroke="#555577" stroke-width="1" />
		</marker>
	</defs>

	<!-- Input Spectrogram -->
	<rect x="10" y="90" width="90" height="90" rx="4" fill="none" stroke="#33ddff" stroke-width="1.5" />
	<text x="55" y="110" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="10" font-weight="600">Mel-Spec</text>
	<!-- Mini spectrogram visual -->
	<g opacity="0.5">
		{#each Array(6) as _, row}
			{#each Array(7) as _, col}
				<rect x={20 + col * 9} y={118 + row * 7} width="7" height="5" rx="0.5"
					fill={row < 2 ? '#33ddff' : row < 4 ? '#555577' : '#2a2848'} />
			{/each}
		{/each}
	</g>
	<text x="55" y="178" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">1 x 64 x 94</text>

	<!-- Arrow -->
	<line x1="100" y1="135" x2="120" y2="135" stroke="#555577" stroke-width="1" marker-end="url(#arrowHC)" />

	<!-- Conv Block 1 -->
	<rect x="120" y="95" width="80" height="80" rx="4" fill="#151428" stroke="#2a2848" stroke-width="1" />
	<text x="160" y="117" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="10" font-weight="600">Conv 16</text>
	<text x="160" y="132" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">BN + ReLU</text>
	<text x="160" y="148" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="9">MaxPool</text>
	<text x="160" y="167" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">32 x 47</text>

	<line x1="200" y1="135" x2="220" y2="135" stroke="#555577" stroke-width="1" marker-end="url(#arrowHC)" />

	<!-- Conv Block 2 -->
	<rect x="220" y="100" width="80" height="70" rx="4" fill="#151428" stroke="#2a2848" stroke-width="1" />
	<text x="260" y="120" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="10" font-weight="600">Conv 32</text>
	<text x="260" y="135" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">BN + ReLU</text>
	<text x="260" y="150" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="9">MaxPool</text>
	<text x="260" y="165" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">16 x 23</text>

	<line x1="300" y1="135" x2="320" y2="135" stroke="#555577" stroke-width="1" marker-end="url(#arrowHC)" />

	<!-- Conv Block 3 -->
	<rect x="320" y="105" width="80" height="60" rx="4" fill="#151428" stroke="#2a2848" stroke-width="1" />
	<text x="360" y="124" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="10" font-weight="600">Conv 64</text>
	<text x="360" y="139" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">BN + ReLU</text>
	<text x="360" y="153" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="9">AdaptPool</text>
	<text x="360" y="163" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">4 x 4</text>

	<line x1="400" y1="135" x2="420" y2="135" stroke="#555577" stroke-width="1" marker-end="url(#arrowHC)" />

	<!-- Flatten -- highlighted as bottleneck -->
	<rect x="420" y="105" width="80" height="60" rx="4" fill="#151428" stroke="#ff3388" stroke-width="1.5" />
	<text x="460" y="127" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="10" font-weight="600">Flatten</text>
	<text x="460" y="145" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">1,024</text>
	<text x="460" y="160" text-anchor="middle" fill="#ff3388" font-family="system-ui, sans-serif" font-size="8">bottleneck</text>

	<line x1="500" y1="135" x2="520" y2="135" stroke="#555577" stroke-width="1" marker-end="url(#arrowHC)" />

	<!-- Dense 256 -->
	<rect x="520" y="105" width="80" height="60" rx="4" fill="#151428" stroke="#ff3388" stroke-width="1.5" />
	<text x="560" y="125" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="10" font-weight="600">Dense 256</text>
	<text x="560" y="140" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">ReLU</text>
	<text x="560" y="155" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">Dropout 0.3</text>

	<line x1="600" y1="135" x2="620" y2="135" stroke="#555577" stroke-width="1" marker-end="url(#arrowHC)" />

	<!-- Dense 138 output -->
	<rect x="620" y="110" width="90" height="50" rx="4" fill="none" stroke="#00ccaa" stroke-width="1.5" />
	<text x="665" y="133" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="10" font-weight="600">Dense 138</text>
	<text x="665" y="150" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">output</text>

	<!-- Annotation: parameter count in bottleneck region -->
	<rect x="420" y="185" width="180" height="28" rx="3" fill="#151428" stroke="#ff3388" stroke-width="0.5" opacity="0.8" />
	<text x="510" y="203" text-anchor="middle" fill="#ff3388" font-family="system-ui, sans-serif" font-size="9">~265K params in FC layers (overfitting risk)</text>

	<!-- Bracket under bottleneck -->
	<line x1="420" y1="170" x2="420" y2="185" stroke="#ff3388" stroke-width="0.5" stroke-dasharray="3,2" />
	<line x1="600" y1="170" x2="600" y2="185" stroke="#ff3388" stroke-width="0.5" stroke-dasharray="3,2" />

	<!-- Shrinking visual hint: decreasing bars on top -->
	<text x="55" y="82" text-anchor="middle" fill="#555577" font-family="monospace" font-size="8">64x94</text>
	<text x="160" y="88" text-anchor="middle" fill="#555577" font-family="monospace" font-size="8">32x47</text>
	<text x="260" y="93" text-anchor="middle" fill="#555577" font-family="monospace" font-size="8">16x23</text>
	<text x="360" y="98" text-anchor="middle" fill="#555577" font-family="monospace" font-size="8">4x4</text>
</svg>

{:else if model === 'pure-cnn'}
<svg class="diagram" viewBox="0 0 920 330" xmlns="http://www.w3.org/2000/svg">
	<defs>
		<marker id="arrowPC" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
			<path d="M0,0 L8,3 L0,6" fill="none" stroke="#555577" stroke-width="1" />
		</marker>
	</defs>

	<!-- Input Spectrogram -->
	<rect x="10" y="100" width="80" height="80" rx="4" fill="none" stroke="#33ddff" stroke-width="1.5" />
	<text x="50" y="122" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="10" font-weight="600">Mel-Spec</text>
	<g opacity="0.5">
		{#each Array(5) as _, row}
			{#each Array(6) as _, col}
				<rect x={18 + col * 9} y={130 + row * 7} width="7" height="5" rx="0.5"
					fill={row < 2 ? '#33ddff' : row < 3 ? '#555577' : '#2a2848'} />
			{/each}
		{/each}
	</g>
	<text x="50" y="178" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">1 x 64 x 94</text>

	<line x1="90" y1="140" x2="110" y2="140" stroke="#555577" stroke-width="1" marker-end="url(#arrowPC)" />

	<!-- Conv Block 1 -->
	<rect x="110" y="100" width="80" height="80" rx="4" fill="#151428" stroke="#2a2848" stroke-width="1" />
	<text x="150" y="122" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="10" font-weight="600">Conv 32</text>
	<text x="150" y="137" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">BN + ReLU</text>
	<text x="150" y="152" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="9">MaxPool</text>
	<text x="150" y="172" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">32 x 47</text>

	<line x1="190" y1="140" x2="210" y2="140" stroke="#555577" stroke-width="1" marker-end="url(#arrowPC)" />

	<!-- Conv Block 2 -->
	<rect x="210" y="105" width="80" height="70" rx="4" fill="#151428" stroke="#2a2848" stroke-width="1" />
	<text x="250" y="125" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="10" font-weight="600">Conv 64</text>
	<text x="250" y="140" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">BN + ReLU</text>
	<text x="250" y="155" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="9">MaxPool</text>
	<text x="250" y="170" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">16 x 23</text>

	<line x1="290" y1="140" x2="310" y2="140" stroke="#555577" stroke-width="1" marker-end="url(#arrowPC)" />

	<!-- Conv Block 3 -->
	<rect x="310" y="110" width="80" height="60" rx="4" fill="#151428" stroke="#2a2848" stroke-width="1" />
	<text x="350" y="130" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="10" font-weight="600">Conv 128</text>
	<text x="350" y="145" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">BN + ReLU</text>
	<text x="350" y="158" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="9">MaxPool</text>
	<text x="350" y="168" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">8 x 11</text>

	<line x1="390" y1="140" x2="410" y2="140" stroke="#555577" stroke-width="1" marker-end="url(#arrowPC)" />

	<!-- Conv Block 4 -->
	<rect x="410" y="115" width="80" height="50" rx="4" fill="#151428" stroke="#2a2848" stroke-width="1" />
	<text x="450" y="135" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="10" font-weight="600">Conv 256</text>
	<text x="450" y="150" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">BN + ReLU</text>
	<text x="450" y="162" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">8 x 11</text>

	<line x1="490" y1="140" x2="510" y2="140" stroke="#555577" stroke-width="1" marker-end="url(#arrowPC)" />

	<!-- GAP -- highlighted as key innovation -->
	<rect x="510" y="95" width="120" height="90" rx="4" fill="#151428" stroke="#00ccaa" stroke-width="1.5" />
	<text x="570" y="115" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="10" font-weight="600">Global Avg Pool</text>
	<!-- Visual: 256 feature maps each averaged to one number -->
	<!-- Show small grids collapsing to dots -->
	<g transform="translate(525, 122)">
		<!-- Feature map 1 -->
		<rect x="0" y="0" width="16" height="12" rx="1" fill="#33ddff" opacity="0.15" stroke="#33ddff" stroke-width="0.5" />
		<line x1="20" y1="6" x2="28" y2="6" stroke="#555577" stroke-width="0.5" />
		<circle cx="32" cy="6" r="3" fill="#33ddff" opacity="0.5" />
		<!-- Feature map 2 -->
		<rect x="0" y="18" width="16" height="12" rx="1" fill="#ff3388" opacity="0.15" stroke="#ff3388" stroke-width="0.5" />
		<line x1="20" y1="24" x2="28" y2="24" stroke="#555577" stroke-width="0.5" />
		<circle cx="32" cy="24" r="3" fill="#ff3388" opacity="0.5" />
		<!-- Feature map 3 -->
		<rect x="0" y="36" width="16" height="12" rx="1" fill="#00ccaa" opacity="0.15" stroke="#00ccaa" stroke-width="0.5" />
		<line x1="20" y1="42" x2="28" y2="42" stroke="#555577" stroke-width="0.5" />
		<circle cx="32" cy="42" r="3" fill="#00ccaa" opacity="0.5" />
	</g>
	<text x="595" y="143" text-anchor="middle" fill="#555577" font-family="monospace" font-size="8">x 256</text>
	<text x="570" y="178" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">256 numbers</text>

	<line x1="630" y1="140" x2="650" y2="140" stroke="#555577" stroke-width="1" marker-end="url(#arrowPC)" />

	<!-- Dropout -->
	<rect x="650" y="120" width="70" height="40" rx="4" fill="#151428" stroke="#2a2848" stroke-width="1" />
	<text x="685" y="140" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="10" font-weight="600">Dropout</text>
	<text x="685" y="155" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">0.4</text>

	<line x1="720" y1="140" x2="740" y2="140" stroke="#555577" stroke-width="1" marker-end="url(#arrowPC)" />

	<!-- Output Dense 138 -->
	<rect x="740" y="115" width="90" height="50" rx="4" fill="none" stroke="#00ccaa" stroke-width="1.5" />
	<text x="785" y="138" text-anchor="middle" fill="#e8e0f0" font-family="system-ui, sans-serif" font-size="10" font-weight="600">Dense 138</text>
	<text x="785" y="155" text-anchor="middle" fill="#555577" font-family="monospace" font-size="9">output</text>

	<!-- Annotation: simple classifier -->
	<rect x="510" y="205" width="220" height="28" rx="3" fill="#151428" stroke="#00ccaa" stroke-width="0.5" opacity="0.8" />
	<text x="620" y="223" text-anchor="middle" fill="#00ccaa" font-family="system-ui, sans-serif" font-size="9">~35K classifier params (256 -> 138, minimal)</text>

	<!-- Bracket under GAP region -->
	<line x1="510" y1="190" x2="510" y2="205" stroke="#00ccaa" stroke-width="0.5" stroke-dasharray="3,2" />
	<line x1="730" y1="190" x2="730" y2="205" stroke="#00ccaa" stroke-width="0.5" stroke-dasharray="3,2" />

	<!-- Spatial dimension annotations along top -->
	<text x="50" y="92" text-anchor="middle" fill="#555577" font-family="monospace" font-size="8">64x94</text>
	<text x="150" y="95" text-anchor="middle" fill="#555577" font-family="monospace" font-size="8">32x47</text>
	<text x="250" y="100" text-anchor="middle" fill="#555577" font-family="monospace" font-size="8">16x23</text>
	<text x="350" y="105" text-anchor="middle" fill="#555577" font-family="monospace" font-size="8">8x11</text>
	<text x="450" y="110" text-anchor="middle" fill="#555577" font-family="monospace" font-size="8">8x11</text>
	<text x="570" y="88" text-anchor="middle" fill="#555577" font-family="monospace" font-size="8">1x1</text>
</svg>
{/if}

<style>
	.diagram {
		width: 100%;
		height: auto;
		display: block;
		margin: 16px 0;
	}
</style>
