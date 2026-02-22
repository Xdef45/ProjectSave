<script lang="ts">
	import { goto } from '$app/navigation';
	import { t } from '$lib/i18n';
	import { isOnline, checkConnection } from '$lib/stores/connection';
	import { orchestrateLoginFlow } from '$lib/services/auth';
	import { fade, fly } from 'svelte/transition';

	// --- État du formulaire ---
	let isRegistering = false;
	let username = '';
	let password = '';
	let confirmPassword = '';

	// --- État de chargement et d'erreur ---
	let isLoading = false;
	let loadingState = '';
	let showRules = false;
	let error = '';

	// Noms d'utilisateurs réservés par le système (interdits pour l'inscription)
	const FORBIDDEN_USERNAMES = [
		'root',
		'daemon',
		'bin',
		'sys',
		'sync',
		'sudo',
		'admin',
		'administrator'
	];

	// Nettoyage automatique du nom d'utilisateur (réactivité Svelte)
	$: safeUsername = username.trim().toLowerCase();

	// Validation des règles du nom d'utilisateur en temps réel
	$: usernameErrors = {
		min: isRegistering && safeUsername.length > 0 && safeUsername.length < 3,
		start: isRegistering && safeUsername.length > 0 && !/^[a-z]/.test(safeUsername),
		chars: isRegistering && safeUsername.length > 0 && !/^[a-z0-9_-]+$/.test(safeUsername),
		reserved: isRegistering && FORBIDDEN_USERNAMES.includes(safeUsername)
	};

	$: isUsernameValid =
		!isRegistering ||
		(safeUsername.length >= 3 &&
			/^[a-z]/.test(safeUsername) &&
			/^[a-z0-9_-]+$/.test(safeUsername) &&
			!FORBIDDEN_USERNAMES.includes(safeUsername));

	// Règles de sécurité du mot de passe
	$: rules = {
		length: { valid: password.length >= 12 },
		lower: { valid: /[a-z]/.test(password) },
		upper: { valid: /[A-Z]/.test(password) },
		number: { valid: /[0-9]/.test(password) },
		special: { valid: /[^A-Za-z0-9]/.test(password) }
	};

	$: isPasswordValid = isRegistering
		? Object.values(rules).every((r) => r.valid)
		: password.length > 0;

	$: passwordsMatch = !isRegistering || password === confirmPassword;

	// Détermine si le bouton de soumission doit être cliquable
	$: canSubmit = isUsernameValid && isPasswordValid && passwordsMatch && safeUsername.length > 0;

	// Bascule entre l'écran de connexion et celui d'inscription
	function toggleMode() {
		if (isLoading) return;
		isRegistering = !isRegistering;
		error = '';
		showRules = false;

		// SÉCURITÉ : On vide systématiquement les mots de passe lors d'un changement de vue
		password = '';
		confirmPassword = '';
	}

	// Gestionnaire principal de l'authentification
	async function handleAuth() {
		if (!canSubmit || isLoading) return;

		error = '';
		isLoading = true;
		loadingState = $t('common.loading');

		// Vérification de la connectivité réseau avant de lancer le flux
		const online = await checkConnection();
		if (!online) {
			error = $t('errors.offline');
			isLoading = false;
			return;
		}

		try {
			// Lancement du flux complexe d'authentification (génération des clés, communication backend, etc.)
			await orchestrateLoginFlow(
				{ username: safeUsername, password },
				isRegistering,
				(translationKey) => {
					loadingState = $t(translationKey); // Mise à jour dynamique du texte de chargement
				}
			);

			await new Promise((r) => setTimeout(r, 600));
			localStorage.setItem('currentClientName', safeUsername);

			// SÉCURITÉ : Nettoyage strict des variables avant la redirection vers le tableau de bord
			password = '';
			confirmPassword = '';

			await goto('/dashboard');
		} catch (err: unknown) {
			// SÉCURITÉ : Suppression immédiate en cas d'échec pour éviter les regards indiscrets (shoulder-surfing)
			password = '';
			confirmPassword = '';

			// Formatage défensif de l'erreur renvoyée
			let rawMsg = '';
			if (err instanceof Error) {
				rawMsg = err.message;
			} else if (typeof err === 'string') {
				rawMsg = err;
			} else {
				rawMsg = String(err);
			}

			const cleanMsg = rawMsg.trim();
			const lowerMsg = cleanMsg.toLowerCase();

			// Routage des codes d'erreurs du backend vers les traductions i18n
			switch (cleanMsg) {
				case '0':
					error = $t('errors.auth_not_signup');
					break;
				case '1':
					error = $t('errors.auth_exists');
					break;
				case '2':
					error = $t('errors.auth_user_too_short');
					break;
				case '3':
					error = $t('errors.auth_invalid');
					break;
				case '4':
					error = $t('errors.auth_pass_too_short');
					break;
				case '5':
					error = $t('errors.auth_pass_special');
					break;
				case '6':
					error = $t('errors.auth_pass_upper');
					break;
				case '7':
					error = $t('errors.auth_pass_number');
					break;
				default:
					// Gestion des erreurs systèmes, matérielles ou inattendues (fichiers, WSL, SSH)
					if (lowerMsg === 'key_missing' || lowerMsg.includes('clé')) {
						error = $t('errors.auth_key_error');
					} else if (lowerMsg === 'key_save_failed') {
						error = $t('errors.auth_io_error');
					} else if (
						lowerMsg.includes('wsl') ||
						lowerMsg.includes('ssh') ||
						lowerMsg.includes('borg')
					) {
						error = `${$t('errors.system_prefix')} ${cleanMsg}`;
					} else {
						error = cleanMsg || $t('errors.unknown');
					}
					break;
			}
		} finally {
			isLoading = false;
		}
	}
