
// this file is generated — do not edit it


/// <reference types="@sveltejs/kit" />

/**
 * Environment variables [loaded by Vite](https://vitejs.dev/guide/env-and-mode.html#env-files) from `.env` files and `process.env`. Like [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), this module cannot be imported into client-side code. This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured).
 * 
 * _Unlike_ [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), the values exported from this module are statically injected into your bundle at build time, enabling optimisations like dead code elimination.
 * 
 * ```ts
 * import { API_KEY } from '$env/static/private';
 * ```
 * 
 * Note that all environment variables referenced in your code should be declared (for example in an `.env` file), even if they don't have a value until the app is deployed:
 * 
 * ```
 * MY_FEATURE_FLAG=""
 * ```
 * 
 * You can override `.env` values from the command line like so:
 * 
 * ```sh
 * MY_FEATURE_FLAG="enabled" npm run dev
 * ```
 */
declare module '$env/static/private' {
	export const PA_API_KEY: string;
	export const MANPATH: string;
	export const STARSHIP_SHELL: string;
	export const NoDefaultCurrentDirectoryInExePath: string;
	export const GHOSTTY_RESOURCES_DIR: string;
	export const rvm_use_flag: string;
	export const CLAUDE_CODE_ENTRYPOINT: string;
	export const TERM_PROGRAM: string;
	export const rvm_bin_path: string;
	export const NODE: string;
	export const INIT_CWD: string;
	export const GEM_HOME: string;
	export const rvm_quiet_flag: string;
	export const SHELL: string;
	export const TERM: string;
	export const rvm_gemstone_url: string;
	export const HOMEBREW_REPOSITORY: string;
	export const IRBRC: string;
	export const TMPDIR: string;
	export const rvm_docs_type: string;
	export const npm_config_global_prefix: string;
	export const CONDA_SHLVL: string;
	export const CONDA_PROMPT_MODIFIER: string;
	export const TERM_PROGRAM_VERSION: string;
	export const KO_DOCKER_REPO: string;
	export const ZPFX: string;
	export const COLOR: string;
	export const MY_RUBY_HOME: string;
	export const rvm_hook: string;
	export const npm_config_noproxy: string;
	export const npm_config_local_prefix: string;
	export const LC_ALL: string;
	export const GIT_EDITOR: string;
	export const PMSPEC: string;
	export const USER: string;
	export const rvm_gemstone_package_file: string;
	export const COMMAND_MODE: string;
	export const npm_config_globalconfig: string;
	export const CONDA_EXE: string;
	export const KUBECONFIG: string;
	export const rvm_path: string;
	export const CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS: string;
	export const SSH_AUTH_SOCK: string;
	export const __CF_USER_TEXT_ENCODING: string;
	export const npm_execpath: string;
	export const rvm_proxy: string;
	export const VERCEL_PLUGIN_BOOTSTRAP_HINTS: string;
	export const FZF_DEFAULT_OPTS: string;
	export const rvm_ruby_file: string;
	export const TMUX: string;
	export const _CE_CONDA: string;
	export const rvm_prefix: string;
	export const rvm_silent_flag: string;
	export const PATH: string;
	export const rvm_ruby_make: string;
	export const VERCEL_PLUGIN_AGENT_BROWSER_AVAILABLE: string;
	export const npm_package_json: string;
	export const _: string;
	export const GHOSTTY_SHELL_FEATURES: string;
	export const LaunchInstanceID: string;
	export const npm_config_userconfig: string;
	export const npm_config_init_module: string;
	export const CONDA_PREFIX: string;
	export const __CFBundleIdentifier: string;
	export const npm_command: string;
	export const PWD: string;
	export const FZF_ALT_C_COMMAND: string;
	export const JAVA_HOME: string;
	export const npm_lifecycle_event: string;
	export const OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE: string;
	export const EDITOR: string;
	export const npm_package_name: string;
	export const LANG: string;
	export const rvm_sdk: string;
	export const CLOUDSDK_PYTHON: string;
	export const npm_config_npm_version: string;
	export const TMUX_PANE: string;
	export const XPC_FLAGS: string;
	export const npm_config_node_gyp: string;
	export const npm_package_version: string;
	export const KRB5CCNAME: string;
	export const XPC_SERVICE_NAME: string;
	export const _CE_M: string;
	export const rvm_version: string;
	export const HOME: string;
	export const SHLVL: string;
	export const rvm_pretty_print_flag: string;
	export const rvm_script_name: string;
	export const TERMINFO: string;
	export const rvm_ruby_mode: string;
	export const KO_INSECURE_REGISTRY: string;
	export const HOMEBREW_PREFIX: string;
	export const ZSH_CACHE_DIR: string;
	export const npm_config_cache: string;
	export const CONDA_PYTHON_EXE: string;
	export const LOGNAME: string;
	export const STARSHIP_SESSION_KEY: string;
	export const npm_lifecycle_script: string;
	export const VISUAL: string;
	export const rvm_alias_expanded: string;
	export const FZF_CTRL_T_COMMAND: string;
	export const GEM_PATH: string;
	export const XDG_DATA_DIRS: string;
	export const COREPACK_ENABLE_AUTO_PIN: string;
	export const FZF_DEFAULT_COMMAND: string;
	export const GHOSTTY_BIN_DIR: string;
	export const TMUX_PLUGIN_MANAGER_PATH: string;
	export const BUN_INSTALL: string;
	export const CONDA_DEFAULT_ENV: string;
	export const GOPATH: string;
	export const npm_config_user_agent: string;
	export const rvm_nightly_flag: string;
	export const HOMEBREW_CELLAR: string;
	export const INFOPATH: string;
	export const rvm_ruby_make_install: string;
	export const rvm_niceness: string;
	export const BAT_THEME: string;
	export const OSLogRateLimit: string;
	export const rvm_ruby_bits: string;
	export const docker: string;
	export const rvm_bin_flag: string;
	export const CLAUDECODE: string;
	export const RUBY_VERSION: string;
	export const SECURITYSESSIONID: string;
	export const is_vim: string;
	export const rvm_only_path_flag: string;
	export const PA_API_URL: string;
	export const npm_node_execpath: string;
	export const npm_config_prefix: string;
	export const COLORTERM: string;
	export const NODE_ENV: string;
}

