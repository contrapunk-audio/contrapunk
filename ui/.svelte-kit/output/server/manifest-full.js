export const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set(["favicon.png","logo.svg","samples/by_class/string_0_fret_0.wav","samples/by_class/string_0_fret_1.wav","samples/by_class/string_0_fret_10.wav","samples/by_class/string_0_fret_11.wav","samples/by_class/string_0_fret_12.wav","samples/by_class/string_0_fret_13.wav","samples/by_class/string_0_fret_14.wav","samples/by_class/string_0_fret_15.wav","samples/by_class/string_0_fret_16.wav","samples/by_class/string_0_fret_17.wav","samples/by_class/string_0_fret_18.wav","samples/by_class/string_0_fret_19.wav","samples/by_class/string_0_fret_2.wav","samples/by_class/string_0_fret_20.wav","samples/by_class/string_0_fret_21.wav","samples/by_class/string_0_fret_22.wav","samples/by_class/string_0_fret_3.wav","samples/by_class/string_0_fret_4.wav","samples/by_class/string_0_fret_5.wav","samples/by_class/string_0_fret_6.wav","samples/by_class/string_0_fret_7.wav","samples/by_class/string_0_fret_8.wav","samples/by_class/string_0_fret_9.wav","samples/by_class/string_1_fret_0.wav","samples/by_class/string_1_fret_1.wav","samples/by_class/string_1_fret_10.wav","samples/by_class/string_1_fret_11.wav","samples/by_class/string_1_fret_12.wav","samples/by_class/string_1_fret_13.wav","samples/by_class/string_1_fret_14.wav","samples/by_class/string_1_fret_15.wav","samples/by_class/string_1_fret_16.wav","samples/by_class/string_1_fret_17.wav","samples/by_class/string_1_fret_18.wav","samples/by_class/string_1_fret_19.wav","samples/by_class/string_1_fret_2.wav","samples/by_class/string_1_fret_20.wav","samples/by_class/string_1_fret_21.wav","samples/by_class/string_1_fret_22.wav","samples/by_class/string_1_fret_3.wav","samples/by_class/string_1_fret_4.wav","samples/by_class/string_1_fret_5.wav","samples/by_class/string_1_fret_6.wav","samples/by_class/string_1_fret_7.wav","samples/by_class/string_1_fret_8.wav","samples/by_class/string_1_fret_9.wav","samples/by_class/string_2_fret_0.wav","samples/by_class/string_2_fret_1.wav","samples/by_class/string_2_fret_10.wav","samples/by_class/string_2_fret_11.wav","samples/by_class/string_2_fret_12.wav","samples/by_class/string_2_fret_13.wav","samples/by_class/string_2_fret_14.wav","samples/by_class/string_2_fret_15.wav","samples/by_class/string_2_fret_16.wav","samples/by_class/string_2_fret_17.wav","samples/by_class/string_2_fret_18.wav","samples/by_class/string_2_fret_19.wav","samples/by_class/string_2_fret_2.wav","samples/by_class/string_2_fret_20.wav","samples/by_class/string_2_fret_21.wav","samples/by_class/string_2_fret_22.wav","samples/by_class/string_2_fret_3.wav","samples/by_class/string_2_fret_4.wav","samples/by_class/string_2_fret_5.wav","samples/by_class/string_2_fret_6.wav","samples/by_class/string_2_fret_7.wav","samples/by_class/string_2_fret_8.wav","samples/by_class/string_2_fret_9.wav","samples/by_class/string_3_fret_0.wav","samples/by_class/string_3_fret_1.wav","samples/by_class/string_3_fret_10.wav","samples/by_class/string_3_fret_11.wav","samples/by_class/string_3_fret_12.wav","samples/by_class/string_3_fret_13.wav","samples/by_class/string_3_fret_14.wav","samples/by_class/string_3_fret_15.wav","samples/by_class/string_3_fret_16.wav","samples/by_class/string_3_fret_17.wav","samples/by_class/string_3_fret_18.wav","samples/by_class/string_3_fret_19.wav","samples/by_class/string_3_fret_2.wav","samples/by_class/string_3_fret_20.wav","samples/by_class/string_3_fret_21.wav","samples/by_class/string_3_fret_22.wav","samples/by_class/string_3_fret_3.wav","samples/by_class/string_3_fret_4.wav","samples/by_class/string_3_fret_5.wav","samples/by_class/string_3_fret_6.wav","samples/by_class/string_3_fret_7.wav","samples/by_class/string_3_fret_8.wav","samples/by_class/string_3_fret_9.wav","samples/by_class/string_4_fret_0.wav","samples/by_class/string_4_fret_1.wav","samples/by_class/string_4_fret_10.wav","samples/by_class/string_4_fret_11.wav","samples/by_class/string_4_fret_12.wav","samples/by_class/string_4_fret_13.wav","samples/by_class/string_4_fret_14.wav","samples/by_class/string_4_fret_15.wav","samples/by_class/string_4_fret_16.wav","samples/by_class/string_4_fret_17.wav","samples/by_class/string_4_fret_18.wav","samples/by_class/string_4_fret_19.wav","samples/by_class/string_4_fret_2.wav","samples/by_class/string_4_fret_20.wav","samples/by_class/string_4_fret_21.wav","samples/by_class/string_4_fret_22.wav","samples/by_class/string_4_fret_3.wav","samples/by_class/string_4_fret_4.wav","samples/by_class/string_4_fret_5.wav","samples/by_class/string_4_fret_6.wav","samples/by_class/string_4_fret_7.wav","samples/by_class/string_4_fret_8.wav","samples/by_class/string_4_fret_9.wav","samples/by_class/string_5_fret_0.wav","samples/by_class/string_5_fret_1.wav","samples/by_class/string_5_fret_10.wav","samples/by_class/string_5_fret_11.wav","samples/by_class/string_5_fret_12.wav","samples/by_class/string_5_fret_13.wav","samples/by_class/string_5_fret_14.wav","samples/by_class/string_5_fret_15.wav","samples/by_class/string_5_fret_16.wav","samples/by_class/string_5_fret_17.wav","samples/by_class/string_5_fret_18.wav","samples/by_class/string_5_fret_19.wav","samples/by_class/string_5_fret_2.wav","samples/by_class/string_5_fret_20.wav","samples/by_class/string_5_fret_21.wav","samples/by_class/string_5_fret_22.wav","samples/by_class/string_5_fret_3.wav","samples/by_class/string_5_fret_4.wav","samples/by_class/string_5_fret_5.wav","samples/by_class/string_5_fret_6.wav","samples/by_class/string_5_fret_7.wav","samples/by_class/string_5_fret_8.wav","samples/by_class/string_5_fret_9.wav","samples/confused_pairs/A2_on_A_string_open.wav","samples/confused_pairs/A2_on_E_string_fret5.wav","samples/confused_pairs/B3_on_B_string_open.wav","samples/confused_pairs/B3_on_G_string_fret4.wav","samples/confused_pairs/D3_on_A_string_fret5.wav","samples/confused_pairs/D3_on_D_string_open.wav","samples/confused_pairs/E4_on_B_string_fret5.wav","samples/confused_pairs/E4_on_E_string_open.wav","samples/confused_pairs/G3_on_D_string_fret5.wav","samples/confused_pairs/G3_on_G_string_open.wav","samples/index.json","samples/showcase/A2_fret12.wav","samples/showcase/A2_fret5.wav","samples/showcase/A2_open.wav","samples/showcase/B3_fret12.wav","samples/showcase/B3_fret5.wav","samples/showcase/B3_open.wav","samples/showcase/D3_fret12.wav","samples/showcase/D3_fret5.wav","samples/showcase/D3_open.wav","samples/showcase/E2_fret12.wav","samples/showcase/E2_fret5.wav","samples/showcase/E2_open.wav","samples/showcase/E4_fret12.wav","samples/showcase/E4_fret5.wav","samples/showcase/E4_open.wav","samples/showcase/G3_fret12.wav","samples/showcase/G3_fret5.wav","samples/showcase/G3_open.wav","spectrograms/confused_pairs/A2_A2_string_open.json","spectrograms/confused_pairs/A2_E2_string_fret5.json","spectrograms/confused_pairs/D3_A2_string_fret5.json","spectrograms/confused_pairs/D3_D3_string_open.json","spectrograms/confused_pairs/G3_D3_string_fret5.json","spectrograms/confused_pairs/G3_G3_string_open.json","spectrograms/index.json","spectrograms/quality/clipped_sample.json","spectrograms/quality/normal_sample.json","spectrograms/showcase/A2_open.json","spectrograms/showcase/B3_open.json","spectrograms/showcase/D3_open.json","spectrograms/showcase/E2_open.json","spectrograms/showcase/E4_open.json","spectrograms/showcase/G3_open.json","training/concepts.json","training/round_01/confusion_data.json","training/round_01/confusion_hybrid_cnn.png","training/round_01/confusion_pure_cnn.png","training/round_01/confusion_random_forest.png","training/round_01/fret_accuracy.json","training/round_01/fret_heatmap_hybrid_cnn.png","training/round_01/fret_heatmap_pure_cnn.png","training/round_01/fret_heatmap_random_forest.png","training/round_01/per_string_hybrid_cnn.png","training/round_01/per_string_pure_cnn.png","training/round_01/per_string_random_forest.png","training/round_01/results.json","training/round_01/training_curves_hybrid_cnn.png","training/round_01/training_curves_pure_cnn.png","training/round_01/training_meta.json","training/round_02/before_after_spectrograms.png","training/round_02/comparison_bars.png","training/round_02/onset_distribution.png","training/round_02/results.json","training/round_03/comparison_bars.png","training/round_03/removed_samples.png","training/round_03/results.json","training/round_04/results.json","training/round_04/round_04_comparison.png","training/round_04/round_04_delta.png","training/round_05/augmentation_examples.png","training/round_05/results.json","training/round_05/round_05_comparison.png","training/round_05/training_curves_hybrid_cnn.png","training/round_05/training_curves_pure_cnn.png"]),
	mimeTypes: {".png":"image/png",".svg":"image/svg+xml",".wav":"audio/wav",".json":"application/json"},
	_: {
		client: {start:"_app/immutable/entry/start.Dni8fiRr.js",app:"_app/immutable/entry/app.CzxdLqHL.js",imports:["_app/immutable/entry/start.Dni8fiRr.js","_app/immutable/chunks/Q5HKDPz-.js","_app/immutable/chunks/nVkCXwlp.js","_app/immutable/chunks/CIumhm75.js","_app/immutable/entry/app.CzxdLqHL.js","_app/immutable/chunks/CYEXfWnH.js","_app/immutable/chunks/nVkCXwlp.js","_app/immutable/chunks/C7gO59vg.js","_app/immutable/chunks/eZFB7GD9.js","_app/immutable/chunks/CIumhm75.js","_app/immutable/chunks/Cswa20vR.js","_app/immutable/chunks/C_o2MEX2.js","_app/immutable/chunks/BxTRC_am.js","_app/immutable/chunks/9CnBRr9g.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
		nodes: [
			__memo(() => import('./nodes/0.js')),
			__memo(() => import('./nodes/1.js')),
			__memo(() => import('./nodes/2.js')),
			__memo(() => import('./nodes/3.js')),
			__memo(() => import('./nodes/4.js')),
			__memo(() => import('./nodes/5.js')),
			__memo(() => import('./nodes/6.js')),
			__memo(() => import('./nodes/7.js')),
			__memo(() => import('./nodes/8.js')),
			__memo(() => import('./nodes/9.js')),
			__memo(() => import('./nodes/10.js')),
			__memo(() => import('./nodes/11.js')),
			__memo(() => import('./nodes/12.js'))
		],
		remotes: {
			
		},
		routes: [
			{
				id: "/",
				pattern: /^\/$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 3 },
				endpoint: null
			},
			{
				id: "/diary",
				pattern: /^\/diary\/?$/,
				params: [],
				page: { layouts: [0,2,], errors: [1,,], leaf: 4 },
				endpoint: null
			},
			{
				id: "/diary/machine-learning",
				pattern: /^\/diary\/machine-learning\/?$/,
				params: [],
				page: { layouts: [0,2,], errors: [1,,], leaf: 5 },
				endpoint: null
			},
			{
				id: "/diary/machine-learning/playground",
				pattern: /^\/diary\/machine-learning\/playground\/?$/,
				params: [],
				page: { layouts: [0,2,], errors: [1,,], leaf: 6 },
				endpoint: null
			},
			{
				id: "/diary/machine-learning/round-1",
				pattern: /^\/diary\/machine-learning\/round-1\/?$/,
				params: [],
				page: { layouts: [0,2,], errors: [1,,], leaf: 7 },
				endpoint: null
			},
			{
				id: "/diary/machine-learning/round-2",
				pattern: /^\/diary\/machine-learning\/round-2\/?$/,
				params: [],
				page: { layouts: [0,2,], errors: [1,,], leaf: 8 },
				endpoint: null
			},
			{
				id: "/diary/machine-learning/round-3",
				pattern: /^\/diary\/machine-learning\/round-3\/?$/,
				params: [],
				page: { layouts: [0,2,], errors: [1,,], leaf: 9 },
				endpoint: null
			},
			{
				id: "/diary/machine-learning/round-4",
				pattern: /^\/diary\/machine-learning\/round-4\/?$/,
				params: [],
				page: { layouts: [0,2,], errors: [1,,], leaf: 10 },
				endpoint: null
			},
			{
				id: "/diary/machine-learning/round-5",
				pattern: /^\/diary\/machine-learning\/round-5\/?$/,
				params: [],
				page: { layouts: [0,2,], errors: [1,,], leaf: 11 },
				endpoint: null
			},
			{
				id: "/diary/machine-learning/the-pivot",
				pattern: /^\/diary\/machine-learning\/the-pivot\/?$/,
				params: [],
				page: { layouts: [0,2,], errors: [1,,], leaf: 12 },
				endpoint: null
			}
		],
		prerendered_routes: new Set([]),
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();