</script>

<div class="login-card relative">
	{#if isLoading}
		<div
			class="bg-surface/80 rounded-card absolute inset-0 z-50 flex flex-col items-center justify-center backdrop-blur-md transition-all"
			in:fade={{ duration: 200 }}
			out:fade={{ duration: 200 }}
		>
			<div class="relative mb-6">
				<div class="border-primary/30 h-16 w-16 rounded-full border-4"></div>
				<div
					class="border-primary absolute top-0 left-0 h-16 w-16 animate-spin rounded-full border-4 border-t-transparent"
				></div>
			</div>

			{#key loadingState}
				<p
					class="text-text-main animate-pulse px-4 text-center text-lg font-semibold"
					in:fly={{ y: 10, duration: 300 }}
				>
					{loadingState}
				</p>
			{/key}
		</div>
	{/if}

	<div class="header mb-8 transition-opacity duration-300" class:opacity-20={isLoading}>
		<h2 class="mb-2 text-2xl">
			{#if isRegistering}
				{$t('register.create_account')}
			{:else}
				{$t('login.welcome')}
				<strong class="text-primary">{$t('common.brand_name')}</strong>
			{/if}
		</h2>
		<p class="text-text-muted text-sm">
			{isRegistering ? $t('register.subtitle') : $t('login.subtitle')}
		</p>
	</div>

	<form
		on:submit|preventDefault={handleAuth}
		class="transition-opacity duration-300"
		class:opacity-20={isLoading}
	>
		<div class="input-group">
			<label for="username">{$t('login.username')}</label>
			<input
				type="text"
				id="username"
				placeholder={$t('login.username_placeholder')}
				bind:value={username}
				disabled={isLoading}
				required
				autocomplete="username"
			/>
			{#if isRegistering && username.length > 0}
				{#if usernameErrors.min}
					<p class="mt-1 text-xs text-red-500">{$t('register.error.username_min')}</p>
				{:else if usernameErrors.start}
					<p class="mt-1 text-xs text-red-500">{$t('register.error.username_start')}</p>
				{:else if usernameErrors.chars}
					<p class="mt-1 text-xs text-red-500">{$t('register.error.username_chars')}</p>
				{:else if usernameErrors.reserved}
					<p class="mt-1 text-xs text-red-500">{$t('register.error.username_reserved')}</p>
				{/if}
			{/if}
		</div>

		<div class="input-group relative">
			<label for="password">{$t('login.password')}</label>
			<input
				type="password"
				id="password"
				placeholder="••••••••"
				bind:value={password}
				disabled={isLoading}
				on:focus={() => isRegistering && (showRules = true)}
				on:blur={() => (showRules = false)}
				required
				autocomplete={isRegistering ? 'new-password' : 'current-password'}
			/>

			{#if isRegistering}
				<div class="validation-popup {showRules ? 'visible' : ''}">
					<h4>{$t('login.pwd_security')}</h4>
					<ul>
						{#each Object.entries(rules) as [key, rule]}
							<li class={rule.valid ? 'valid' : 'invalid'}>
								<span aria-hidden="true">{rule.valid ? '✓' : '✕'}</span>
								{$t('rules.' + key)}
							</li>
						{/each}
					</ul>
				</div>
			{/if}
		</div>

		{#if isRegistering}
			<div class="input-group">
				<label for="confirm">{$t('register.confirm_password')}</label>
				<input
					type="password"
					id="confirm"
					placeholder="••••••••"
					bind:value={confirmPassword}
					disabled={isLoading}
					required
					autocomplete="new-password"
				/>
				{#if confirmPassword && !passwordsMatch}
					<p class="mt-1 text-xs text-red-500">
						{$t('errors.pwd_mismatch')}
					</p>
				{/if}
			</div>
		{/if}

		{#if error}
			<div
				class="mb-4 rounded border border-red-200 bg-red-50 p-3 text-sm text-red-500"
				role="alert"
			>
				{error}
			</div>
		{/if}

		<button type="submit" class="btn-login" disabled={!canSubmit || isLoading || !$isOnline}>
			{isRegistering ? $t('common.register_btn') : $t('common.login_btn')}
		</button>

		<div class="mt-4 text-center text-sm">
			<p class="text-text-muted">
				{isRegistering ? $t('register.have_account') : $t('login.no_account')}

				<button
					type="button"
					class="text-primary ml-1 cursor-pointer border-none bg-transparent p-0 font-semibold hover:underline disabled:opacity-50"
					on:click={toggleMode}
					disabled={isLoading}
				>
					{isRegistering ? $t('common.login_btn') : $t('register.create_btn')}
				</button>
			</p>
		</div>
	</form>
</div>