/**
 * Similar to [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private), except that it only includes environment variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`), and can therefore safely be exposed to client-side code.
 * 
 * Values are replaced statically at build time.
 * 
 * ```ts
 * import { PUBLIC_BASE_URL } from '$env/static/public';
 * ```
 */
declare module '$env/static/public' {
	
}

/**
 * This module provides access to runtime environment variables, as defined by the platform you're running on. For example if you're using [`adapter-node`](https://github.com/sveltejs/kit/tree/main/packages/adapter-node) (or running [`vite preview`](https://svelte.dev/docs/kit/cli)), this is equivalent to `process.env`. This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured).
 * 
 * This module cannot be imported into client-side code.
 * 
 * ```ts
 * import { env } from '$env/dynamic/private';
 * console.log(env.DEPLOYMENT_SPECIFIC_VARIABLE);
 * ```
 * 
 * > [!NOTE] In `dev`, `$env/dynamic` always includes environment variables from `.env`. In `prod`, this behavior will depend on your adapter.
 */
declare module '$env/dynamic/private' {
	export const env: {
		PA_API_KEY: string;
		MANPATH: string;
		STARSHIP_SHELL: string;
		NoDefaultCurrentDirectoryInExePath: string;
		GHOSTTY_RESOURCES_DIR: string;
		rvm_use_flag: string;
		CLAUDE_CODE_ENTRYPOINT: string;
		TERM_PROGRAM: string;
		rvm_bin_path: string;
		NODE: string;
		INIT_CWD: string;
		GEM_HOME: string;
		rvm_quiet_flag: string;
		SHELL: string;
		TERM: string;
		rvm_gemstone_url: string;
		HOMEBREW_REPOSITORY: string;
		IRBRC: string;
		TMPDIR: string;
		rvm_docs_type: string;
		npm_config_global_prefix: string;
		CONDA_SHLVL: string;
		CONDA_PROMPT_MODIFIER: string;
		TERM_PROGRAM_VERSION: string;
		KO_DOCKER_REPO: string;
		ZPFX: string;
		COLOR: string;
		MY_RUBY_HOME: string;
		rvm_hook: string;
		npm_config_noproxy: string;
		npm_config_local_prefix: string;
		LC_ALL: string;
		GIT_EDITOR: string;
		PMSPEC: string;
		USER: string;
		rvm_gemstone_package_file: string;
		COMMAND_MODE: string;
		npm_config_globalconfig: string;
		CONDA_EXE: string;
		KUBECONFIG: string;
		rvm_path: string;
		CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS: string;
		SSH_AUTH_SOCK: string;
		__CF_USER_TEXT_ENCODING: string;
		npm_execpath: string;
		rvm_proxy: string;
		VERCEL_PLUGIN_BOOTSTRAP_HINTS: string;
		FZF_DEFAULT_OPTS: string;
		rvm_ruby_file: string;
		TMUX: string;
		_CE_CONDA: string;
		rvm_prefix: string;
		rvm_silent_flag: string;
		PATH: string;
		rvm_ruby_make: string;
		VERCEL_PLUGIN_AGENT_BROWSER_AVAILABLE: string;
		npm_package_json: string;
		_: string;
		GHOSTTY_SHELL_FEATURES: string;
		LaunchInstanceID: string;
		npm_config_userconfig: string;
		npm_config_init_module: string;
		CONDA_PREFIX: string;
		__CFBundleIdentifier: string;
		npm_command: string;
		PWD: string;
		FZF_ALT_C_COMMAND: string;
		JAVA_HOME: string;
		npm_lifecycle_event: string;
		OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE: string;
		EDITOR: string;
		npm_package_name: string;
		LANG: string;
		rvm_sdk: string;
		CLOUDSDK_PYTHON: string;
		npm_config_npm_version: string;
		TMUX_PANE: string;
		XPC_FLAGS: string;
		npm_config_node_gyp: string;
		npm_package_version: string;
		KRB5CCNAME: string;
		XPC_SERVICE_NAME: string;
		_CE_M: string;
		rvm_version: string;
		HOME: string;
		SHLVL: string;
		rvm_pretty_print_flag: string;
		rvm_script_name: string;
		TERMINFO: string;
		rvm_ruby_mode: string;
		KO_INSECURE_REGISTRY: string;
		HOMEBREW_PREFIX: string;
		ZSH_CACHE_DIR: string;
		npm_config_cache: string;
		CONDA_PYTHON_EXE: string;
		LOGNAME: string;
		STARSHIP_SESSION_KEY: string;
		npm_lifecycle_script: string;
		VISUAL: string;
		rvm_alias_expanded: string;
		FZF_CTRL_T_COMMAND: string;
		GEM_PATH: string;
		XDG_DATA_DIRS: string;
		COREPACK_ENABLE_AUTO_PIN: string;
		FZF_DEFAULT_COMMAND: string;
		GHOSTTY_BIN_DIR: string;
		TMUX_PLUGIN_MANAGER_PATH: string;
		BUN_INSTALL: string;
		CONDA_DEFAULT_ENV: string;
		GOPATH: string;
		npm_config_user_agent: string;
		rvm_nightly_flag: string;
		HOMEBREW_CELLAR: string;
		INFOPATH: string;
		rvm_ruby_make_install: string;
		rvm_niceness: string;
		BAT_THEME: string;
		OSLogRateLimit: string;
		rvm_ruby_bits: string;
		docker: string;
		rvm_bin_flag: string;
		CLAUDECODE: string;
		RUBY_VERSION: string;
		SECURITYSESSIONID: string;
		is_vim: string;
		rvm_only_path_flag: string;
		PA_API_URL: string;
		npm_node_execpath: string;
		npm_config_prefix: string;
		COLORTERM: string;
		NODE_ENV: string;
		[key: `PUBLIC_${string}`]: undefined;
		[key: `${string}`]: string | undefined;
	}
}

/**
 * Similar to [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), but only includes variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`), and can therefore safely be exposed to client-side code.
 * 
 * Note that public dynamic environment variables must all be sent from the server to the client, causing larger network requests — when possible, use `$env/static/public` instead.
 * 
 * ```ts
 * import { env } from '$env/dynamic/public';
 * console.log(env.PUBLIC_DEPLOYMENT_SPECIFIC_VARIABLE);
 * ```
 */
declare module '$env/dynamic/public' {
	export const env: {
		[key: `PUBLIC_${string}`]: string | undefined;
	}
}
